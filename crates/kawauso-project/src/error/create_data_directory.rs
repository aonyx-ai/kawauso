//! The error of creating the data directory of a project

use thiserror::Error;

use crate::project::DataDirectory;

/// The error returned when the data directory of a project cannot be created
///
/// The variants separate what the user of the application has to do next. A
/// local data directory that the crate cannot determine needs an environment
/// with a home directory at an absolute path. A directory that cannot be
/// created needs a writable location, and the message names the path.
///
/// An application whose files are optional, such as a log, can report this
/// error as a warning and continue without the directory.
///
/// A later release can add variants, and it can add fields to a variant.
/// Match with a wildcard arm, and bind the fields of a variant with `..`.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CreateDataDirectoryError {
    /// The data directory cannot be created
    ///
    /// The crate tried to create the directory and the directories above it,
    /// and the file system refused. A file in the place of a directory, or a
    /// directory that the user cannot write, are examples.
    #[error("failed to create the data directory of the project at `{path}`")]
    #[non_exhaustive]
    UncreatableDataDirectory {
        /// The path of the data directory that cannot be created
        path: DataDirectory,

        /// The cause of the failure
        source: std::io::Error,
    },

    /// The crate cannot determine the local data directory of the user
    ///
    /// The platform has no such directory, the crate cannot determine the
    /// home directory of the user, or the directory that it found is not an
    /// absolute path. A relative path would resolve against the working
    /// directory, which can be inside the repository.
    ///
    /// The variant carries no cause: no operation failed, the crate only
    /// lacks a usable value.
    #[error("the local data directory of the platform is unknown")]
    #[non_exhaustive]
    UnknownLocalDataDirectory {},
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
    fn create_data_directory_error_is_send_and_sync() {
        fn assert_send_and_sync<T: Send + Sync>() {}

        assert_send_and_sync::<CreateDataDirectoryError>();
    }
}
