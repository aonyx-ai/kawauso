//! The project that an application runs in
//!
//! A project is a directory that a marker identifies. This module holds the
//! project and the walk that finds it.

pub mod application_name;
pub mod configuration_file;
pub mod configuration_path;
pub mod no_configuration;
pub mod project_root;

use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use bon::bon;
use kawauso_config::Loader;
use serde::de::DeserializeOwned;

pub use self::application_name::ApplicationName;
pub use self::configuration_file::ConfigurationFile;
pub use self::configuration_path::ConfigurationPath;
pub use self::no_configuration::NoConfiguration;
pub use self::project_root::ProjectRoot;
use crate::error::DiscoverProjectError;
use crate::error::LoadProjectError;
use crate::error::discover::Markers;
use crate::search::Fallback;
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
/// The type parameter is the configuration of the project. An application
/// that reads a configuration writes the type that its file deserializes
/// into, such as `Project<Configuration>`.
///
/// The parameter defaults to [`NoConfiguration`] for an application that
/// never reads its configuration file. Without the default, such an
/// application would still have to name a type that it never uses, because
/// nothing else in its code would say what the parameter is. It writes
/// `Project` instead.
///
/// Every project of an application is one type, whether or not a file exists
/// at its configuration path, so a caller can hold more than one project in a
/// collection.
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
/// let project: Project = Project::builder().application("example").load(&search)?;
///
/// // The project reports the canonical path of the directory
/// assert_eq!(project.root().get(), std::fs::canonicalize(&root)?);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Project<T = NoConfiguration> {
    /// The directory of the project
    root: ProjectRoot,

    /// The marker that identified the project
    ///
    /// `None` when no marker matched and the search fell back to the start
    /// directory.
    marker: Option<Marker>,

    /// The configuration that the project holds
    ///
    /// `None` when no file exists at the configuration path of the project.
    configuration: Option<T>,

    /// The path at which the project keeps its configuration file
    ///
    /// The path is known whether or not a file exists at it, so that an
    /// application can tell its user where to put the file.
    configuration_path: ConfigurationPath,
}

#[bon]
impl<T> Project<T>
where
    T: DeserializeOwned,
{
    /// Describes a project, then finds it and loads its configuration
    ///
    /// The developer describes the project once: which file holds the
    /// configuration, and, through the [`Search`], where the walk starts and
    /// which markers identify the project. [`load`][load] then runs the
    /// search and reads the file.
    ///
    /// Every project belongs to an application, and the name of the
    /// application decides where the configuration file is:
    /// `.config/<name>.toml` inside the project. An application whose host
    /// dictates another location names it with
    /// [`configuration_file`][configuration-file] instead. An application
    /// that has no configuration file declares this with
    /// [`without_configuration`][without-configuration], and the project
    /// then reads no file.
    ///
    /// The search must name at least one marker. A search without one does
    /// not compile:
    ///
    /// ```compile_fail
    /// use kawauso_project::Project;
    /// use kawauso_project::Search;
    ///
    /// let search = Search::start("src");
    /// let project: Project = Project::builder().load(&search).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`UndiscoverableProject`][undiscoverable] when the search
    /// produces no project: no marker matched, or the walk could not begin.
    ///
    /// Returns [`UnloadableConfiguration`][unloadable] when a configuration
    /// file exists at the location of the project and cannot be read, parsed,
    /// or deserialized.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde::Deserialize;
    ///
    /// use kawauso_project::Project;
    /// use kawauso_project::Search;
    ///
    /// #[derive(Deserialize)]
    /// struct Configuration {
    ///     port: u16,
    /// }
    ///
    /// let directory = tempfile::tempdir()?;
    /// std::fs::create_dir_all(directory.path().join(".config"))?;
    /// std::fs::write(
    ///     directory.path().join(".config").join("example.toml"),
    ///     "port = 8080",
    /// )?;
    ///
    /// let search = Search::start(directory.path()).marker(".config/example.toml");
    /// let project: Project<Configuration> =
    ///     Project::builder().application("example").load(&search)?;
    ///
    /// assert_eq!(project.configuration().unwrap().port, 8080);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// [application]: ProjectBuilder::application
    /// [configuration-file]: ProjectBuilder::configuration_file
    /// [load]: ProjectBuilder::load
    /// [without-configuration]: ProjectBuilder::without_configuration
    /// [undiscoverable]: LoadProjectError::UndiscoverableProject
    /// [unloadable]: LoadProjectError::UnloadableConfiguration
    // The original function stays private, so `builder` is the only way to
    // describe a project. Its own visibility is set here, because bon gives
    // the generated functions the visibility of the function that it wraps.
    #[builder(
        builder_type(vis = "pub"),
        state_mod(vis = "pub"),
        start_fn(name = builder, vis = "pub"),
        finish_fn(name = load, vis = "pub")
    )]
    // project[impl configuration.load]
    // project[impl configuration.location]
    // project[impl discover.markers.required]
    // project[verify discover.markers.required]
    // project[impl configuration.location.custom]
    // project[impl configuration.location.directory]
    // project[impl configuration.missing]
    // project[impl configuration.none]
    // project[impl discover.start.caller]
    fn new(
        // bon requires the argument of the finishing function before the
        // members that get a setter.
        #[builder(finish_fn)] search: &Search<Marked>,
        #[builder(into)] application: ApplicationName,
        // The generated setter is private. The two public methods on the
        // builder wrap it, and each one requires this member to be unset, so
        // a caller cannot both name a file and opt out of the configuration.
        #[builder(default = ConfigurationSource::Conventional, setters(name = configuration_source, vis = ""))]
        configuration: ConfigurationSource,
    ) -> Result<Self, LoadProjectError> {
        let start = resolve(search.start_directory().get(), std::env::current_dir())
            .map_err(|source| LoadProjectError::UndiscoverableProject { source })?;

        let discovery = walk(&start, search.markers(), search.fallback())
            .map_err(|source| LoadProjectError::UndiscoverableProject { source })?;

        let configuration_file = match &configuration {
            ConfigurationSource::File(file) => file.clone(),
            ConfigurationSource::Conventional | ConfigurationSource::None => {
                configuration_file_of(&application)
            }
            ConfigurationSource::Directory => configuration_directory_of(&application),
        };

        let configuration_path =
            ConfigurationPath::new(discovery.root.get().join(configuration_file.get()));

        let configuration = match configuration {
            // The application declared that it has no configuration file, so
            // a file at the location belongs to something else and stays
            // untouched.
            ConfigurationSource::None => None,
            ConfigurationSource::Conventional
            | ConfigurationSource::File(_)
            | ConfigurationSource::Directory => {
                if exists(configuration_path.get()) {
                    Some(load_configuration(&configuration_path)?)
                } else {
                    None
                }
            }
        };

        Ok(Self {
            root: discovery.root,
            marker: discovery.marker,
            configuration,
            configuration_path,
        })
    }
}

/// Where a project gets its configuration
///
/// The default is the conventional location, which the name of the
/// application decides. A developer whose host dictates another location
/// names it, and a developer whose application has no configuration file at
/// all opts out.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
enum ConfigurationSource {
    /// The conventional location, `.config/<application>.toml`
    Conventional,

    /// A location that the developer names
    File(ConfigurationFile),

    /// The conventional directory, `.config/<application>/config.toml`
    Directory,

    /// The application has no configuration file
    None,
}

impl<'search, T, S: project_builder::State> ProjectBuilder<'search, T, S>
where
    T: DeserializeOwned,
{
    /// Names the file that holds the configuration of the project
    ///
    /// The value is a path relative to the directory of the project. Use this
    /// method for an application whose host dictates a location, such as a
    /// GitHub Action that reads `.github`. An application that keeps its file
    /// at the conventional `.config/<application>.toml` names no file.
    ///
    /// Where the project gets its configuration is one thing, so this method,
    /// [`with_configuration_directory`][directory], and
    /// [`without_configuration`][without] describe the same thing. A builder
    /// that calls more than one of them does not compile.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde::Deserialize;
    ///
    /// use kawauso_project::Project;
    /// use kawauso_project::Search;
    ///
    /// #[derive(Deserialize)]
    /// struct Configuration {
    ///     port: u16,
    /// }
    ///
    /// let directory = tempfile::tempdir()?;
    /// std::fs::create_dir_all(directory.path().join(".github"))?;
    /// std::fs::write(
    ///     directory.path().join(".github").join("example.toml"),
    ///     "port = 8080",
    /// )?;
    ///
    /// let search = Search::start(directory.path()).marker(".github/example.toml");
    /// let project: Project<Configuration> = Project::builder()
    ///     .application("example")
    ///     .configuration_file(".github/example.toml")
    ///     .load(&search)?;
    ///
    /// assert_eq!(project.configuration().unwrap().port, 8080);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// [directory]: ProjectBuilder::with_configuration_directory
    /// [without]: ProjectBuilder::without_configuration
    pub fn configuration_file(
        self,
        configuration_file: impl Into<ConfigurationFile>,
    ) -> ProjectBuilder<'search, T, project_builder::SetConfiguration<S>>
    where
        S::Configuration: project_builder::IsUnset,
    {
        self.configuration_source(ConfigurationSource::File(configuration_file.into()))
    }

    /// Declares that the application has no configuration file
    ///
    /// The project reads no file, and [`configuration`][configuration]
    /// reports `None` for it. Use this method for an application that wants
    /// only the directory of its project. A file at the conventional
    /// location then belongs to something else, and the project leaves it
    /// alone.
    ///
    /// [`configuration_path`][configuration-path] still reports the
    /// conventional location. An application that writes the file later
    /// therefore knows where the file goes.
    ///
    /// Where the project gets its configuration is one thing, so this method,
    /// [`configuration_file`][file], and
    /// [`with_configuration_directory`][directory] describe the same thing. A
    /// builder that calls more than one of them does not compile:
    ///
    /// ```compile_fail
    /// use kawauso_project::Project;
    /// use kawauso_project::Search;
    ///
    /// let search = Search::start(".").marker(".git");
    /// let project: Project = Project::builder()
    ///     .application("example")
    ///     .configuration_file(".github/example.toml")
    ///     .without_configuration()
    ///     .load(&search)
    ///     .unwrap();
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_project::Project;
    /// use kawauso_project::Search;
    ///
    /// let directory = tempfile::tempdir()?;
    /// std::fs::write(directory.path().join(".git"), "")?;
    ///
    /// let search = Search::start(directory.path()).marker(".git");
    /// let project: Project = Project::builder()
    ///     .application("example")
    ///     .without_configuration()
    ///     .load(&search)?;
    ///
    /// assert!(project.configuration().is_none());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// [configuration]: Project::configuration
    /// [configuration-path]: Project::configuration_path
    /// [directory]: ProjectBuilder::with_configuration_directory
    /// [file]: ProjectBuilder::configuration_file
    pub fn without_configuration(
        self,
    ) -> ProjectBuilder<'search, T, project_builder::SetConfiguration<S>>
    where
        S::Configuration: project_builder::IsUnset,
    {
        self.configuration_source(ConfigurationSource::None)
    }

    /// Selects the configuration directory that the application owns in `.config`
    ///
    /// The configuration file becomes `config.toml` in
    /// `.config/<application>`, the directory in which an application keeps
    /// the other files it owns.
    ///
    /// Where the project gets its configuration is one thing, so this method,
    /// [`configuration_file`][file], and [`without_configuration`][without]
    /// describe the same thing. A builder that calls more than one of them
    /// does not compile:
    ///
    /// ```compile_fail
    /// use kawauso_project::Project;
    /// use kawauso_project::Search;
    ///
    /// let search = Search::start(".").marker(".git");
    /// let project: Project = Project::builder()
    ///     .application("example")
    ///     .with_configuration_directory()
    ///     .configuration_file(".github/example.toml")
    ///     .load(&search)
    ///     .unwrap();
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use serde::Deserialize;
    ///
    /// use kawauso_project::Project;
    /// use kawauso_project::Search;
    ///
    /// #[derive(Deserialize)]
    /// struct Configuration {
    ///     port: u16,
    /// }
    ///
    /// let directory = tempfile::tempdir()?;
    /// std::fs::write(directory.path().join(".git"), "")?;
    /// std::fs::create_dir_all(directory.path().join(".config").join("example"))?;
    /// std::fs::write(
    ///     directory.path().join(".config").join("example").join("config.toml"),
    ///     "port = 8080",
    /// )?;
    ///
    /// let search = Search::start(directory.path()).marker(".git");
    /// let project: Project<Configuration> = Project::builder()
    ///     .application("example")
    ///     .with_configuration_directory()
    ///     .load(&search)?;
    ///
    /// assert_eq!(project.configuration().unwrap().port, 8080);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// [file]: ProjectBuilder::configuration_file
    /// [without]: ProjectBuilder::without_configuration
    pub fn with_configuration_directory(
        self,
    ) -> ProjectBuilder<'search, T, project_builder::SetConfiguration<S>>
    where
        S::Configuration: project_builder::IsUnset,
    {
        self.configuration_source(ConfigurationSource::Directory)
    }
}

impl<T> Project<T> {
    /// Returns the configuration that the project holds
    ///
    /// Returns `None` when no file exists at the configuration path of the
    /// project, which is a normal state and not a failure. An application
    /// whose configuration is required reports that itself, and
    /// [`configuration_path`][configuration-path] tells its user where to put
    /// the file.
    ///
    /// [configuration-path]: Project::configuration_path
    pub fn configuration(&self) -> Option<&T> {
        self.configuration.as_ref()
    }

    /// Returns the path at which the project keeps its configuration file
    ///
    /// The path is known whether or not a file exists at it.
    pub fn configuration_path(&self) -> &ConfigurationPath {
        &self.configuration_path
    }

    /// Returns the marker that identified the project
    ///
    /// Returns `None` when no marker matched and the search fell back to the
    /// start directory.
    pub fn marker(&self) -> Option<&Marker> {
        self.marker.as_ref()
    }

    /// Returns the directory of the project
    pub fn root(&self) -> &ProjectRoot {
        &self.root
    }
}

/// What the walk found: a directory, and the marker that identified it
///
/// The walk runs before the crate knows the configuration of the project, so
/// it cannot produce a [`Project`] on its own. This type carries its result
/// until the configuration joins it.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
struct Discovery {
    /// The directory of the project
    root: ProjectRoot,

    /// The marker that identified the project, if one matched
    marker: Option<Marker>,
}

/// Returns the conventional location of the configuration file
///
/// Our projects keep the file of a tool in the subdirectory `.config`, with
/// the name of the application and the extension `.toml`. The crate states
/// this convention here, so that no application repeats it.
// project[impl configuration.location]
fn configuration_file_of(application: &ApplicationName) -> ConfigurationFile {
    ConfigurationFile::new(PathBuf::from(".config").join(format!("{application}.toml")))
}

/// Returns the directory location of the configuration file
///
/// An application that owns the directory `.config/<application>` keeps its
/// configuration in `config.toml` inside it, next to the other files that it
/// writes. The directory, like the file layout, is conventional, so no
/// application repeats it.
// project[impl configuration.location.directory]
fn configuration_directory_of(application: &ApplicationName) -> ConfigurationFile {
    ConfigurationFile::new(
        PathBuf::from(".config")
            .join(application.get())
            .join("config.toml"),
    )
}

/// Reads a configuration file and deserializes it
///
/// The read and the deserialization belong to `kawauso-config`, so a failure
/// in a file of a project is reported in the same words as for every other
/// configuration file. The cause is boxed, because the type that produced it
/// belongs to a dependency and never crosses the boundary of this crate.
///
/// # Errors
///
/// Returns an error when the file cannot be read, parsed, or deserialized.
// project[impl configuration.error]
// project[impl configuration.load]
fn load_configuration<T>(path: &ConfigurationPath) -> Result<T, LoadProjectError>
where
    T: DeserializeOwned,
{
    Loader::path(path.get())
        .load()
        .map_err(|source| LoadProjectError::UnloadableConfiguration {
            path: path.clone(),
            source: Box::new(source),
        })
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

/// Resolves the start of the walk to the canonical path of a directory
///
/// A relative start resolves against the working directory of the process,
/// which the caller supplies, so that a test can name one. The result is the
/// canonical path: the `.` and `..` components go, and the symbolic links
/// resolve.
///
/// The walk goes up one component at a time. A `..` component after a
/// symbolic link would otherwise take the walk through a directory that the
/// caller never entered. The directory that the walk reports is the directory
/// that holds the marker, and a caller that joins a path onto it reaches the
/// entry that the walk saw.
///
/// A start that names a file yields the directory that holds the file,
/// because the project that governs a file is the project of that directory.
///
/// # Errors
///
/// Returns an error when the start is relative and the working directory is
/// unknown, or when the start does not exist or cannot be read.
// project[impl discover.start.absolute+2]
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

    // Canonicalization reads the file system, so a start that does not exist,
    // or that the process cannot read, fails here. The report names the start
    // as the caller gave it, resolved against the working directory, because
    // the canonical path of such a start does not exist.
    let start = std::fs::canonicalize(&absolute).map_err(|source| {
        DiscoverProjectError::UnreadableStart {
            start: StartDirectory::new(absolute),
            source,
        }
    })?;

    // Canonicalization already read the path, so the file system answers this
    // question, and a path that it cannot answer for is no directory.
    if start.is_dir() {
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
fn walk(
    start: &Path,
    markers: &[Marker],
    fallback: Fallback,
) -> Result<Discovery, DiscoverProjectError> {
    if let Some(marker) = markers.iter().find(|marker| !is_inside(marker)) {
        return Err(DiscoverProjectError::OutsideMarker {
            marker: marker.clone(),
        });
    }

    for directory in start.ancestors() {
        for marker in markers {
            if exists(&directory.join(marker.get())) {
                return Ok(Discovery {
                    root: ProjectRoot::new(directory.to_path_buf()),
                    marker: Some(marker.clone()),
                });
            }
        }
    }

    match fallback {
        // project[impl discover.fallback]
        Fallback::Start => Ok(Discovery {
            root: ProjectRoot::new(start.to_path_buf()),
            marker: None,
        }),
        Fallback::Error => Err(DiscoverProjectError::MissingProject {
            start: StartDirectory::new(start.to_path_buf()),
            markers: Markers::new(markers.to_vec()),
        }),
    }
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

    /// Returns the canonical path of the temporary directory
    ///
    /// A temporary directory can sit below a symbolic link, which is what
    /// macOS does for `/var`. The walk reports canonical paths, so a test
    /// that names a location in the directory canonicalizes it first.
    fn canonical(root: &TempDir) -> PathBuf {
        std::fs::canonicalize(root.path()).unwrap()
    }

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

        assert_eq!(start, canonical(&root).join("src"));
    }

    // project[verify discover.start.absolute+2]
    #[test]
    fn resolve_with_a_relative_start_resolves_against_the_working_directory() {
        let root = tempfile::tempdir().unwrap();
        directory(&root, "src");

        let start = resolve(Path::new("src"), Ok(root.path().to_path_buf())).unwrap();

        assert_eq!(start, canonical(&root).join("src"));
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

    // A `..` component after a symbolic link leaves the tree that the link
    // points into. A walk that keeps the link therefore passes through a
    // directory that the user never entered, which this test holds the
    // resolution to.
    // project[verify discover.start.absolute+2]
    #[cfg(unix)]
    #[test]
    fn resolve_with_a_symbolic_link_returns_the_real_path() {
        let root = tempfile::tempdir().unwrap();
        directory(&root, "real/inner");
        std::os::unix::fs::symlink(root.path().join("real"), root.path().join("link")).unwrap();

        let start = resolve(&root.path().join("link").join("inner"), Ok(PathBuf::new())).unwrap();

        assert_eq!(start, canonical(&root).join("real").join("inner"));
    }

    // project[verify discover.start.absolute+2]
    #[test]
    fn resolve_with_dot_components_removes_them() {
        let root = tempfile::tempdir().unwrap();
        directory(&root, "src");

        let start = resolve(
            &root.path().join("src").join("..").join("src"),
            Ok(PathBuf::new()),
        )
        .unwrap();

        assert_eq!(start, canonical(&root).join("src"));
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

        let discovery = walk(root.path(), &[Marker::from(".git")], Fallback::Error).unwrap();

        assert_eq!(discovery.root.get(), root.path());
    }

    // project[verify discover.markers.error.outside]
    #[test]
    fn walk_with_a_marker_above_the_directory_returns_an_error() {
        let root = tempfile::tempdir().unwrap();

        let error = walk(root.path(), &[Marker::from("../.git")], Fallback::Error).unwrap_err();

        assert!(matches!(error, DiscoverProjectError::OutsideMarker { .. }));
    }

    // project[verify discover.walk]
    #[test]
    fn walk_with_a_marker_in_an_ancestor_returns_the_ancestor() {
        let root = tempfile::tempdir().unwrap();
        file(&root, "Cargo.toml");
        let start = directory(&root, "crates/example/src");

        let discovery = walk(&start, &[Marker::from("Cargo.toml")], Fallback::Error).unwrap();

        assert_eq!(discovery.root.get(), root.path());
    }

    // project[verify discover.result]
    #[test]
    fn walk_with_a_marker_in_an_ancestor_returns_the_marker() {
        let root = tempfile::tempdir().unwrap();
        directory(&root, ".git");
        let start = directory(&root, "src");

        let discovery = walk(&start, &[Marker::from(".git")], Fallback::Error).unwrap();

        assert_eq!(discovery.marker.unwrap().get(), Path::new(".git"));
    }

    // project[verify discover.order]
    #[test]
    fn walk_with_a_marker_in_two_ancestors_returns_the_nearer_ancestor() {
        let root = tempfile::tempdir().unwrap();
        directory(&root, ".git");
        let parent = directory(&root, "crates");
        directory(&root, "crates/.git");
        let start = directory(&root, "crates/example");

        let discovery = walk(&start, &[Marker::from(".git")], Fallback::Error).unwrap();

        assert_eq!(discovery.root.get(), parent);
    }

    // project[verify discover.markers.error.outside]
    #[test]
    fn walk_with_a_marker_that_names_no_entry_returns_an_error() {
        let root = tempfile::tempdir().unwrap();

        let error = walk(root.path(), &[Marker::from(".")], Fallback::Error).unwrap_err();

        assert!(matches!(error, DiscoverProjectError::OutsideMarker { .. }));
    }

    // project[verify discover.markers.error.outside]
    #[test]
    fn walk_with_an_absolute_marker_returns_an_error() {
        let root = tempfile::tempdir().unwrap();

        let error = walk(root.path(), &[Marker::from("/etc")], Fallback::Error).unwrap_err();

        assert!(matches!(error, DiscoverProjectError::OutsideMarker { .. }));
    }

    // project[verify discover.precedence]
    #[test]
    fn walk_with_markers_in_the_start_and_an_ancestor_reports_the_start() {
        let root = tempfile::tempdir().unwrap();
        directory(&root, ".git");
        let start = directory(&root, "crates/example");
        directory(&root, "crates/example/.git");

        let discovery = walk(&start, &[Marker::from(".git")], Fallback::Error).unwrap();

        assert_eq!(discovery.root.get(), start);
    }

    // project[verify discover.markers.walk]
    #[test]
    fn walk_with_the_second_marker_in_the_start_returns_the_start() {
        let root = tempfile::tempdir().unwrap();
        directory(&root, ".git");
        let start = directory(&root, "crates/example");
        file(&root, "crates/example/Cargo.toml");

        let discovery = walk(
            &start,
            &[Marker::from(".git"), Marker::from("Cargo.toml")],
            Fallback::Error,
        )
        .unwrap();

        assert_eq!(discovery.root.get(), start);
    }

    // project[verify discover.markers.order]
    #[test]
    fn walk_with_two_markers_in_the_start_returns_the_first() {
        let root = tempfile::tempdir().unwrap();
        directory(&root, ".git");
        file(&root, "Cargo.toml");

        let discovery = walk(
            root.path(),
            &[Marker::from(".git"), Marker::from("Cargo.toml")],
            Fallback::Error,
        )
        .unwrap();

        assert_eq!(discovery.marker.unwrap().get(), Path::new(".git"));
    }

    // project[verify discover.fallback]
    #[test]
    fn walk_without_a_match_and_the_fallback_returns_no_marker() {
        let root = tempfile::tempdir().unwrap();

        let discovery = walk(root.path(), &[Marker::from(ABSENT)], Fallback::Start).unwrap();

        assert!(discovery.marker.is_none());
    }

    // project[verify discover.fallback]
    #[test]
    fn walk_without_a_match_and_the_fallback_returns_the_start() {
        let root = tempfile::tempdir().unwrap();

        let discovery = walk(root.path(), &[Marker::from(ABSENT)], Fallback::Start).unwrap();

        assert_eq!(discovery.root.get(), root.path());
    }

    // project[verify discover.error.missing.message]
    #[test]
    fn walk_without_a_match_names_the_start_and_the_markers() {
        let root = tempfile::tempdir().unwrap();

        let error = walk(root.path(), &[Marker::from(ABSENT)], Fallback::Error).unwrap_err();

        assert_eq!(
            error.to_string(),
            format!(
                "no project exists at or above `{}`, because no directory holds any of these markers: `{ABSENT}`",
                root.path().display()
            )
        );
    }

    // project[verify discover.error.missing]
    #[test]
    fn walk_without_a_match_returns_an_error() {
        let root = tempfile::tempdir().unwrap();

        let error = walk(root.path(), &[Marker::from(ABSENT)], Fallback::Error).unwrap_err();

        assert!(matches!(error, DiscoverProjectError::MissingProject { .. }));
    }
}
