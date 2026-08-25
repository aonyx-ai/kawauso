//! Errors for the search for a project

pub mod markers;

use thiserror::Error;

pub use self::markers::Markers;
use crate::search::Marker;
use crate::search::StartDirectory;

/// The error returned when a search does not produce a project
///
/// The variants separate what the user of the application has to do next. A
/// search that walked to the root of the file system and matched nothing
/// needs a marker, and the error names the start and the markers that it
/// looked for. A start that cannot be read needs a path that exists. A
/// search that never started needs an environment in which the process can
/// name its own working directory. A marker that leaves its directory is a
/// mistake in the application, which the user reports to its developers.
///
/// A later release can add variants, and it can add fields to a variant.
/// Match with a wildcard arm, and bind the fields of a variant with `..`.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DiscoverProjectError {
    /// No directory of the walk holds a marker
    ///
    /// The walk went from the start to the root of the file system, and no
    /// marker existed in any directory on the way. The user can create a
    /// marker in the directory that is the project, or run the application
    /// inside a project.
    ///
    /// The start is the resolved form of the directory at which the walk
    /// began, and the markers are in the order in which the search tested
    /// them.
    // project[impl discover.error.missing.message]
    #[error("no directory at or above `{start}` holds any of these markers: {markers}")]
    #[non_exhaustive]
    MissingProject {
        /// The directory at which the walk started
        start: StartDirectory,

        /// The markers that the search tested, in the order of the test
        markers: Markers,
    },

    /// A marker is not a relative path inside a searched directory
    ///
    /// The application named a marker that is absolute, that moves above
    /// its directory, or that names no entry at all. Such a value takes the
    /// search outside the directories that it reads. The search therefore
    /// stops before it reads the file system.
    ///
    /// The value comes from the application, and not from its user. A user
    /// who reads this message has to report it to the developers.
    ///
    /// The variant carries no cause: the value is the full diagnosis, and no
    /// other operation failed.
    // project[impl discover.markers.error.outside]
    #[error("the marker `{marker}` is not a relative path inside a searched directory")]
    #[non_exhaustive]
    OutsideMarker {
        /// The marker that is not inside the directory that it belongs to
        marker: Marker,
    },

    /// The start of the walk cannot be read
    ///
    /// The developer named a start, and the file system reports nothing at
    /// that path, or refuses to report on it. A start that does not exist is
    /// most likely a mistake in an argument of the user, and the report that
    /// no project exists would hide that mistake.
    // project[impl discover.start.error.unreadable]
    #[error("failed to read the start of the search at `{start}`")]
    #[non_exhaustive]
    UnreadableStart {
        /// The start that cannot be read, in its resolved form
        start: StartDirectory,

        /// The cause of the failure
        source: std::io::Error,
    },

    /// The process cannot name its own working directory
    ///
    /// The walk starts at the working directory, or resolves a relative
    /// start against it, and the operating system refused to report it.
    /// This happens when the directory was removed after the process
    /// started, or when the process cannot read a component of its path.
    // project[impl discover.start.error.unknown-directory]
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
    fn discover_project_error_is_send_and_sync() {
        fn assert_send_and_sync<T: Send + Sync>() {}

        assert_send_and_sync::<DiscoverProjectError>();
    }
}
