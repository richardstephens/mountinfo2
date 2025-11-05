/**
 * The MIT License
 * Copyright (c) 2022 Guillem Castro
 * Copyright (c) 2025 Richard Stephens
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 */
use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::LazyLock;

mod error;
mod fstype;

pub use error::{MountInfoError, ParseLineError};

pub use fstype::FsType;

/// A struct representing a mount point.
#[derive(Debug)]
pub struct MountPoint {
    /// The id of the mount point. It is unique for each mount point,
    /// but can be resused afer a call to the umount syscall.
    pub id: Option<u32>,
    /// The id of the parent mount.
    pub parent_id: Option<u32>,
    /// The path to the directory that acts as the root for this mount point.
    pub root: Option<PathBuf>,
    // Filesystem-specific information
    pub what: String,
    /// The mount point directory relative to the root.
    pub path: PathBuf,
    /// The filesystem type.
    pub fstype: FsType,
    /// Some additional mount options
    pub options: MountOptions,
}
static PROC_MOUNTINFO_LINE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(\d*)\s(\d*)\s(\d*:\d*)\s([\S]*)\s([\S]*)\s([A-Za-z0-9,]*)\s([A-Za-z0-9:\s]*)\s\- ([\S]*)\s([\S]*)(.*)").unwrap()
});

impl MountPoint {
    /// Creates a new mount point from a line of the `/proc/self/mountinfo` file.
    fn parse_proc_mountinfo_line(line: &str) -> Result<Self, ParseLineError> {
        // The line format is:
        // <id> <parent_id> <major>:<minor> <root> <mount_point> <mount_options> <optional tags> "-" <fstype> <mount souce> <super options>
        // Ref: https://www.kernel.org/doc/Documentation/filesystems/proc.txt - /proc/<pid>/mountinfo - Information about mounts
        if !PROC_MOUNTINFO_LINE_RE.is_match(line) {
            return Err(ParseLineError::InvalidFormat);
        }
        let caps = PROC_MOUNTINFO_LINE_RE
            .captures(line)
            .ok_or(ParseLineError::MissingCaptureGroups)?;
        Ok(MountPoint {
            id: Some(
                caps[1]
                    .parse::<u32>()
                    .map_err(ParseLineError::InvalidMountId)?,
            ),
            parent_id: Some(
                caps[2]
                    .parse::<u32>()
                    .map_err(ParseLineError::InvalidParentId)?,
            ),
            root: Some(PathBuf::from(caps[4].to_string())),
            path: PathBuf::from(caps[5].to_string()),
            options: MountOptions::new(&caps[6]),
            fstype: FsType::from_str(&caps[8]).unwrap(),
            what: caps[9].to_string(),
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum ReadWrite {
    ReadOnly,
    ReadWrite,
}

/// A struct representing the mount options.
#[derive(Debug)]
pub struct MountOptions {
    /// If it was mounted as read-only or read-write.
    pub read_write: ReadWrite,
    /// Additional options, not currently parsed by this library.
    pub others: Vec<String>,
}

impl MountOptions {
    /// Creates a new mount options from a string.
    /// The string must be a comma-separated list of options.
    pub fn new(options: &str) -> Self {
        let mut read_write = ReadWrite::ReadOnly;
        let mut others = Vec::new();
        for option in options.split(',') {
            match option {
                "ro" => read_write = ReadWrite::ReadOnly,
                "rw" => read_write = ReadWrite::ReadWrite,
                &_ => others.push(option.to_owned()),
            }
        }
        MountOptions { read_write, others }
    }
}

/// A struct containing the mount information.
/// Note that it will only contain the mount points visible for the calling process.
/// If the calling process is inside a chroot, not all mount points will be visible.
#[derive(Debug)]
pub struct MountInfo {
    /// The list of mount points visible for the current process.
    pub mounting_points: Vec<MountPoint>,
}

impl MountInfo {
    /// The most "modern" file with mount information. Introduced in Linux 2.6.26.
    /// According to the docs, this should be the most reliable (and up-to-date) way to get the mount information.
    const MOUNT_INFO_FILE: &'static str = "/proc/self/mountinfo";

    /// This file should exists even in ancient versions of the Linux kernel.
    /// We use it as a fallback, if for some reason /proc/self/mountinfo is not available.
    /// Believe it or not, there are still devices running ancient versions of the Linux kernel.
    const MTAB_FILE: &'static str = "/etc/mtab";

    /// Creates a new instance of the MountInfo struct.
    /// It will read the contents of the /proc/self/mountinfo file, if it exists.
    /// If it does not exist, it will fall-back to read the contents of the /etc/mtab file.
    pub fn new() -> Result<Self, MountInfoError> {
        match Self::new_from_proc() {
            Err(MountInfoError::NoMountInfoFile) => {}
            x => {
                return x;
            }
        };
        if Path::new(MountInfo::MTAB_FILE).exists() {
            let mut mtab = File::open(MountInfo::MTAB_FILE)?;
            Ok(MountInfo {
                mounting_points: MountInfo::parse_mtab(&mut mtab).map_err(MountInfoError::Io)?,
            })
        } else {
            Err(MountInfoError::NoMountInfoFile)
        }
    }

    /// Creates a new instance of the MountInfo struct.
    /// It will read the contents of the /proc/self/mountinfo file, or fail if it does not exist.
    pub fn new_from_proc() -> Result<Self, MountInfoError> {
        if Path::new(MountInfo::MOUNT_INFO_FILE).exists() {
            let mut mtab = File::open(MountInfo::MOUNT_INFO_FILE)?;
            Ok(MountInfo {
                mounting_points: MountInfo::parse_proc_mountinfo(&mut mtab)?,
            })
        } else {
            Err(MountInfoError::NoMountInfoFile)
        }
    }

    /// Check if a certain filesystem type is mounted at the given path.
    pub fn contains<P: AsRef<Path>>(&self, mounting_point: P, fstype: FsType) -> bool {
        let path = mounting_point.as_ref();
        self.mounting_points
            .iter()
            .any(|mts| mts.path == path && mts.fstype == fstype)
    }

    /// Check if the given path is a mount point.
    pub fn is_mounted<P: AsRef<Path>>(&self, path: P) -> bool {
        self.mounting_points
            .iter()
            .any(|mts| mts.path == path.as_ref())
    }

    fn parse_proc_mountinfo(
        file: &mut dyn std::io::Read,
    ) -> Result<Vec<MountPoint>, MountInfoError> {
        let mut result = Vec::new();
        let reader = io::BufReader::new(file);
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let mpoint = MountPoint::parse_proc_mountinfo_line(&line).map_err(|source| {
                MountInfoError::ParseError {
                    line: line_num + 1,
                    source,
                }
            })?;
            result.push(mpoint);
        }
        Ok(result)
    }

    fn parse_mtab(file: &mut dyn std::io::Read) -> Result<Vec<MountPoint>, std::io::Error> {
        let mut results: Vec<MountPoint> = vec![];
        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            let l = line?;
            let parts: Vec<&str> = l.split_whitespace().collect();
            if !parts.is_empty() {
                results.push(MountPoint {
                    what: parts[0].to_string(),
                    path: PathBuf::from(parts[1]),
                    fstype: FsType::from_str(parts[2]).unwrap(),
                    options: MountOptions::new(parts[3]),
                    id: None,
                    parent_id: None,
                    root: None,
                })
            }
        }
        Ok(results)
    }
}

// unit tests
#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_load_mount_points() {
        let mut file = Cursor::new(b"tmpfs /tmp tmpfs rw,seclabel,nosuid,nodev,size=8026512k,nr_inodes=1048576,inode64 0 0");
        let munt_points = MountInfo::parse_mtab(&mut file).unwrap();
        assert_eq!(munt_points.len(), 1);
        assert_eq!(munt_points[0].what, "tmpfs".to_owned());
        assert_eq!(munt_points[0].path, PathBuf::from("/tmp"));
        assert_eq!(munt_points[0].fstype, FsType::Tmpfs);
    }

    #[test]
    fn test_contains() {
        let mut file = Cursor::new(b"tmpfs /tmp tmpfs rw,seclabel,nosuid,nodev,size=8026512k,nr_inodes=1048576,inode64 0 0");
        let mtab = MountInfo {
            mounting_points: MountInfo::parse_mtab(&mut file).unwrap(),
        };
        assert_eq!(mtab.contains("/tmp", FsType::Tmpfs), true);
    }

    #[test]
    fn test_is_mounted() {
        let mut file = Cursor::new(b"tmpfs /tmp tmpfs rw,seclabel,nosuid,nodev,size=8026512k,nr_inodes=1048576,inode64 0 0");
        let mtab = MountInfo {
            mounting_points: MountInfo::parse_mtab(&mut file).unwrap(),
        };
        assert_eq!(mtab.is_mounted("/tmp"), true);
    }

    #[test]
    fn test_mount_options() {
        let options =
            MountOptions::new("rw,seclabel,nosuid,nodev,size=8026512k,nr_inodes=1048576,inode64");
        assert_eq!(options.read_write, ReadWrite::ReadWrite);
        assert_ne!(options.others.len(), 0);
        let more_options =
            MountOptions::new("ro,seclabel,nosuid,nodev,size=8026512k,nr_inodes=1048576,inode64");
        assert_eq!(more_options.read_write, ReadWrite::ReadOnly);
        assert_ne!(more_options.others.len(), 0);
    }

    #[test]
    fn parse_proc_mountinfo_line() {
        let line: String = "48 46 253:15 / /boot/efi rw,relatime shared:46 - vfat /dev/vda15 rw,fmask=0077,dmask=0077,codepage=437,iocharset=iso8859-1,shortname=mixed,errors=remount-ro".into();
        let mount_point = MountPoint::parse_proc_mountinfo_line(&line).unwrap();
        assert_eq!(Some(48), mount_point.id);
        assert_eq!(Some(46), mount_point.parent_id);
        assert_eq!(Some(PathBuf::from("/")), mount_point.root);
        assert_eq!(&PathBuf::from("/boot/efi"), &mount_point.path);
        assert_eq!("/dev/vda15", &mount_point.what);
        assert_eq!(FsType::Other("vfat".into()), mount_point.fstype);
        assert_eq!(ReadWrite::ReadWrite, mount_point.options.read_write);
    }
}
