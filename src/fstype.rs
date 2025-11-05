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
use std::fmt;
use std::str::FromStr;

/// Some common filesystems types
/// The String representation must be the same when creating using `from_str`
/// and when converting to `String` using `fmt::Display`
#[derive(Debug, PartialEq)]
pub enum FsType {
    /// procfs filesystem. Pseudo filesystem that exposes the kernel's process table.
    /// Usually mounted at /proc.
    Proc,
    /// overlayfs filesystem. A filesystem that combines multiple lower filesystems into a single directory.
    Overlay,
    /// tmpfs filesystem. A filesystem that provides a temporary file system stored in volatile memory.
    Tmpfs,
    /// sysfs filesystem. A filesystem that provides access to the kernel's internal device tree.
    Sysfs,
    /// btrfs filesystem. A filesystem that provides a hierarchical data structure for storing data in a compressed fashion.
    Btrfs,
    /// ext2 filesystem. A filesystem that provides a file system that is optimized for storing data on a local disk.
    Ext2,
    /// ext3 filesystem. A filesystem that provides a file system that is optimized for storing data on a local disk.
    Ext3,
    /// ext4 filesystem. A filesystem that provides a file system that is optimized for storing data on a local disk.
    Ext4,
    /// devtmpfs filesystem.
    Devtmpfs,
    /// Other filesystems.
    Other(String),
}

impl FromStr for FsType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "proc" => Ok(FsType::Proc),
            "tmpfs" => Ok(FsType::Tmpfs),
            "overlay" => Ok(FsType::Overlay),
            "sysfs" => Ok(FsType::Sysfs),
            "btrfs" => Ok(FsType::Btrfs),
            "ext2" => Ok(FsType::Ext2),
            "ext3" => Ok(FsType::Ext3),
            "ext4" => Ok(FsType::Ext4),
            "devtmpfs" => Ok(FsType::Devtmpfs),
            _ => Ok(FsType::Other(s.to_string())),
        }
    }
}

impl fmt::Display for FsType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fsname = match self {
            FsType::Proc => "proc",
            FsType::Overlay => "overlay",
            FsType::Tmpfs => "tmpfs",
            FsType::Sysfs => "sysfs",
            FsType::Btrfs => "btrfs",
            FsType::Ext2 => "ext2",
            FsType::Ext3 => "ext3",
            FsType::Ext4 => "ext4",
            FsType::Devtmpfs => "devtmpfs",
            FsType::Other(fsname) => fsname,
        };
        write!(f, "{}", fsname)
    }
}
