//! Errors for the search for a configuration file

pub mod searched_locations;

use thiserror::Error;

pub use self::searched_locations::SearchedLocations;
use crate::loader::ConfigurationPath;

/// The error returned when a search does not produce a configuration file
///
/// The variants separate what the user of the application has to do next. A
/// search that read every location of its strategy and found nothing needs a
/// new file, and the error names the locations where it can go. A location
/// that holds a directory needs that directory removed. A search that never
/// started needs an environment in which the process can name its own
/// working directory or the directory of the user.
///
/// A later release can add variants, and it can add fields to a variant.
/// Match with a wildcard arm, and bind the fields of a variant with `..`.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DiscoverConfigurationError {
    /// No location that the search read holds a configuration file
    ///
    /// The search reached the end of its locations. The user can create the
    /// file at any of them, and the one that the message names first is the
    /// one that wins.
    // config[impl discover.error.missing]
    #[error("no configuration file exists at any of these locations: {locations}")]
    #[non_exhaustive]
    MissingFile {
        /// The locations that the search read, in the order of the search
        locations: SearchedLocations,
    },

    /// A location holds a directory with the name of the configuration file
    ///
    /// The search stops at such a location instead of moving on. A directory
    /// with the name of the file is a mistake that is hard to see, and a
    /// file from a later location would hide it.
    // config[impl discover.error.directory.path]
    #[error("the path `{path}` is a directory, not a configuration file")]
    #[non_exhaustive]
    UnexpectedDirectory {
        /// The path of the directory
        path: ConfigurationPath,
    },

    /// The platform does not name the directory that holds the configuration of a user
    ///
    /// The environment of the process names that directory, and the process
    /// runs in an environment that does not, such as one without a home
    /// directory.
    ///
    /// The variant carries no cause: the environment reports no failure, it
    /// only lacks a value.
    #[error("the configuration directory of the platform is unknown")]
    #[non_exhaustive]
    UnknownConfigurationDirectory {},

    /// The process cannot name its own working directory
    ///
    /// The search starts at the working directory, and the operating system
    /// refused to report it. This happens when the directory was removed
    /// after the process started, or when the process cannot read a
    /// component of its path.
    #[error("failed to determine the working directory of the process")]
    #[non_exhaustive]
    UnknownWorkingDirectory {
        /// The cause of the failure
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every
    // test would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    // A caller sends the error between threads and keeps it in a report that
    // another thread reads. This test holds the error to the auto traits that
    // make this possible, because a private field of a later version could
    // take them away without a word from the compiler.
    #[test]
    fn discover_configuration_error_is_send_and_sync() {
        fn assert_send_and_sync<T: Send + Sync>() {}

        assert_send_and_sync::<DiscoverConfigurationError>();
    }
}
