//! The error for the loading of a configuration

use thiserror::Error;

use crate::error::deserialize::DeserializeConfigurationError;
use crate::loader::ConfigurationPath;

/// The error returned when a configuration cannot be loaded
///
/// The variants separate the failures that a caller acts on in different
/// ways. Failures of a file name the path, so that a report tells the user
/// which file to fix. A failure of caller-supplied contents has no path and
/// points into the application instead.
///
/// A failure during deserialization carries a
/// [`DeserializeConfigurationError`] as its cause, which
/// [`Error::source`][source] exposes for error reports. The message of the
/// cause names the position or the field that has to change.
///
/// A later release can add variants, and it can add fields to a variant.
/// Match with a wildcard arm, and bind the fields of a variant with `..`.
///
/// [source]: https://doc.rust-lang.org/std/error/trait.Error.html#method.source
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum LoadConfigurationError {
    /// The caller-supplied contents do not deserialize
    ///
    /// The loader got its contents from the caller, not from a file.
    /// Contents that an application embeds or computes are part of the
    /// application, so this failure is a defect to report to its
    /// developers, not a file that the user can correct.
    #[error("failed to deserialize the configuration contents")]
    #[non_exhaustive]
    InvalidContents {
        /// The cause of the failure
        source: DeserializeConfigurationError,
    },

    /// The contents of the file do not deserialize
    ///
    /// The file exists, and the crate read it. Its contents are not valid
    /// TOML, or they do not match the caller's type. The user can open the
    /// file at the path and correct it. The cause names the position or the
    /// field that has to change.
    #[error("failed to deserialize the configuration file at `{path}`")]
    #[non_exhaustive]
    InvalidFile {
        /// The path of the file that does not deserialize
        path: ConfigurationPath,

        /// The cause of the failure
        source: DeserializeConfigurationError,
    },

    /// No file exists at the caller-supplied path
    ///
    /// The variant carries no cause: that no file exists is the full
    /// diagnosis, and the underlying report of the operating system would
    /// only repeat it.
    // config[impl load.file.error.missing]
    #[error("no configuration file exists at `{path}`")]
    #[non_exhaustive]
    MissingFile {
        /// The path at which no file exists
        path: ConfigurationPath,
    },

    /// A file exists at the path, but it cannot be read
    ///
    /// The read can fail because permissions are missing, because the path
    /// points to a directory, or because the file is not valid UTF-8. The
    /// cause states the reason.
    // config[impl load.file.error.unreadable]
    #[error("failed to read the configuration file at `{path}`")]
    #[non_exhaustive]
    UnreadableFile {
        /// The path of the file that cannot be read
        path: ConfigurationPath,

        /// The cause of the failure
        source: std::io::Error,
    },
}
