//! The error of loading a project

use std::error::Error;

use thiserror::Error as ErrorDerive;

use crate::error::DiscoverProjectError;
use crate::project::ConfigurationPath;

/// The error returned when a project cannot be loaded
///
/// The variants separate what the user of the application has to do next. A
/// search that produced no project needs a marker, or a start inside a
/// project. A configuration file that cannot be read or deserialized needs a
/// correction in the file, and the message names the path. A configuration
/// file that the crate cannot create needs a writable directory.
///
/// A project without a configuration file is no failure of this operation.
/// The project then reports no configuration, and the application decides
/// what that means.
///
/// A later release can add variants, and it can add fields to a variant.
/// Match with a wildcard arm, and bind the fields of a variant with `..`.
#[derive(Debug, ErrorDerive)]
#[non_exhaustive]
pub enum LoadProjectError {
    /// The search produced no project
    ///
    /// The walk found no marker, or it could not begin. The cause carries
    /// which of the two happened.
    ///
    /// The message of this variant is the message of its cause. A search that
    /// matched nothing names the start and the markers that it tested, and a
    /// search that never began names the reason. A caller that reports only
    /// the first line of an error therefore still tells its user what to
    /// correct.
    #[error(transparent)]
    #[non_exhaustive]
    UndiscoverableProject {
        /// The cause of the failure
        source: DiscoverProjectError,
    },

    /// The configuration file of the project cannot be created
    ///
    /// No file exists at the path, and the caller asked for the creation.
    /// Serializing the value, creating the directories above the file, or
    /// writing the file failed. The application depends on the file for the
    /// work that follows. The crate therefore reports the failure instead of
    /// a project whose configuration lives only in memory.
    ///
    /// The cause is opaque. It exists for a report that walks the chain of an
    /// error, and its type is not part of the API of this crate.
    #[error("failed to create the configuration file of the project at `{path}`")]
    #[non_exhaustive]
    UncreatableConfiguration {
        /// The path of the configuration file that cannot be created
        path: ConfigurationPath,

        /// The cause of the failure
        source: Box<dyn Error + Send + Sync>,
    },

    /// The configuration file of the project cannot be loaded
    ///
    /// A file exists at the path, and reading it, parsing it as TOML, or
    /// deserializing it into the type of the application failed. The user
    /// wrote this file and expects the application to use it, so the crate
    /// reports the failure instead of running without a configuration.
    ///
    /// The cause is opaque. It exists for a report that walks the chain of an
    /// error, and its type is not part of the API of this crate.
    #[error("failed to load the configuration file of the project at `{path}`")]
    #[non_exhaustive]
    UnloadableConfiguration {
        /// The path of the configuration file that cannot be loaded
        path: ConfigurationPath,

        /// The cause of the failure
        source: Box<dyn Error + Send + Sync>,
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
    fn load_project_error_is_send_and_sync() {
        fn assert_send_and_sync<T: Send + Sync>() {}

        assert_send_and_sync::<LoadProjectError>();
    }
}
