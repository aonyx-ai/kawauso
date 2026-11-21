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
///
/// # Examples
///
/// ```no_run
/// use kawauso_project::Project;
/// use kawauso_project::Search;
/// use kawauso_project::error::CreateDataDirectoryError;
/// use kawauso_project::project::ProjectIdentifier;
///
/// let search = Search::working_directory().marker(".git");
/// let project: Project = Project::builder()
///     .application("example".parse()?)
///     .load(&search)?;
/// let identifier: ProjectIdentifier = "my-project".parse()?;
///
/// match project.data_directory(&identifier) {
///     Ok(directory) => println!("{}", directory.get().display()),
///     Err(CreateDataDirectoryError::UnknownLocalDataDirectory { .. }) => {
///         println!("the platform has no local data directory");
///     }
///     Err(error) => return Err(error.into()),
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
    /// This happens in one of three cases. The platform has no such directory,
    /// or the crate cannot determine the home directory of the user. Otherwise
    /// the directory that it found is not an absolute path. A relative path would resolve against the working
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
    #![allow(
        clippy::missing_panics_doc,
        reason = "An assertion in a test panics by design. A `# Panics` section on every test would repeat that and give the reader no information."
    )]

    use super::*;

    /// Shows that `CreateDataDirectoryError` is Send and Sync
    ///
    /// A caller sends the error between threads and keeps it in a report that
    /// another thread reads. This test holds the error to the auto traits that
    /// make this possible. The test exists because a private field of a later
    /// version could take them away without a word from the compiler.
    #[test]
    fn create_data_directory_error_is_send_and_sync() {
        fn assert_send_and_sync<T: Send + Sync>() {}

        assert_send_and_sync::<CreateDataDirectoryError>();
    }

    #[test]
    fn create_data_directory_error_is_send_sync_and_unpin() {
        fn assert_send_sync_and_unpin<T: Send + Sync + Unpin>() {}

        assert_send_sync_and_unpin::<CreateDataDirectoryError>();
    }
}
