use thiserror::Error;

/// Errors that can occur when parsing mountinfo files.
#[derive(Error, Debug)]
pub enum MountInfoError {
    /// IO error occurred when reading mount information.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Error parsing a specific line in the mountinfo file.
    #[error("Failed to parse line {line}: {source}")]
    ParseError {
        line: usize,
        #[source]
        source: ParseLineError,
    },

    /// No mountinfo file was found on the system.
    #[error("No mountinfo file found")]
    NoMountInfoFile,
}

/// Errors that can occur when parsing a single line of mountinfo.
#[derive(Error, Debug)]
pub enum ParseLineError {
    /// The line format does not match the expected pattern.
    #[error("Invalid line format: does not match expected pattern")]
    InvalidFormat,

    /// Failed to parse the mount ID field.
    #[error("Failed to parse mount ID: {0}")]
    InvalidMountId(#[source] std::num::ParseIntError),

    /// Failed to parse the parent ID field.
    #[error("Failed to parse parent ID: {0}")]
    InvalidParentId(#[source] std::num::ParseIntError),

    /// The regex failed to capture expected groups.
    #[error("Regex capture failed: missing expected groups")]
    MissingCaptureGroups,
}
