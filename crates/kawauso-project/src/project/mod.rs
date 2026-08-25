//! The project that an application runs in
//!
//! A project is a directory that a marker identifies. This module holds the
//! project and the walk that finds it.

pub mod project_root;

use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

pub use self::project_root::ProjectRoot;
use crate::error::DiscoverProjectError;
use crate::error::discover::Markers;
use crate::search::Marker;
use crate::search::Search;
use crate::search::StartDirectory;
use crate::search::state::Marked;

/// The project that an application runs in
///
/// A project is the directory in which a marker of the search matched. An
/// application anchors the relative paths of its resources at this directory,
/// and it writes the files that it creates there.
///
/// The search observed the directory during the walk. No application derives
/// it from the path of a file, because such a calculation repeats the
/// convention of the search and breaks when the convention changes.
///
/// # Examples
///
/// ```
/// use kawauso_project::Project;
/// use kawauso_project::Search;
///
/// let directory = tempfile::tempdir()?;
/// let root = directory.path().join("project");
/// std::fs::create_dir_all(root.join("src"))?;
/// std::fs::write(root.join("Cargo.toml"), "")?;
///
/// let search = Search::start(root.join("src")).marker("Cargo.toml");
/// let project = Project::discover(&search)?;
///
/// assert_eq!(project.root().get(), root);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Project {
    /// The directory of the project
    root: ProjectRoot,

    /// The marker that identified the project
    marker: Marker,
}

impl Project {
    /// Finds the project of an application
    ///
    /// The walk starts at the directory that the search names and goes up one
    /// directory at a time, up to the root of the file system. In each
    /// directory it tests the markers of the search, in the order in which
    /// the developer named them. The first directory that holds any marker is
    /// the project.
    ///
    /// A relative start resolves against the working directory of the
    /// process. The resolution happens when this method runs, and not when
    /// the search is built.
    ///
    /// The search must name at least one marker. A search without one does
    /// not compile:
    ///
    /// ```compile_fail
    /// use kawauso_project::Project;
    /// use kawauso_project::Search;
    ///
    /// let search = Search::start("src");
    /// let project = Project::discover(&search);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`MissingProject`][missing] when no marker exists in any
    /// directory up to the root of the file system.
    ///
    /// Returns [`OutsideMarker`][outside] when a marker is not a relative
    /// path inside a directory.
    ///
    /// Returns [`UnreadableStart`][unreadable] when the start does not exist
    /// or cannot be read.
    ///
    /// Returns [`UnknownWorkingDirectory`][unknown] when the start is
    /// relative and the operating system does not report the working
    /// directory of the process.
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_project::Project;
    /// use kawauso_project::Search;
    ///
    /// let directory = tempfile::tempdir()?;
    /// let root = directory.path().join("repository");
    /// std::fs::create_dir_all(root.join("crates").join("example"))?;
    /// std::fs::create_dir(root.join(".git"))?;
    ///
    /// let search = Search::start(root.join("crates").join("example")).marker(".git");
    /// let project = Project::discover(&search)?;
    ///
    /// assert_eq!(project.marker().get(), std::path::Path::new(".git"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// [missing]: DiscoverProjectError::MissingProject
    /// [outside]: DiscoverProjectError::OutsideMarker
    /// [unknown]: DiscoverProjectError::UnknownWorkingDirectory
    /// [unreadable]: DiscoverProjectError::UnreadableStart
    // project[impl discover.markers.required]
    // project[impl discover.start.caller]
    // project[verify discover.markers.required]
    pub fn discover(search: &Search<Marked>) -> Result<Self, DiscoverProjectError> {
        let start = resolve(search.start_directory().get(), std::env::current_dir())?;

        walk(&start, search.markers())
    }

    /// Returns the marker that identified the project
    pub fn marker(&self) -> &Marker {
        &self.marker
    }

    /// Returns the directory of the project
    pub fn root(&self) -> &ProjectRoot {
        &self.root
    }
}

/// Reports whether an entry exists at a path
///
/// Any entry counts: a file, a directory, or a symbolic link to one of them.
/// A `.git` entry, for example, is a directory in a repository and a file in
/// a worktree, and both identify the repository. A symbolic link whose target
/// does not exist counts as nothing, because the file system reports nothing
/// about the target.
fn exists(path: &Path) -> bool {
    std::fs::metadata(path).is_ok()
}

/// Reports whether a marker names an entry inside another directory
///
/// A path that joins onto a directory can leave it again. An absolute path
/// replaces the directory, and a `..` component moves above it. Either one
/// takes the search to a place that the walk never reaches, and an absolute
/// path takes every directory of the walk to the same place.
///
/// A path that names no entry at all, such as an empty path or `.`, is no
/// marker either: every directory holds it, so the start would always be the
/// project.
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

/// Removes the `.` and `..` components of an absolute path
///
/// The removal is lexical: a `..` component removes the component before it,
/// whether or not that component is a symbolic link. This is how a shell
/// interprets a path that the user typed, and the result is a path that the
/// user recognizes.
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

/// Resolves the start of the walk to an absolute directory
///
/// A relative start resolves against the working directory of the process,
/// which the caller supplies, so that a test can name one. The `.` and `..`
/// components go, because the walk goes up one component at a time, and a
/// `..` component would take it through a directory that the caller never
/// named. Symbolic links stay, so the walk sees the tree that the caller
/// named, and the paths that it reports are paths that the caller recognizes.
///
/// A start that names a file yields the directory that holds the file,
/// because the project that governs a file is the project of that directory.
///
/// # Errors
///
/// Returns an error when the start is relative and the working directory is
/// unknown, or when the start does not exist or cannot be read.
// project[impl discover.start.absolute]
// project[impl discover.start.error.unknown-directory]
// project[impl discover.start.error.unreadable]
// project[impl discover.start.file]
fn resolve(
    start: &Path,
    working_directory: std::io::Result<PathBuf>,
) -> Result<PathBuf, DiscoverProjectError> {
    let absolute = if start.is_absolute() {
        start.to_path_buf()
    } else {
        // An empty path is the working directory to every other operation of
        // the standard library, and joining it onto the working directory
        // gives the same answer here.
        let working_directory = working_directory
            .map_err(|source| DiscoverProjectError::UnknownWorkingDirectory { source })?;

        working_directory.join(start)
    };

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

/// Walks from the start to the root of the file system and tests the markers
///
/// The walk is the outer loop, so every marker is tested in one directory
/// before any marker is tested in the directory above it. A project therefore
/// wins over the directory that contains it.
///
/// The markers are tested before the walk begins, because a marker that
/// leaves its directory is a mistake in the application, and reading the file
/// system would not make the answer any better.
///
/// # Errors
///
/// Returns an error when a marker is not a relative path inside a directory,
/// and when no marker matches in any directory of the walk.
// project[impl discover.error.missing]
// project[impl discover.markers]
// project[impl discover.markers.error.outside]
// project[impl discover.markers.walk]
// project[impl discover.order]
// project[impl discover.precedence]
// project[impl discover.result]
// project[impl discover.walk]
fn walk(start: &Path, markers: &[Marker]) -> Result<Project, DiscoverProjectError> {
    if let Some(marker) = markers.iter().find(|marker| !is_inside(marker)) {
        return Err(DiscoverProjectError::OutsideMarker {
            marker: marker.clone(),
        });
    }

    for directory in start.ancestors() {
        for marker in markers {
            if exists(&directory.join(marker.get())) {
                return Ok(Project {
                    root: ProjectRoot::new(directory.to_path_buf()),
                    marker: marker.clone(),
                });
            }
        }
    }

    Err(DiscoverProjectError::MissingProject {
        start: StartDirectory::new(start.to_path_buf()),
        markers: Markers::new(markers.to_vec()),
    })
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every
    // test would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use std::io::ErrorKind;

    use tempfile::TempDir;

    use super::*;

    /// A marker that no directory of the file system holds
    ///
    /// A walk that must not match needs a marker that no ancestor of a
    /// temporary directory holds, up to the root of the file system.
    const ABSENT: &str = ".kawauso-project-absent-marker";

    /// Creates a directory below the temporary directory and returns its path
    fn directory(root: &TempDir, path: &str) -> PathBuf {
        let directory = root.path().join(path);
        std::fs::create_dir_all(&directory).unwrap();

        directory
    }

    /// Creates an empty file below the temporary directory
    fn file(root: &TempDir, path: &str) {
        let file = root.path().join(path);
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(file, "").unwrap();
    }

    // project[verify discover.start.file]
    #[test]
    fn resolve_with_a_file_returns_its_directory() {
        let root = tempfile::tempdir().unwrap();
        file(&root, "src/main.rs");

        let start = resolve(&root.path().join("src").join("main.rs"), Ok(PathBuf::new())).unwrap();

        assert_eq!(start, root.path().join("src"));
    }

    // project[verify discover.start.absolute]
    #[test]
    fn resolve_with_a_relative_start_resolves_against_the_working_directory() {
        let root = tempfile::tempdir().unwrap();
        directory(&root, "src");

        let start = resolve(Path::new("src"), Ok(root.path().to_path_buf())).unwrap();

        assert_eq!(start, root.path().join("src"));
    }

    // project[verify discover.start.error.unreadable]
    #[test]
    fn resolve_with_a_start_that_does_not_exist_returns_an_error() {
        let root = tempfile::tempdir().unwrap();

        let error = resolve(&root.path().join("absent"), Ok(PathBuf::new())).unwrap_err();

        assert!(matches!(
            error,
            DiscoverProjectError::UnreadableStart { .. }
        ));
    }

    // project[verify discover.start.absolute]
    #[test]
    fn resolve_with_dot_components_removes_them() {
        let root = tempfile::tempdir().unwrap();
        directory(&root, "src");

        let start = resolve(
            &root.path().join("src").join("..").join("src"),
            Ok(PathBuf::new()),
        )
        .unwrap();

        assert_eq!(start, root.path().join("src"));
    }

    // project[verify discover.start.error.unknown-directory]
    #[test]
    fn resolve_without_a_working_directory_returns_an_error() {
        let unknown = std::io::Error::new(ErrorKind::NotFound, "the working directory is gone");

        let error = resolve(Path::new("src"), Err(unknown)).unwrap_err();

        assert!(matches!(
            error,
            DiscoverProjectError::UnknownWorkingDirectory { .. }
        ));
    }

    // project[verify discover.markers]
    #[test]
    fn walk_with_a_directory_as_the_marker_returns_the_project() {
        let root = tempfile::tempdir().unwrap();
        directory(&root, ".git");

        let project = walk(root.path(), &[Marker::from(".git")]).unwrap();

        assert_eq!(project.root().get(), root.path());
    }

    // project[verify discover.markers.error.outside]
    #[test]
    fn walk_with_a_marker_above_the_directory_returns_an_error() {
        let root = tempfile::tempdir().unwrap();

        let error = walk(root.path(), &[Marker::from("../.git")]).unwrap_err();

        assert!(matches!(error, DiscoverProjectError::OutsideMarker { .. }));
    }

    // project[verify discover.walk]
    #[test]
    fn walk_with_a_marker_in_an_ancestor_returns_the_ancestor() {
        let root = tempfile::tempdir().unwrap();
        file(&root, "Cargo.toml");
        let start = directory(&root, "crates/example/src");

        let project = walk(&start, &[Marker::from("Cargo.toml")]).unwrap();

        assert_eq!(project.root().get(), root.path());
    }

    // project[verify discover.result]
    #[test]
    fn walk_with_a_marker_in_an_ancestor_returns_the_marker() {
        let root = tempfile::tempdir().unwrap();
        directory(&root, ".git");
        let start = directory(&root, "src");

        let project = walk(&start, &[Marker::from(".git")]).unwrap();

        assert_eq!(project.marker().get(), Path::new(".git"));
    }

    // project[verify discover.order]
    #[test]
    fn walk_with_a_marker_in_two_ancestors_returns_the_nearer_ancestor() {
        let root = tempfile::tempdir().unwrap();
        directory(&root, ".git");
        let parent = directory(&root, "crates");
        directory(&root, "crates/.git");
        let start = directory(&root, "crates/example");

        let project = walk(&start, &[Marker::from(".git")]).unwrap();

        assert_eq!(project.root().get(), parent);
    }

    // project[verify discover.markers.error.outside]
    #[test]
    fn walk_with_a_marker_that_names_no_entry_returns_an_error() {
        let root = tempfile::tempdir().unwrap();

        let error = walk(root.path(), &[Marker::from(".")]).unwrap_err();

        assert!(matches!(error, DiscoverProjectError::OutsideMarker { .. }));
    }

    // project[verify discover.markers.error.outside]
    #[test]
    fn walk_with_an_absolute_marker_returns_an_error() {
        let root = tempfile::tempdir().unwrap();

        let error = walk(root.path(), &[Marker::from("/etc")]).unwrap_err();

        assert!(matches!(error, DiscoverProjectError::OutsideMarker { .. }));
    }

    // project[verify discover.precedence]
    #[test]
    fn walk_with_markers_in_the_start_and_an_ancestor_reports_the_start() {
        let root = tempfile::tempdir().unwrap();
        directory(&root, ".git");
        let start = directory(&root, "crates/example");
        directory(&root, "crates/example/.git");

        let project = walk(&start, &[Marker::from(".git")]).unwrap();

        assert_eq!(project.root().get(), start);
    }

    // project[verify discover.markers.walk]
    #[test]
    fn walk_with_the_second_marker_in_the_start_returns_the_start() {
        let root = tempfile::tempdir().unwrap();
        directory(&root, ".git");
        let start = directory(&root, "crates/example");
        file(&root, "crates/example/Cargo.toml");

        let project = walk(&start, &[Marker::from(".git"), Marker::from("Cargo.toml")]).unwrap();

        assert_eq!(project.root().get(), start);
    }

    // project[verify discover.markers.order]
    #[test]
    fn walk_with_two_markers_in_the_start_returns_the_first() {
        let root = tempfile::tempdir().unwrap();
        directory(&root, ".git");
        file(&root, "Cargo.toml");

        let project = walk(
            root.path(),
            &[Marker::from(".git"), Marker::from("Cargo.toml")],
        )
        .unwrap();

        assert_eq!(project.marker().get(), Path::new(".git"));
    }

    // project[verify discover.error.missing.message]
    #[test]
    fn walk_without_a_match_names_the_start_and_the_markers() {
        let root = tempfile::tempdir().unwrap();

        let error = walk(root.path(), &[Marker::from(ABSENT)]).unwrap_err();

        assert_eq!(
            error.to_string(),
            format!(
                "no directory at or above `{}` holds any of these markers: `{ABSENT}`",
                root.path().display()
            )
        );
    }

    // project[verify discover.error.missing]
    #[test]
    fn walk_without_a_match_returns_an_error() {
        let root = tempfile::tempdir().unwrap();

        let error = walk(root.path(), &[Marker::from(ABSENT)]).unwrap_err();

        assert!(matches!(error, DiscoverProjectError::MissingProject { .. }));
    }
}
