//! The project that an application runs in
//!
//! This module turns a search into a project. [`Project`] is the entry point
//! of the crate: [`discover`][discover] walks up from a directory until it
//! finds one that holds a marker of the application, and the project is that
//! directory together with the marker that identified it.
//!
//! A project has a configuration file at a conventional location, and the
//! module loads it. The reading and the deserialization are the work of the
//! crate `kawauso-config`; the project adds what that crate cannot know,
//! which file belongs to the project and whether the project has it.
//!
//! [discover]: Project::discover

pub mod configuration_file;
pub mod project_root;

use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use kawauso_config::Loader;
use serde::de::DeserializeOwned;

pub use self::configuration_file::ConfigurationFile;
pub use self::project_root::ProjectRoot;
use crate::error::DiscoverProjectError;
use crate::error::LoadProjectConfigurationError;
use crate::error::discover::Markers;
use crate::search::Fallback;
use crate::search::Marker;
use crate::search::ProjectSearch;
use crate::search::StartDirectory;

/// The project that an application runs in
///
/// A project is a directory that a marker identifies. The search walked up
/// from a start directory and ended at the first directory that holds one
/// of the markers of the application. The project keeps that directory and
/// the marker, so that a tool anchors its relative paths at a directory that
/// the search observed, and never at one that it derived from the path of a
/// file.
///
/// A project also knows the configuration file of the application, which is
/// `.config/<name>.toml` by convention. The file is a marker of the search,
/// and the search tests it before every other marker, so the project knows
/// whether the file exists without a second look at the file system.
/// [`configuration`][configuration] loads it, and
/// [`configuration_or_default`][configuration-or-default] returns the
/// default of the type when the file is absent.
///
/// A project that the search fell back to has no marker, and it has no
/// configuration file.
///
/// # Examples
///
/// ```
/// use kawauso_project::Project;
/// use kawauso_project::ProjectSearch;
///
/// let directory = tempfile::tempdir()?;
/// std::fs::create_dir(directory.path().join(".git"))?;
///
/// let search = ProjectSearch::new("example")
///     .marker(".git")
///     .start(directory.path());
/// let project = Project::discover(search)?;
///
/// assert_eq!(project.root().get(), directory.path());
/// assert!(!project.has_configuration());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// [configuration]: Project::configuration
/// [configuration-or-default]: Project::configuration_or_default
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Project {
    /// The directory of the project
    root: ProjectRoot,

    /// The marker that identified the project
    ///
    /// `None` when the search fell back to the start directory.
    marker: Option<Marker>,

    /// The relative path of the configuration file of the application
    ///
    /// The project compares it with the marker to learn whether the file
    /// exists, and it joins it onto the root to name the file.
    configuration: Marker,
}

impl Project {
    /// Finds the project of an application
    ///
    /// The walk starts at the working directory of the process, or at the
    /// directory that the search names, and goes up one directory at a time.
    /// In each directory, it tests the configuration file of the application
    /// first, and then the markers of the search in the order in which the
    /// developer named them. The first directory in which any marker exists
    /// is the project.
    ///
    /// The working directory is read when this method runs, not when the
    /// search is built.
    ///
    /// # Errors
    ///
    /// Returns [`MissingProject`][missing] when no marker exists in any
    /// directory up to the root of the file system, and the search does not
    /// fall back to the start.
    ///
    /// Returns [`OutsideMarker`][outside] when a marker is not a relative
    /// path inside a directory.
    ///
    /// Returns [`UnreadableStart`][unreadable] when the search names a start
    /// that does not exist or cannot be read.
    ///
    /// Returns [`UnknownWorkingDirectory`][unknown] when the walk needs the
    /// working directory of the process and the operating system does not
    /// report it.
    ///
    /// # Examples
    ///
    /// A search that starts at a directory of the caller:
    ///
    /// ```
    /// use kawauso_project::Project;
    /// use kawauso_project::ProjectSearch;
    ///
    /// let directory = tempfile::tempdir()?;
    /// let project_directory = directory.path().join("project");
    /// std::fs::create_dir_all(project_directory.join("src"))?;
    /// std::fs::write(project_directory.join("src").join("main.rs"), "")?;
    ///
    /// let search = ProjectSearch::new("example")
    ///     .marker("src/main.rs")
    ///     .start(project_directory.join("src"));
    /// let project = Project::discover(search)?;
    ///
    /// assert_eq!(project.root().get(), project_directory);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// A search that starts at the working directory:
    ///
    /// ```no_run
    /// use kawauso_project::Project;
    ///
    /// // Finds the first directory at or above the working directory that
    /// // holds `.config/example.toml`
    /// let project = Project::discover("example")?;
    /// # Ok::<(), kawauso_project::error::DiscoverProjectError>(())
    /// ```
    ///
    /// [missing]: DiscoverProjectError::MissingProject
    /// [outside]: DiscoverProjectError::OutsideMarker
    /// [unknown]: DiscoverProjectError::UnknownWorkingDirectory
    /// [unreadable]: DiscoverProjectError::UnreadableStart
    // project[impl discover.start.caller]
    // project[impl discover.start.working-directory]
    pub fn discover(search: impl Into<ProjectSearch>) -> Result<Self, DiscoverProjectError> {
        let search = search.into();
        let start = match search.start_directory() {
            Some(start) => Ok(start.get().to_path_buf()),
            None => std::env::current_dir(),
        };

        discover_from(start, &search)
    }

    /// Returns the directory of the project
    pub fn root(&self) -> &ProjectRoot {
        &self.root
    }

    /// Returns the marker that identified the project
    ///
    /// A project that the search fell back to has none.
    pub fn marker(&self) -> Option<&Marker> {
        self.marker.as_ref()
    }

    /// Returns the path of the configuration file of the project
    ///
    /// The path is derived from the directory of the project and the
    /// convention, whether the file exists or not.
    /// [`has_configuration`][has-configuration] says whether it does.
    ///
    /// [has-configuration]: Project::has_configuration
    pub fn configuration_file(&self) -> ConfigurationFile {
        ConfigurationFile::new(self.root.get().join(self.configuration.get()))
    }

    /// Reports whether the project has a configuration file
    ///
    /// The answer comes from the search: the configuration file is the
    /// first marker that the search tests in a directory, so the project has
    /// the file when that marker identified it. The file system is not read
    /// a second time.
    pub fn has_configuration(&self) -> bool {
        self.marker.as_ref() == Some(&self.configuration)
    }

    /// Loads the configuration file of the project into the caller's type
    ///
    /// The method derives the path of the file from the directory of the
    /// project and the convention, reads the file, parses it as TOML, and
    /// deserializes it into `T`. The type must implement the
    /// [`Deserialize`][deserialize] trait of serde, which its derive macro
    /// generates.
    ///
    /// Use this method for a tool whose configuration is required. A tool
    /// that runs without a configuration file uses
    /// [`configuration_or_default`][configuration-or-default].
    ///
    /// # Errors
    ///
    /// Returns [`MissingFile`][missing] when the project has no
    /// configuration file.
    ///
    /// Returns [`UnloadableConfiguration`][unloadable] when the file cannot
    /// be read, when its contents are not valid TOML, or when the document
    /// does not match `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_project::Project;
    /// use kawauso_project::ProjectSearch;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct Configuration {
    ///     port: u16,
    /// }
    ///
    /// let directory = tempfile::tempdir()?;
    /// std::fs::create_dir(directory.path().join(".config"))?;
    /// std::fs::write(directory.path().join(".config/example.toml"), "port = 8080")?;
    ///
    /// let project = Project::discover(ProjectSearch::new("example").start(directory.path()))?;
    /// let configuration: Configuration = project.configuration()?;
    ///
    /// assert_eq!(configuration.port, 8080);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// [configuration-or-default]: Project::configuration_or_default
    /// [deserialize]: https://docs.rs/serde/latest/serde/trait.Deserialize.html
    /// [missing]: LoadProjectConfigurationError::MissingFile
    /// [unloadable]: LoadProjectConfigurationError::UnloadableConfiguration
    // project[impl configuration.error.missing]
    pub fn configuration<T>(&self) -> Result<T, LoadProjectConfigurationError>
    where
        T: DeserializeOwned,
    {
        if !self.has_configuration() {
            return Err(LoadProjectConfigurationError::MissingFile {
                path: self.configuration_file(),
            });
        }

        self.load()
    }

    /// Loads the configuration file of the project, or returns the default
    ///
    /// The method behaves like [`configuration`][configuration] when the
    /// project has a configuration file. When it has none, the method
    /// returns the default of `T` instead of an error, so a project without
    /// a configuration file is a supported target of the tool.
    ///
    /// The default replaces an absent file only. A file that exists and
    /// cannot be loaded is a mistake that the user has to correct, and the
    /// method reports it.
    ///
    /// # Errors
    ///
    /// Returns [`UnloadableConfiguration`][unloadable] when the file exists
    /// and cannot be read, when its contents are not valid TOML, or when the
    /// document does not match `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_project::Project;
    /// use kawauso_project::ProjectSearch;
    /// use serde::Deserialize;
    ///
    /// #[derive(Default, Deserialize)]
    /// struct Configuration {
    ///     #[serde(default)]
    ///     ignore: Vec<String>,
    /// }
    ///
    /// let directory = tempfile::tempdir()?;
    /// std::fs::create_dir(directory.path().join(".git"))?;
    ///
    /// let search = ProjectSearch::new("example")
    ///     .marker(".git")
    ///     .start(directory.path());
    /// let project = Project::discover(search)?;
    /// let configuration: Configuration = project.configuration_or_default()?;
    ///
    /// assert!(configuration.ignore.is_empty());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// [configuration]: Project::configuration
    /// [unloadable]: LoadProjectConfigurationError::UnloadableConfiguration
    // project[impl configuration.default]
    // project[impl configuration.default.error]
    pub fn configuration_or_default<T>(&self) -> Result<T, LoadProjectConfigurationError>
    where
        T: DeserializeOwned + Default,
    {
        if !self.has_configuration() {
            return Ok(T::default());
        }

        self.load()
    }

    /// Loads the configuration file that the search found
    ///
    /// The reading and the deserialization are the work of `kawauso-config`,
    /// so that a failure in the file is reported in the same words as for
    /// every other configuration file. The failure gains the directory of
    /// the project, and its cause names the file.
    ///
    /// # Errors
    ///
    /// Returns an error when the file cannot be read, when its contents are
    /// not valid TOML, or when the document does not match the caller's type.
    // project[impl configuration.load]
    fn load<T>(&self) -> Result<T, LoadProjectConfigurationError>
    where
        T: DeserializeOwned,
    {
        Loader::path(self.configuration_file().get())
            .load()
            .map_err(
                |source| LoadProjectConfigurationError::UnloadableConfiguration {
                    root: self.root.clone(),
                    source: Box::new(source),
                },
            )
    }
}

/// Finds the project of an application from a start directory
///
/// The start arrives as a result, and not as a path, because the operating
/// system can refuse to report the working directory. The refusal becomes a
/// failure of the search here, next to the walk that needs the directory.
///
/// A marker that leaves its directory fails the search before it reads the
/// file system. Such a value comes from the application, not from the user,
/// and it is the same mistake in every environment. The report is therefore
/// the same in every environment as well.
///
/// The walk is the outer loop, and the markers are the inner loop: every
/// marker is tested in one directory before any marker is tested in the
/// directory above it. A project therefore wins over the directory that
/// contains it, whichever of its markers identifies it.
///
/// # Errors
///
/// Returns an error when a marker is not a relative path inside its
/// directory, when the working directory is unknown, when the start cannot
/// be read, or when no marker exists in any directory of the walk and the
/// search does not fall back to the start.
// project[impl discover.error.missing]
// project[impl discover.fallback]
// project[impl discover.markers]
// project[impl discover.markers.error.outside]
// project[impl discover.markers.walk]
// project[impl discover.order]
// project[impl discover.precedence]
// project[impl discover.result]
// project[impl discover.start.error.unknown-directory]
// project[impl discover.walk]
fn discover_from(
    start: std::io::Result<PathBuf>,
    search: &ProjectSearch,
) -> Result<Project, DiscoverProjectError> {
    let markers = markers_of(search);

    if let Some(marker) = markers.iter().find(|marker| !is_inside(marker)) {
        return Err(DiscoverProjectError::OutsideMarker {
            marker: marker.clone(),
        });
    }

    let start = start.map_err(|source| DiscoverProjectError::UnknownWorkingDirectory { source })?;
    let start = resolve(start)?;

    for directory in start.ancestors() {
        for marker in &markers {
            if exists(&directory.join(marker.get())) {
                return Ok(Project {
                    root: ProjectRoot::new(directory.to_path_buf()),
                    marker: Some(marker.clone()),
                    configuration: search.configuration_marker().clone(),
                });
            }
        }
    }

    match search.fallback() {
        Fallback::Start => Ok(Project {
            root: ProjectRoot::new(start),
            marker: None,
            configuration: search.configuration_marker().clone(),
        }),
        Fallback::Error => Err(DiscoverProjectError::MissingProject {
            start: StartDirectory::new(start),
            markers: Markers::new(markers),
        }),
    }
}

/// Returns the markers of a search in the order of the test
///
/// The configuration file of the application comes first, so that the
/// project can tell from the marker that identified it whether the file
/// exists. The markers that the developer named follow in the order in
/// which they were named, which is the order in which they win.
// project[impl configuration.marker]
// project[impl discover.markers.order]
fn markers_of(search: &ProjectSearch) -> Vec<Marker> {
    std::iter::once(search.configuration_marker().clone())
        .chain(search.markers().iter().cloned())
        .collect()
}

/// Resolves the start of the walk to an absolute directory
///
/// A relative start resolves against the working directory of the process.
/// The `.` and `..` components go, because the walk goes up one component
/// at a time, and a `..` component would take it through a directory that
/// the caller never named. Symbolic links stay, so the walk sees the tree
/// that the caller named, and the paths that it reports are paths that the
/// caller recognizes.
///
/// A start that names a file yields the directory that holds the file,
/// because the project that governs a file is the project of that directory.
///
/// # Errors
///
/// Returns an error when the working directory is unknown, or when the
/// start does not exist or cannot be read.
// project[impl discover.start.absolute]
// project[impl discover.start.file]
// project[impl discover.start.error.unreadable]
fn resolve(start: PathBuf) -> Result<PathBuf, DiscoverProjectError> {
    // An empty path is the working directory to every other operation of
    // the standard library, and the resolution treats it the same way.
    let start = if start.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        start
    };

    let absolute = std::path::absolute(&start)
        .map_err(|source| DiscoverProjectError::UnknownWorkingDirectory { source })?;
    let start = normalize(&absolute);

    let metadata =
        std::fs::metadata(&start).map_err(|source| DiscoverProjectError::UnreadableStart {
            start: StartDirectory::new(start.clone()),
            source,
        })?;

    if metadata.is_dir() {
        return Ok(start);
    }

    let directory = start
        .parent()
        .map_or_else(|| start.clone(), Path::to_path_buf);

    Ok(directory)
}

/// Removes the `.` and `..` components of an absolute path
///
/// The removal is lexical: a `..` component removes the component before
/// it, whether or not that component is a symbolic link. This is how a
/// shell interprets a path that the user typed, and the result is a path
/// that the user recognizes.
fn normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::ParentDir => {
                normalized.pop();
            }
            Component::CurDir => {}
            component => normalized.push(component),
        }
    }

    normalized
}

/// Reports whether a marker names an entry inside another directory
///
/// A path that joins onto a directory can leave it again. An absolute path
/// replaces the directory, and a `..` component moves above it. Either one
/// takes the search to a place that the walk never reaches, and an absolute
/// path takes every directory of the walk to the same place.
///
/// A path that names no entry at all, such as an empty path or `.`, is no
/// marker either: every directory holds it, so the start would always be
/// the project.
fn is_inside(marker: &Marker) -> bool {
    let mut names_an_entry = false;

    for component in marker.get().components() {
        match component {
            Component::Normal(_) => names_an_entry = true,
            Component::CurDir => {}
            Component::ParentDir | Component::Prefix(_) | Component::RootDir => return false,
        }
    }

    names_an_entry
}

/// Reports whether an entry exists at a path
///
/// Any entry counts: a file, a directory, or a symbolic link to one of them.
/// A `.git` entry, for example, is a directory in a repository and a file in
/// a worktree, and both identify the repository. A symbolic link whose
/// target does not exist counts as nothing, because the file system reports
/// nothing about the target.
fn exists(path: &Path) -> bool {
    std::fs::metadata(path).is_ok()
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every
    // test would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use std::io::ErrorKind;

    use super::*;

    /// The name of the application whose project the tests search for
    const APPLICATION: &str = "kawauso";

    /// Creates a directory in another directory and returns its path
    fn child_of(parent: &Path) -> PathBuf {
        let child = parent.join("project");
        std::fs::create_dir(&child).unwrap();

        child
    }

    /// Creates an entry at a relative path inside a directory
    ///
    /// The directories on the way are created as well. The entry is an empty
    /// file, because the search only tests whether the entry exists.
    fn entry_in(directory: &Path, entry: &str) {
        let path = directory.join(entry);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "").unwrap();
    }

    /// Returns a name that no other entry on this machine has
    ///
    /// The operating system gives a temporary directory a unique name, and
    /// the tests borrow it for an application and for markers that must not
    /// exist anywhere on the walk, which passes through directories that the
    /// tests do not own. The directory itself is an entry of its parent, so
    /// a marker adds a suffix to the name instead of using it as it is.
    fn unique_name(directory: &Path) -> String {
        directory.file_name().unwrap().to_str().unwrap().to_owned()
    }

    // The marker that identifies the project can be a directory, such as
    // `.git` in a repository, and not only a file.
    // project[verify discover.markers]
    #[test]
    fn discover_from_with_a_directory_as_the_marker_returns_the_directory() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join(".git")).unwrap();
        let search = ProjectSearch::new(APPLICATION).marker(".git");

        let project = discover_from(Ok(root.path().to_path_buf()), &search).unwrap();

        assert_eq!(project.root().get(), root.path());
    }

    // The start names a file, and no marker exists anywhere on the walk, so
    // the fallback reports where the walk began: the directory that holds
    // the file, and not the file itself.
    // project[verify discover.start.file]
    #[test]
    fn discover_from_with_a_file_as_the_start_starts_at_its_directory() {
        let root = tempfile::tempdir().unwrap();
        let name = unique_name(root.path());
        entry_in(root.path(), "src/main.rs");
        let search = ProjectSearch::new(name.as_str())
            .marker(format!("{name}.missing"))
            .or_start();

        let project = discover_from(Ok(root.path().join("src/main.rs")), &search).unwrap();

        assert_eq!(project.root().get(), root.path().join("src"));
    }

    // project[verify discover.markers.error.outside]
    #[test]
    fn discover_from_with_a_marker_above_the_directory_returns_an_error() {
        let root = tempfile::tempdir().unwrap();
        let search = ProjectSearch::new(APPLICATION).marker("../.git");

        let error = discover_from(Ok(root.path().to_path_buf()), &search).unwrap_err();

        assert!(matches!(error, DiscoverProjectError::OutsideMarker { .. }));
    }

    // project[verify discover.walk]
    #[test]
    fn discover_from_with_a_marker_in_an_ancestor_returns_the_ancestor() {
        let root = tempfile::tempdir().unwrap();
        let start = child_of(root.path());
        entry_in(root.path(), ".git");
        let search = ProjectSearch::new(APPLICATION).marker(".git");

        let project = discover_from(Ok(start), &search).unwrap();

        assert_eq!(project.root().get(), root.path());
    }

    // The search reports the marker as well as the directory, so that a tool
    // can tell which entry identified the project.
    // project[verify discover.result]
    #[test]
    fn discover_from_with_a_marker_in_an_ancestor_returns_the_marker() {
        let root = tempfile::tempdir().unwrap();
        let start = child_of(root.path());
        entry_in(root.path(), ".git");
        let search = ProjectSearch::new(APPLICATION).marker(".git");

        let project = discover_from(Ok(start), &search).unwrap();

        assert_eq!(project.marker(), Some(&Marker::from(".git")));
    }

    // project[verify discover.order]
    #[test]
    fn discover_from_with_a_marker_in_the_start_and_an_ancestor_returns_the_start() {
        let root = tempfile::tempdir().unwrap();
        let start = child_of(root.path());
        entry_in(root.path(), ".git");
        entry_in(&start, ".git");
        let search = ProjectSearch::new(APPLICATION).marker(".git");

        let project = discover_from(Ok(start.clone()), &search).unwrap();

        assert_eq!(project.root().get(), start);
    }

    // project[verify discover.markers.error.outside]
    #[test]
    fn discover_from_with_a_marker_that_names_no_entry_returns_an_error() {
        let root = tempfile::tempdir().unwrap();
        let search = ProjectSearch::new(APPLICATION).marker(".");

        let error = discover_from(Ok(root.path().to_path_buf()), &search).unwrap_err();

        assert!(matches!(error, DiscoverProjectError::OutsideMarker { .. }));
    }

    // project[verify discover.start.error.unreadable]
    #[test]
    fn discover_from_with_a_start_that_does_not_exist_returns_an_error() {
        let root = tempfile::tempdir().unwrap();
        let search = ProjectSearch::new(APPLICATION).marker(".git");

        let error = discover_from(Ok(root.path().join("missing")), &search).unwrap_err();

        assert!(matches!(
            error,
            DiscoverProjectError::UnreadableStart { .. }
        ));
    }

    // project[verify discover.markers.error.outside]
    #[test]
    fn discover_from_with_an_absolute_marker_returns_an_error() {
        let root = tempfile::tempdir().unwrap();
        let search = ProjectSearch::new(APPLICATION).marker(root.path().to_path_buf());

        let error = discover_from(Ok(root.path().to_path_buf()), &search).unwrap_err();

        assert!(matches!(error, DiscoverProjectError::OutsideMarker { .. }));
    }

    // project[verify discover.start.absolute]
    #[test]
    fn discover_from_with_dot_components_in_the_start_removes_them() {
        let root = tempfile::tempdir().unwrap();
        let start = child_of(root.path());
        entry_in(&start, ".git");
        let search = ProjectSearch::new(APPLICATION).marker(".git");

        let project = discover_from(Ok(root.path().join("project/../project/.")), &search).unwrap();

        assert_eq!(project.root().get(), start);
    }

    // A marker exists in the start and in its ancestor. The search ends at
    // the start and never reads the ancestor.
    // project[verify discover.precedence]
    #[test]
    fn discover_from_with_markers_in_the_start_and_an_ancestor_reports_the_start() {
        let root = tempfile::tempdir().unwrap();
        let start = child_of(root.path());
        entry_in(root.path(), ".git");
        entry_in(&start, ".git");
        let search = ProjectSearch::new(APPLICATION).marker(".git");

        let project = discover_from(Ok(start.clone()), &search).unwrap();

        assert_ne!(project.root().get(), root.path());
    }

    // The configuration file is tested before every marker that the
    // developer named, whatever the order of the calls.
    // project[verify configuration.marker]
    // project[verify discover.markers.order]
    #[test]
    fn discover_from_with_the_configuration_file_and_a_marker_returns_the_file() {
        let root = tempfile::tempdir().unwrap();
        entry_in(root.path(), ".config/kawauso.toml");
        entry_in(root.path(), ".git");
        let search = ProjectSearch::new(APPLICATION).marker(".git");

        let project = discover_from(Ok(root.path().to_path_buf()), &search).unwrap();

        assert_eq!(
            project.marker(),
            Some(&Marker::from(".config/kawauso.toml"))
        );
    }

    // The second marker exists in the start, and the first marker exists in
    // the ancestor. The start wins, because every marker of one directory is
    // tested before any marker of the directory above it.
    // project[verify discover.markers.walk]
    #[test]
    fn discover_from_with_the_second_marker_in_the_start_returns_the_start() {
        let root = tempfile::tempdir().unwrap();
        let start = child_of(root.path());
        entry_in(root.path(), "src/main.rs");
        entry_in(&start, ".git");
        let search = ProjectSearch::new(APPLICATION)
            .marker("src/main.rs")
            .marker(".git");

        let project = discover_from(Ok(start.clone()), &search).unwrap();

        assert_eq!(project.root().get(), start);
    }

    // Both markers exist in the start, and the one that the developer named
    // first wins.
    // project[verify discover.markers.order]
    #[test]
    fn discover_from_with_two_markers_in_the_start_returns_the_first() {
        let root = tempfile::tempdir().unwrap();
        entry_in(root.path(), "src/main.rs");
        entry_in(root.path(), ".git");
        let search = ProjectSearch::new(APPLICATION)
            .marker("src/main.rs")
            .marker(".git");

        let project = discover_from(Ok(root.path().to_path_buf()), &search).unwrap();

        assert_eq!(project.marker(), Some(&Marker::from("src/main.rs")));
    }

    // project[verify discover.fallback]
    #[test]
    fn discover_from_without_a_match_and_the_fallback_returns_no_marker() {
        let root = tempfile::tempdir().unwrap();
        let name = unique_name(root.path());
        let search = ProjectSearch::new(name.as_str())
            .marker(format!("{name}.missing"))
            .or_start();

        let project = discover_from(Ok(root.path().to_path_buf()), &search).unwrap();

        assert_eq!(project.marker(), None);
    }

    // project[verify discover.fallback]
    #[test]
    fn discover_from_without_a_match_and_the_fallback_returns_the_start() {
        let root = tempfile::tempdir().unwrap();
        let name = unique_name(root.path());
        let search = ProjectSearch::new(name.as_str())
            .marker(format!("{name}.missing"))
            .or_start();

        let project = discover_from(Ok(root.path().to_path_buf()), &search).unwrap();

        assert_eq!(project.root().get(), root.path());
    }

    // project[verify discover.error.missing.message]
    #[test]
    fn discover_from_without_a_match_names_the_start_and_the_markers() {
        let root = tempfile::tempdir().unwrap();
        let name = unique_name(root.path());
        let search = ProjectSearch::new(name.as_str()).marker(format!("{name}.missing"));

        let error = discover_from(Ok(root.path().to_path_buf()), &search).unwrap_err();

        assert_eq!(
            error.to_string(),
            format!(
                "no directory at or above `{}` holds any of these markers: `{}`, `{name}.missing`",
                root.path().display(),
                Path::new(".config").join(format!("{name}.toml")).display(),
            )
        );
    }

    // project[verify discover.error.missing]
    #[test]
    fn discover_from_without_a_match_returns_an_error() {
        let root = tempfile::tempdir().unwrap();
        let name = unique_name(root.path());
        let search = ProjectSearch::new(name.as_str()).marker(format!("{name}.missing"));

        let error = discover_from(Ok(root.path().to_path_buf()), &search).unwrap_err();

        assert!(matches!(error, DiscoverProjectError::MissingProject { .. }));
    }

    // project[verify discover.start.error.unknown-directory]
    #[test]
    fn discover_from_without_a_working_directory_returns_an_error() {
        let failure = std::io::Error::from(ErrorKind::NotFound);
        let search = ProjectSearch::new(APPLICATION).marker(".git");

        let error = discover_from(Err(failure), &search).unwrap_err();

        assert!(matches!(
            error,
            DiscoverProjectError::UnknownWorkingDirectory { .. }
        ));
    }

    // A symbolic link that points nowhere is no entry, because the file
    // system reports nothing about its target.
    #[cfg(unix)]
    #[test]
    fn exists_with_a_dangling_symbolic_link_returns_false() {
        let root = tempfile::tempdir().unwrap();
        let link = root.path().join(".git");
        std::os::unix::fs::symlink(root.path().join("missing"), &link).unwrap();

        let result = exists(&link);

        assert!(!result);
    }

    // The root of the file system has no parent, and a `..` there must not
    // panic or produce a path below the root.
    #[test]
    fn normalize_with_a_parent_component_at_the_root_stays_at_the_root() {
        let root = Path::new("/").join("..").join("otter");

        let normalized = normalize(&root);

        assert_eq!(normalized, Path::new("/").join("otter"));
    }
}
