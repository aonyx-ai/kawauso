//! The search for the project of an application
//!
//! A search describes how [`load`][load] finds a project: the
//! directory at which the walk starts, and the markers that identify the
//! project. The module holds the search and the types of its parts.
//!
//! The search is a value, and the walk happens when a project is discovered.
//! An application can therefore build the search once, keep it in a field,
//! and discover the project more than once.
//!
//! [load]: crate::Project::builder

pub mod fallback;
pub mod marker;
pub mod start_directory;
pub mod state;

use std::marker::PhantomData;

pub use self::fallback::Fallback;
pub use self::marker::Marker;
pub use self::start_directory::StartDirectory;
pub use self::state::Marked;
pub use self::state::State;
use self::state::Unmarked;

/// The search for the project of an application
///
/// A search names the directory at which the walk starts, and the markers
/// that identify the project. The walk tests the markers in the order in
/// which the developer named them, and the first directory that holds any of
/// them is the project.
///
/// The type parameter records whether the search names a marker. A search
/// without one cannot find a project, because the walk tests nothing in every
/// directory. [`load`][load] therefore takes a `Search<Marked>`, and
/// a search that never received a marker does not compile.
///
/// # Examples
///
/// A search that stops at the repository:
///
/// ```
/// use kawauso_project::Search;
///
/// let search = Search::start("src/main.rs").marker(".git");
///
/// assert_eq!(search.markers().len(), 1);
/// ```
///
/// A search with more than one marker:
///
/// ```
/// use kawauso_project::Search;
///
/// let search = Search::start("src").marker(".git").marker("Cargo.toml");
///
/// assert_eq!(search.markers().len(), 2);
/// ```
///
/// [load]: crate::Project::builder
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Search<S: State> {
    /// The directory at which the walk starts
    start: StartDirectory,

    /// The markers that the developer named
    ///
    /// The order is the order in which the developer named them, because it
    /// decides which marker the search tests first in each directory.
    markers: Vec<Marker>,

    /// What the search reports when no marker matches
    fallback: Fallback,

    /// Whether the search names a marker
    ///
    /// The field carries no data. It gives the type parameter a place in the
    /// struct, which the compiler requires.
    state: PhantomData<S>,
}

impl Search<Unmarked> {
    /// Creates a search that starts at the directory that the caller names
    ///
    /// The start can be relative, and it can hold `.` and `..` components. It
    /// resolves against the working directory of the process when the search
    /// runs, and not when the search is built. A start that names a file
    /// starts the walk at the directory that holds the file.
    ///
    /// Use this constructor for an application that takes a path from its
    /// user, and [`working_directory`][working-directory] for one that does
    /// not.
    ///
    /// The search has no marker yet, and it cannot discover a project.
    /// [`marker`][marker] adds the first one.
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_project::Search;
    ///
    /// let search = Search::start("src/main.rs");
    ///
    /// assert_eq!(
    ///     search.start_directory().get(),
    ///     std::path::Path::new("src/main.rs")
    /// );
    /// ```
    ///
    /// [marker]: Search::marker
    /// [working-directory]: Search::working_directory
    pub fn start(start: impl Into<StartDirectory>) -> Self {
        Self {
            start: start.into(),
            markers: Vec::new(),
            fallback: Fallback::Error,
            state: PhantomData,
        }
    }

    /// Creates a search that starts at the working directory of the process
    ///
    /// A user runs an application in the project, or in a directory of the
    /// project, so the working directory finds the project without an
    /// argument. Use this constructor for an application that takes no path
    /// of its own, and [`start`][start] for one that does.
    ///
    /// The working directory is read when the search runs, and not when it is
    /// built. An application can therefore build the search once and discover
    /// the project again after the working directory changed.
    ///
    /// The search has no marker yet, and it cannot discover a project.
    /// [`marker`][marker] adds the first one.
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_project::Search;
    ///
    /// let search = Search::working_directory().marker(".git");
    ///
    /// assert_eq!(search.markers().len(), 1);
    /// ```
    ///
    /// [marker]: Search::marker
    /// [start]: Search::start
    // project[impl discover.start.working-directory]
    pub fn working_directory() -> Self {
        // A relative start resolves against the working directory, and `.`
        // resolves to the working directory itself. The walk needs no case of
        // its own for this start.
        Self::start(".")
    }
}

impl<S: State> Search<S> {
    /// Adds a marker that identifies the project
    ///
    /// The value is the path of an entry relative to the project, such as
    /// `.git`, `src/main.rs`, or `.config/example.toml`. The search tests
    /// only whether an entry exists at the path, and it never reads the
    /// entry.
    ///
    /// Every call adds one marker, and the search tests the markers in the
    /// order of the calls. The first marker that matches in a directory
    /// decides which marker the project reports.
    ///
    /// A search that receives a marker can discover a project, whether or not
    /// it had one before.
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_project::Search;
    ///
    /// let search = Search::start(".").marker(".git");
    ///
    /// assert_eq!(search.markers()[0].get(), std::path::Path::new(".git"));
    /// ```
    // project[impl discover.markers.order]
    pub fn marker(self, marker: impl Into<Marker>) -> Search<Marked> {
        let mut markers = self.markers;
        markers.push(marker.into());

        Search {
            start: self.start,
            markers,
            fallback: self.fallback,
            state: PhantomData,
        }
    }

    /// Returns what the search reports when no marker matches
    pub fn fallback(&self) -> Fallback {
        self.fallback
    }

    /// Returns the markers of the search, in the order of the test
    pub fn markers(&self) -> &[Marker] {
        &self.markers
    }

    /// Reports the start directory as the project when no marker matches
    ///
    /// A search without this option returns an error when the walk reaches
    /// the root of the file system, because an application that creates files
    /// must not write them into a directory that nothing identifies as a
    /// project.
    ///
    /// Use this option for an application that also runs outside a project,
    /// with default settings. The project that the search then reports has no
    /// marker, so [`marker`][marker] returns `None` for it.
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_project::Project;
    /// use kawauso_project::Search;
    ///
    /// let directory = tempfile::tempdir()?;
    ///
    /// let search = Search::start(directory.path())
    ///     .marker(".no-such-project-marker")
    ///     .or_start();
    /// let project: Project = Project::builder().load(&search)?;
    ///
    /// assert!(project.marker().is_none());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// [marker]: crate::Project::marker
    // project[impl discover.fallback]
    pub fn or_start(mut self) -> Self {
        self.fallback = Fallback::Start;

        self
    }

    /// Returns the directory at which the walk starts
    ///
    /// The value is the path that the developer named, and not its resolved
    /// form. The search resolves it when it runs.
    pub fn start_directory(&self) -> &StartDirectory {
        &self.start
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every
    // test would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    // project[verify discover.markers.order]
    #[test]
    fn marker_with_two_markers_keeps_the_order_of_the_calls() {
        let search = Search::start(".").marker(".git").marker("src/main.rs");

        let markers: Vec<_> = search.markers().iter().map(Marker::get).collect();

        assert_eq!(
            markers,
            vec![
                std::path::Path::new(".git"),
                std::path::Path::new("src/main.rs")
            ]
        );
    }

    // A caller keeps the search in a field of a type that another thread
    // reads. This test holds the search to the auto traits that make this
    // possible, because a private field of a later version could take them
    // away without a word from the compiler.
    #[test]
    fn search_is_send_and_sync() {
        fn assert_send_and_sync<T: Send + Sync>() {}

        assert_send_and_sync::<Search<Marked>>();
    }
}
