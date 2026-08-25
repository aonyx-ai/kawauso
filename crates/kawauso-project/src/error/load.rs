//! The error for the loading of the configuration of a project

use thiserror::Error;

use crate::project::ConfigurationFile;
use crate::project::ProjectRoot;

/// The error returned when the configuration of a project cannot be loaded
///
/// The variants separate the failures that a caller acts on in different
/// ways. A project without a configuration file needs one, and the error
/// names the path at which the file has to go. A file that exists and cannot
/// be loaded has to be corrected, and the cause names the position, the
/// field, or the reason that the user has to act on.
///
/// A later release can add variants, and it can add fields to a variant.
/// Match with a wildcard arm, and bind the fields of a variant with `..`.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum LoadProjectConfigurationError {
    /// The project has no configuration file
    ///
    /// The search tested the location of the file in the directory of the
    /// project and found nothing there. The user can create the file at the
    /// path that the message names.
    ///
    /// The variant carries no cause: that no file exists is the full
    /// diagnosis, and the search observed it without a failed operation.
    // project[impl configuration.error.missing]
    #[error("no configuration file exists at `{path}`")]
    #[non_exhaustive]
    MissingFile {
        /// The path at which the project keeps its configuration file
        path: ConfigurationFile,
    },

    /// The configuration file of the project cannot be loaded
    ///
    /// The file exists, and the load failed while it read, parsed, or
    /// deserialized the file, or the file disappeared after the search
    /// found it. The cause names the file and the place in it that has to
    /// change.
    // project[impl configuration.error]
    #[error("failed to load the configuration of the project at `{root}`")]
    #[non_exhaustive]
    UnloadableConfiguration {
        /// The directory of the project whose configuration cannot be loaded
        root: ProjectRoot,

        /// The cause of the failure
        source: Box<dyn std::error::Error + Send + Sync>,
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
    fn load_project_configuration_error_is_send_and_sync() {
        fn assert_send_and_sync<T: Send + Sync>() {}

        assert_send_and_sync::<LoadProjectConfigurationError>();
    }
}
