//! The search for the project of an application
//!
//! A search describes how [`discover`][discover] finds a project: the
//! markers that identify it, the directory at which the walk starts, and
//! what the search reports when no marker matches. The module holds the
//! search and the types of its options.
//!
//! The search is a value, and the walk happens when a project is
//! discovered. An application can therefore build the search once, keep it
//! in a field, and discover the project more than once.
//!
//! [discover]: crate::Project::discover

pub mod application_name;
pub mod fallback;
pub mod marker;
pub mod start_directory;

use std::path::PathBuf;

pub use self::application_name::ApplicationName;
pub use self::fallback::Fallback;
pub use self::marker::Marker;
pub use self::start_directory::StartDirectory;

/// The search for the project of an application
///
/// The search needs the name of an application, from which it derives the
/// configuration file of the application, `.config/<name>.toml`. That file
/// is the first marker of the search. The developer adds the other markers
/// that identify a project of their tool, such as `.git` or `src/main.rs`,
/// and the search tests them in the order in which the developer named
/// them.
///
/// The walk starts at the working directory of the process, or at the
/// directory that the developer names. When no marker matches in any
/// directory up to the root of the file system, the search reports an
/// error, or the start directory when the developer opted in to that.
///
/// A name alone converts into a search. [`discover`][discover] therefore
/// accepts `"example"` as well as a value of this type.
///
/// # Examples
///
/// A search that stops at the repository:
///
/// ```
/// use kawauso_project::ProjectSearch;
///
/// let search = ProjectSearch::new("example").marker(".git");
///
/// assert_eq!(search.markers().len(), 1);
/// ```
///
/// A search that starts at the path that the user named, and that treats
/// the start as the project outside any repository:
///
/// ```
/// use kawauso_project::ProjectSearch;
/// use kawauso_project::search::Fallback;
///
/// let search = ProjectSearch::new("example")
///     .marker(".git")
///     .start("src/main.rs")
///     .or_start();
///
/// assert_eq!(search.fallback(), Fallback::Start);
/// ```
///
/// [discover]: crate::Project::discover
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ProjectSearch {
    /// The name of the application whose project the search finds
    application: ApplicationName,

    /// The relative path of the configuration file of the application
    ///
    /// The file is a marker of the search, and it is tested before the
    /// markers that the developer named.
    configuration: Marker,

    /// The markers that the developer named
    ///
    /// The order is the order in which the developer named them, because it
    /// decides which marker the search tests first in each directory.
    markers: Vec<Marker>,

    /// The directory at which the walk starts
    ///
    /// The working directory of the process, when the developer named none.
    start: Option<StartDirectory>,

    /// What the search reports when no marker matches
    fallback: Fallback,
}

impl ProjectSearch {
    /// Creates a search for the project of an application
    ///
    /// The configuration file of the application is the file with the name
    /// of the application and the extension `.toml` in the subdirectory
    /// `.config` of the project. It is the first marker of the search.
    /// [`marker`][marker] adds the markers that the developer names, and
    /// [`configuration_file`][configuration-file] replaces the location of
    /// the configuration file for a tool whose host dictates another one.
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_project::ProjectSearch;
    ///
    /// let search = ProjectSearch::new("example");
    ///
    /// assert_eq!(
    ///     search.configuration_marker().get(),
    ///     std::path::Path::new(".config/example.toml")
    /// );
    /// ```
    ///
    /// [configuration-file]: ProjectSearch::configuration_file
    /// [marker]: ProjectSearch::marker
    // project[impl configuration.location]
    pub fn new(application: impl Into<ApplicationName>) -> Self {
        let application = application.into();
        let configuration =
            Marker::new(PathBuf::from(".config").join(format!("{application}.toml")));

        Self {
            application,
            configuration,
            markers: Vec::new(),
            start: None,
            fallback: Fallback::Error,
        }
    }

    /// Replaces the location of the configuration file
    ///
    /// The value is the path of the file relative to the project, such as
    /// `.github/example.toml` for a tool whose host reads `.github`. It
    /// replaces the conventional location, so the search no longer tests
    /// `.config/<name>.toml`, and the project loads its configuration from
    /// the new location.
    ///
    /// The value must be a relative path that stays inside the project. A
    /// value that is absolute, or that leaves the project, fails the search
    /// and not this call.
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_project::ProjectSearch;
    ///
    /// let search = ProjectSearch::new("example").configuration_file(".github/example.toml");
    ///
    /// assert_eq!(
    ///     search.configuration_marker().get(),
    ///     std::path::Path::new(".github/example.toml")
    /// );
    /// ```
    // project[impl configuration.location.custom]
    #[must_use]
    pub fn configuration_file(mut self, marker: impl Into<Marker>) -> Self {
        self.configuration = marker.into();

        self
    }

    /// Adds a marker that identifies a project
    ///
    /// A marker is an entry at a relative path inside the project, such as
    /// `.git` or `src/main.rs`. The search tests it in each directory of the
    /// walk, after the configuration file and after the markers that the
    /// developer named before it. The first directory in which any marker
    /// exists is the project.
    ///
    /// A marker such as `.git` ends the walk at the repository: a project
    /// without a configuration file is still a project, and an entry above
    /// the repository is never read.
    ///
    /// The value must be a relative path that stays inside its directory. A
    /// value that is absolute, or that leaves the directory, fails the search
    /// and not this call. A value that the application computes therefore
    /// cannot make a constructor fail.
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_project::ProjectSearch;
    ///
    /// let search = ProjectSearch::new("example")
    ///     .marker("src/main.rs")
    ///     .marker(".git");
    ///
    /// assert_eq!(search.markers().len(), 2);
    /// ```
    #[must_use]
    pub fn marker(mut self, marker: impl Into<Marker>) -> Self {
        self.markers.push(marker.into());

        self
    }

    /// Names the directory at which the walk starts
    ///
    /// The walk starts at the working directory of the process when the
    /// developer names no start. A tool that takes a path from its user
    /// names that path instead, so that the search finds the project that
    /// governs the path, wherever the user runs the tool.
    ///
    /// The value is stored as given. A relative path resolves against the
    /// working directory when the search runs, and a path that names a file
    /// starts the walk at the directory that holds the file.
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_project::ProjectSearch;
    ///
    /// let search = ProjectSearch::new("example").start("crates/example");
    ///
    /// assert!(search.start_directory().is_some());
    /// ```
    #[must_use]
    pub fn start(mut self, start: impl Into<StartDirectory>) -> Self {
        self.start = Some(start.into());

        self
    }

    /// Treats the start directory as the project when no marker matches
    ///
    /// A walk that reaches the root of the file system without a match has
    /// found no project, and the search reports an error. Some tools run
    /// outside any project as well, with the default of their
    /// configuration. For them, this call makes the start directory the
    /// project. Such a project has no marker, and it has no configuration
    /// file.
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_project::ProjectSearch;
    /// use kawauso_project::search::Fallback;
    ///
    /// let search = ProjectSearch::new("example").or_start();
    ///
    /// assert_eq!(search.fallback(), Fallback::Start);
    /// ```
    #[must_use]
    pub fn or_start(mut self) -> Self {
        self.fallback = Fallback::Start;

        self
    }

    /// Returns the name of the application whose project the search finds
    pub fn application(&self) -> &ApplicationName {
        &self.application
    }

    /// Returns the relative path of the configuration file of the application
    pub fn configuration_marker(&self) -> &Marker {
        &self.configuration
    }

    /// Returns the markers in the order in which the developer named them
    ///
    /// The configuration file of the application is not among them. It is
    /// always the first marker of the search.
    pub fn markers(&self) -> &[Marker] {
        &self.markers
    }

    /// Returns the directory at which the walk starts, if the developer named one
    pub fn start_directory(&self) -> Option<&StartDirectory> {
        self.start.as_ref()
    }

    /// Returns what the search reports when no marker matches
    pub fn fallback(&self) -> Fallback {
        self.fallback
    }
}

/// Creates a search from the name of an application
impl From<ApplicationName> for ProjectSearch {
    fn from(application: ApplicationName) -> Self {
        Self::new(application)
    }
}

/// Creates a search from the name of an application
impl From<&str> for ProjectSearch {
    fn from(application: &str) -> Self {
        Self::new(application)
    }
}

/// Creates a search from the name of an application
impl From<String> for ProjectSearch {
    fn from(application: String) -> Self {
        Self::new(application)
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every
    // test would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    // An application that already holds its name in the newtype must not have
    // to take the string out of it again.
    #[test]
    fn from_application_name_keeps_the_name() {
        let search = ProjectSearch::from(ApplicationName::new("example"));

        assert_eq!(search.application().get(), "example");
    }

    // A name that the application builds at runtime arrives as a `String`,
    // and the search takes it without a borrow.
    #[test]
    fn from_string_keeps_the_name() {
        let search = ProjectSearch::from(String::from("example"));

        assert_eq!(search.application().get(), "example");
    }

    // The order of the calls is the order in which the markers are tested, so
    // the type must not sort them or remove a repeat.
    #[test]
    fn marker_keeps_the_order_of_the_calls() {
        let search = ProjectSearch::new("example")
            .marker("src/main.rs")
            .marker(".git");

        assert_eq!(
            search.markers(),
            vec![Marker::from("src/main.rs"), Marker::from(".git")]
        );
    }

    // A search that names no fallback reports an error, because a tool that
    // generates files must not write them into a directory that nothing marks
    // as a project.
    #[test]
    fn new_reports_an_error_without_a_match() {
        let search = ProjectSearch::new("example");

        assert_eq!(search.fallback(), Fallback::Error);
    }
}
