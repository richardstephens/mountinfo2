/**
 * The MIT License
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
