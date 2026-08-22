//! Loading of configuration files
//!
//! This module turns a configuration into a value of the caller's type.
//! [`Loader`] is the entry point of the crate: a constructor selects
//! the source of the configuration, and [`load`][load] reads the source,
//! parses its contents as TOML, and deserializes them.
//!
//! A source can be a search. The module then holds the two strategies that
//! find a configuration file: a walk from the working directory up to the
//! root of the file system, and a look into the directory in which the
//! platform keeps the configuration of a user.
//!
//! [load]: Loader::load

pub mod application_name;
pub mod configuration_directory;
pub mod configuration_path;
pub mod contents;

use std::io::ErrorKind;
use std::path::PathBuf;

use serde::de::DeserializeOwned;

pub use self::application_name::ApplicationName;
pub use self::configuration_directory::ConfigurationDirectory;
pub use self::configuration_path::ConfigurationPath;
pub use self::contents::Contents;
use crate::error::DeserializeConfigurationError;
use crate::error::DiscoverConfigurationError;
use crate::error::LoadConfigurationError;
use crate::error::deserialize::FieldPath;
use crate::error::deserialize::Position;
use crate::error::deserialize::position::Column;
use crate::error::deserialize::position::Line;
use crate::error::discover::SearchedLocations;

/// Loads a configuration from one source
///
/// A loader owns one source of a configuration, and each constructor of the
/// loader names one source: [`contents`][contents] for contents that the
/// caller supplies, [`path`][path] for a file at a caller-supplied path,
/// [`ancestors`][ancestors] for a search near the working directory, and
/// [`user`][user] for a search in the directory of the user. The constructor
/// takes everything that its source needs, so a loader without a source
/// cannot exist, and [`load`][load] fails only for reasons that come from
/// the source.
///
/// The loader is not generic over the configuration type; [`load`][load] is.
/// A loader can therefore live in a field, load more than once, and
/// deserialize into more than one type.
///
/// # Examples
///
/// ```
/// use serde::Deserialize;
///
/// use kawauso_config::Loader;
///
/// #[derive(Deserialize)]
/// struct Configuration {
///     port: u16,
/// }
///
/// let configuration: Configuration = Loader::contents("port = 8080").load()?;
///
/// assert_eq!(configuration.port, 8080);
/// # Ok::<(), kawauso_config::error::LoadConfigurationError>(())
/// ```
///
/// [ancestors]: Loader::ancestors
/// [contents]: Loader::contents
/// [load]: Loader::load
/// [path]: Loader::path
/// [user]: Loader::user
#[derive(Clone, Debug)]
pub struct Loader {
    /// The source from which the loader obtains the configuration
    source: Source,
}

impl Loader {
    /// Creates a loader that searches the working directory and its ancestors
    ///
    /// The configuration of a project belongs to the project, and a user
    /// runs the application from the project or from a directory in it. The
    /// search therefore starts at the working directory of the process and
    /// goes up, one directory at a time, until it reaches the root of the
    /// file system. It looks in each directory for a file that has the name
    /// of the application and the extension `.toml`, and it takes the first
    /// one that it finds. A project can thus override the configuration of
    /// the directory that contains it.
    ///
    /// The search never reads the configuration directory of the user;
    /// [`user`][user] does that. Use this constructor for an application
    /// whose configuration belongs to a project.
    ///
    /// The working directory is read when [`load`][load] runs, not at the
    /// time of this call.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use serde::Deserialize;
    ///
    /// use kawauso_config::Loader;
    ///
    /// #[derive(Deserialize)]
    /// struct Configuration {
    ///     port: u16,
    /// }
    ///
    /// // Reads the first `kawauso.toml` at or above the working directory
    /// let configuration: Configuration = Loader::ancestors("kawauso").load()?;
    /// # Ok::<(), kawauso_config::error::LoadConfigurationError>(())
    /// ```
    ///
    /// [load]: Loader::load
    /// [user]: Loader::user
    pub fn ancestors(application: impl Into<String>) -> Self {
        Self {
            source: Source::Ancestors(ApplicationName::new(application)),
        }
    }

    /// Creates a loader that deserializes caller-supplied contents
    ///
    /// The caller gives the contents of a configuration document, for
    /// example an embedded default or a test fixture. [`load`][load] parses
    /// these contents as TOML and does not touch the file system.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde::Deserialize;
    ///
    /// use kawauso_config::Loader;
    ///
    /// #[derive(Deserialize)]
    /// struct Configuration {
    ///     name: String,
    /// }
    ///
    /// let configuration: Configuration = Loader::contents("name = \"kawauso\"").load()?;
    ///
    /// assert_eq!(configuration.name, "kawauso");
    /// # Ok::<(), kawauso_config::error::LoadConfigurationError>(())
    /// ```
    ///
    /// [load]: Loader::load
    pub fn contents(contents: impl Into<String>) -> Self {
        Self {
            source: Source::Contents(Contents::new(contents)),
        }
    }

    /// Creates a loader that reads the file at the caller-supplied path
    ///
    /// The loader stores the path as given. A relative path resolves against
    /// the working directory of the process at the time [`load`][load] runs,
    /// not at the time of this call.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde::Deserialize;
    ///
    /// use kawauso_config::Loader;
    ///
    /// #[derive(Deserialize)]
    /// struct Configuration {
    ///     port: u16,
    /// }
    ///
    /// let directory = tempfile::tempdir()?;
    /// let path = directory.path().join("kawauso.toml");
    /// std::fs::write(&path, "port = 8080")?;
    ///
    /// let configuration: Configuration = Loader::path(&path).load()?;
    ///
    /// assert_eq!(configuration.port, 8080);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// [load]: Loader::load
    pub fn path(path: impl Into<PathBuf>) -> Self {
        Self {
            source: Source::Path(ConfigurationPath::new(path.into())),
        }
    }

    /// Creates a loader that searches the configuration directory of the user
    ///
    /// The configuration of a user belongs to the user, and every platform
    /// has a place for it. The application gets a directory of its own in
    /// that place, which leaves room for the files that it writes later, and
    /// the search reads the file `config.toml` in that directory.
    ///
    /// On Linux, and on the other systems that follow the [XDG Base
    /// Directory Specification][xdg], the place is the directory that
    /// `XDG_CONFIG_HOME` names, and `.config` in the home directory when the
    /// variable holds no absolute path. On macOS, it is `Library/Application
    /// Support` in the home directory, and a user who sets `XDG_CONFIG_HOME`
    /// keeps the file where the rest of the platform is. On Windows, it is
    /// the directory for the roaming application data of the user.
    ///
    /// The search never reads the working directory;
    /// [`ancestors`][ancestors] does that. Use this constructor for an
    /// application whose configuration belongs to a user.
    ///
    /// The environment of the process is read when [`load`][load] runs, not
    /// at the time of this call.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use serde::Deserialize;
    ///
    /// use kawauso_config::Loader;
    ///
    /// #[derive(Deserialize)]
    /// struct Configuration {
    ///     port: u16,
    /// }
    ///
    /// // Reads `kawauso/config.toml` in the directory of the platform
    /// let configuration: Configuration = Loader::user("kawauso").load()?;
    /// # Ok::<(), kawauso_config::error::LoadConfigurationError>(())
    /// ```
    ///
    /// [ancestors]: Loader::ancestors
    /// [load]: Loader::load
    /// [xdg]: https://specifications.freedesktop.org/basedir/latest/
    pub fn user(application: impl Into<String>) -> Self {
        Self {
            source: Source::User(ApplicationName::new(application)),
        }
    }

    /// Loads the configuration and deserializes it into the caller's type
    ///
    /// The method obtains the contents from the source of the loader, parses
    /// them as TOML, and deserializes them into `T`. The type must implement
    /// the [`Deserialize`][deserialize] trait of serde, which its derive
    /// macro generates.
    ///
    /// # Errors
    ///
    /// Returns [`MissingFile`][missing] when the source is a path and no
    /// file exists at that path.
    ///
    /// Returns [`UnreadableFile`][unreadable] when the source is a path and
    /// the file at that path cannot be read.
    ///
    /// Returns [`InvalidFile`][invalid-file] when the source is a path and
    /// the contents of the file are not valid TOML or do not match `T`.
    ///
    /// Returns [`InvalidContents`][invalid-contents] when the source is
    /// caller-supplied contents, and they are not valid TOML or do not
    /// match `T`.
    ///
    /// Returns [`UndiscoverableFile`][undiscoverable] when the source is a
    /// search, and the search finds no configuration file.
    ///
    /// [deserialize]: https://docs.rs/serde/latest/serde/trait.Deserialize.html
    /// [invalid-contents]: LoadConfigurationError::InvalidContents
    /// [invalid-file]: LoadConfigurationError::InvalidFile
    /// [missing]: LoadConfigurationError::MissingFile
    /// [undiscoverable]: LoadConfigurationError::UndiscoverableFile
    /// [unreadable]: LoadConfigurationError::UnreadableFile
    // Each arm reads the locations of its own source, and no arm falls back
    // to the locations of another one, so the user always knows where the
    // application looks.
    // config[impl discover.strategy]
    pub fn load<T>(&self) -> Result<T, LoadConfigurationError>
    where
        T: DeserializeOwned,
    {
        match &self.source {
            Source::Ancestors(application) => {
                let found = find_in_ancestors(std::env::current_dir(), application);

                load_found(found, application)
            }
            Source::Contents(contents) => load_contents(contents.get()),
            Source::Path(path) => load_file(path),
            Source::User(application) => {
                let directory = ConfigurationDirectory::of_platform();
                let found = find_in_user_directory(directory, application);

                load_found(found, application)
            }
        }
    }
}

/// The source from which a loader obtains the configuration
///
/// Each variant stores what one constructor of the loader takes. The enum is
/// private on purpose: the public vocabulary of the crate is its set of
/// constructors, which can grow without a breaking change, while a public
/// enum would fix the vocabulary in its first release.
#[derive(Clone, Debug)]
enum Source {
    /// A search of the working directory and its ancestors
    Ancestors(ApplicationName),

    /// Contents that the caller supplies directly
    Contents(Contents),

    /// A file at a caller-supplied path
    Path(ConfigurationPath),

    /// A search of the configuration directory of the user
    User(ApplicationName),
}

/// Loads a configuration from contents that the caller supplied
///
/// # Errors
///
/// Returns an error when the contents are not valid TOML, or when the
/// document does not match the caller's type.
fn load_contents<T>(contents: &str) -> Result<T, LoadConfigurationError>
where
    T: DeserializeOwned,
{
    deserialize(contents).map_err(|source| LoadConfigurationError::InvalidContents { source })
}

/// Loads a configuration from the file at a caller-supplied path
///
/// Deserializing the file reuses the function for caller-supplied
/// contents, so the two sources cannot drift apart. The failure of that
/// shared step then gains the path of the file, so that the report names
/// the file that has to change.
///
/// # Errors
///
/// Returns an error when the file cannot be read, when its contents are
/// not valid TOML, or when the document does not match the caller's type.
// config[impl load.file]
fn load_file<T>(path: &ConfigurationPath) -> Result<T, LoadConfigurationError>
where
    T: DeserializeOwned,
{
    let contents = read(path)?;

    load_contents(&contents).map_err(|error| match error {
        LoadConfigurationError::InvalidContents { source } => LoadConfigurationError::InvalidFile {
            path: path.clone(),
            source,
        },
        error => error,
    })
}

/// Loads the configuration file that a search found
///
/// The search and the load are separate actions with separate errors, and
/// this function joins them. A search that found nothing becomes a load
/// failure that names the application, and a search that found a file hands
/// its path to the load.
///
/// # Errors
///
/// Returns an error when the search found no file, when the file cannot be
/// read, when its contents are not valid TOML, or when the document does not
/// match the caller's type.
// config[impl discover.load]
fn load_found<T>(
    found: Result<ConfigurationPath, DiscoverConfigurationError>,
    application: &ApplicationName,
) -> Result<T, LoadConfigurationError>
where
    T: DeserializeOwned,
{
    let path = found.map_err(|source| LoadConfigurationError::UndiscoverableFile {
        application: application.clone(),
        source,
    })?;

    load_file(&path)
}

/// Finds the configuration file of an application near the working directory
///
/// The working directory arrives as a result, and not as a path, because the
/// operating system can refuse to report it. The refusal becomes a failure
/// of the search here, next to the walk that needs the directory.
///
/// The locations are the working directory and each of its ancestors, in
/// that order, and each of them holds a file with the name of the
/// application and the extension `.toml`.
///
/// # Errors
///
/// Returns an error when the working directory is unknown, when no location
/// holds the file, or when a location holds a directory with the name of the
/// file.
// config[impl discover.ancestors.error.unknown-directory]
// config[impl discover.ancestors.name]
// config[impl discover.ancestors.order]
// config[impl discover.ancestors.parents]
// config[impl discover.ancestors.working-directory]
fn find_in_ancestors(
    working_directory: std::io::Result<PathBuf>,
    application: &ApplicationName,
) -> Result<ConfigurationPath, DiscoverConfigurationError> {
    let working_directory = working_directory
        .map_err(|source| DiscoverConfigurationError::UnknownWorkingDirectory { source })?;

    let file_name = format!("{application}.toml");
    let locations = working_directory
        .ancestors()
        .map(|directory| ConfigurationPath::new(directory.join(&file_name)))
        .collect::<Vec<_>>();

    search(SearchedLocations::new(locations))
}

/// Finds the configuration file of an application in the directory of the user
///
/// The directory arrives as an option, and not as a path, because the
/// environment of the process can leave the platform without an answer. The
/// missing answer becomes a failure of the search here, next to the location
/// that needs the directory.
///
/// The application gets a directory of its own, so that it has room for the
/// files that it writes later, and its configuration is the file
/// `config.toml` in that directory.
///
/// # Errors
///
/// Returns an error when the configuration directory of the platform is
/// unknown, when the location holds no file, or when it holds a directory
/// with the name of the file.
// config[impl discover.user.error.unknown-directory]
// config[impl discover.user.name]
fn find_in_user_directory(
    directory: Option<ConfigurationDirectory>,
    application: &ApplicationName,
) -> Result<ConfigurationPath, DiscoverConfigurationError> {
    let directory =
        directory.ok_or(DiscoverConfigurationError::UnknownConfigurationDirectory {})?;
    let location = directory.get().join(application.get()).join("config.toml");

    search(SearchedLocations::new(vec![ConfigurationPath::new(
        location,
    )]))
}

/// Returns the first location in the list that holds a configuration file
///
/// A location that the file system cannot report on holds no configuration
/// file. A parent directory that does not exist, and one that the user
/// cannot read, are therefore not failures of the search.
///
/// A directory with the name of the configuration file ends the search. Such
/// a directory is a mistake that is hard to see, and a file from a location
/// that follows it would hide the mistake.
///
/// Only a regular file is a configuration file. A location that holds a
/// socket, a device, or a named pipe holds none, and the search moves on. A
/// read of such a location fails, and a read of a named pipe blocks until
/// another process writes to it.
///
/// # Errors
///
/// Returns an error when no location holds the file, or when a location
/// holds a directory with the name of the file.
// config[impl discover.ancestors.precedence]
// config[impl discover.error]
// config[impl discover.error.directory]
fn search(locations: SearchedLocations) -> Result<ConfigurationPath, DiscoverConfigurationError> {
    for location in locations.as_slice() {
        let Ok(metadata) = std::fs::metadata(location.get()) else {
            continue;
        };

        if metadata.is_dir() {
            return Err(DiscoverConfigurationError::UnexpectedDirectory {
                path: location.clone(),
            });
        }

        if metadata.is_file() {
            return Ok(location.clone());
        }
    }

    Err(DiscoverConfigurationError::MissingFile { locations })
}

/// Deserializes the contents of a configuration document into the caller's type
///
/// Parsing and deserialization are separate steps, because a failure in each
/// carries different context: a parse failure has a position in the text,
/// and a mismatch has the path of a field.
///
/// # Errors
///
/// Returns an error when the contents are not valid TOML, or when the
/// document does not match the caller's type.
// config[impl load.deserialize]
// config[impl load.error]
fn deserialize<T>(contents: &str) -> Result<T, DeserializeConfigurationError>
where
    T: DeserializeOwned,
{
    let deserializer = toml::Deserializer::parse(contents).map_err(|error| {
        let offset = error.span().map_or(0, |span| span.start);

        DeserializeConfigurationError::MalformedDocument {
            position: position_of(contents, offset),
            source: Box::new(error),
        }
    })?;

    serde_path_to_error::deserialize(deserializer).map_err(|error| {
        let path = FieldPath::new(error.path().to_string());

        DeserializeConfigurationError::MismatchedField {
            path,
            source: Box::new(error.into_inner()),
        }
    })
}

/// Reads the configuration file at the caller-supplied path
///
/// A path at which no file exists and a file that cannot be read map to
/// different variants. The distinction comes from the kind of the read
/// error, not from a check before the read, so no window exists in which
/// another process can create or remove the file between a check and the
/// read.
///
/// # Errors
///
/// Returns an error when no file exists at the path, or when the file
/// cannot be read.
// config[impl load.file.error]
fn read(path: &ConfigurationPath) -> Result<String, LoadConfigurationError> {
    std::fs::read_to_string(path.get()).map_err(|source| match source.kind() {
        ErrorKind::NotFound => LoadConfigurationError::MissingFile { path: path.clone() },
        _ => LoadConfigurationError::UnreadableFile {
            path: path.clone(),
            source,
        },
    })
}

/// Translates a byte offset in a document into a line and a column
///
/// Lines and columns count from one, and columns count characters, not
/// bytes. An offset that is out of bounds, or that points into the middle of
/// a multi-byte character, yields the position of the end of the document:
/// in an error report, an imprecise position is better than a panic.
fn position_of(document: &str, offset: usize) -> Position {
    let head = document.get(..offset).unwrap_or(document);

    let line = head.matches('\n').count() + 1;
    let column = head.rsplit('\n').next().unwrap_or_default().chars().count() + 1;

    Position::new(Line::new(line), Column::new(column))
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every
    // test would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use serde::Deserialize;

    use super::*;

    /// The configuration of an imaginary application
    ///
    /// A caller of the crate defines a type like this one for its own
    /// configuration file.
    #[derive(Eq, PartialEq, Debug, Deserialize)]
    struct Configuration {
        port: u16,
    }

    /// Creates a directory in another directory and returns its path
    fn child_of(parent: &std::path::Path) -> PathBuf {
        let child = parent.join("project");
        std::fs::create_dir(&child).unwrap();

        child
    }

    /// Returns the locations that a failed search read
    fn locations_of(error: &DiscoverConfigurationError) -> Vec<PathBuf> {
        let DiscoverConfigurationError::MissingFile { locations } = error else {
            panic!("expected the MissingFile variant, got {error:?}");
        };

        locations
            .as_slice()
            .iter()
            .map(|location| location.get().to_path_buf())
            .collect()
    }

    // config[verify discover.error.directory]
    #[test]
    fn find_in_ancestors_with_a_directory_ends_the_search() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("kawauso.toml")).unwrap();

        let error = find_in_ancestors(
            Ok(root.path().to_path_buf()),
            &ApplicationName::new("kawauso"),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            DiscoverConfigurationError::UnexpectedDirectory { .. }
        ));
    }

    // config[verify discover.error.directory.path]
    #[test]
    fn find_in_ancestors_with_a_directory_reports_its_path() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("kawauso.toml");
        std::fs::create_dir(&path).unwrap();

        let error = find_in_ancestors(
            Ok(root.path().to_path_buf()),
            &ApplicationName::new("kawauso"),
        )
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            format!(
                "the path `{}` is a directory, not a configuration file",
                path.display()
            )
        );
    }

    // config[verify discover.ancestors.parents]
    #[test]
    fn find_in_ancestors_with_a_file_in_a_parent_returns_it() {
        let root = tempfile::tempdir().unwrap();
        let working_directory = child_of(root.path());
        std::fs::write(root.path().join("kawauso.toml"), "port = 8080").unwrap();

        let path =
            find_in_ancestors(Ok(working_directory), &ApplicationName::new("kawauso")).unwrap();

        assert_eq!(path.get(), root.path().join("kawauso.toml"));
    }

    // config[verify discover.ancestors.working-directory]
    #[test]
    fn find_in_ancestors_with_a_file_in_the_working_directory_returns_it() {
        let root = tempfile::tempdir().unwrap();
        let working_directory = child_of(root.path());
        std::fs::write(working_directory.join("kawauso.toml"), "port = 8080").unwrap();

        let path = find_in_ancestors(
            Ok(working_directory.clone()),
            &ApplicationName::new("kawauso"),
        )
        .unwrap();

        assert_eq!(path.get(), working_directory.join("kawauso.toml"));
    }

    // A named pipe is the case that matters: a read of one blocks until
    // another process writes to it, so a search that took it for a
    // configuration file would hang. A socket is the object that a test can
    // create without a call to libc.
    #[cfg(unix)]
    #[test]
    fn find_in_ancestors_with_a_socket_continues_the_search() {
        let root = tempfile::tempdir().unwrap();
        let working_directory = child_of(root.path());
        std::os::unix::net::UnixListener::bind(working_directory.join("kawauso.toml")).unwrap();
        std::fs::write(root.path().join("kawauso.toml"), "port = 8080").unwrap();

        let path =
            find_in_ancestors(Ok(working_directory), &ApplicationName::new("kawauso")).unwrap();

        assert_eq!(path.get(), root.path().join("kawauso.toml"));
    }

    // config[verify discover.ancestors.name]
    #[test]
    fn find_in_ancestors_with_another_file_in_the_working_directory_skips_it() {
        let root = tempfile::tempdir().unwrap();
        let working_directory = child_of(root.path());
        std::fs::write(working_directory.join("other.toml"), "port = 8080").unwrap();
        std::fs::write(root.path().join("kawauso.toml"), "port = 8080").unwrap();

        let path =
            find_in_ancestors(Ok(working_directory), &ApplicationName::new("kawauso")).unwrap();

        assert_eq!(path.get(), root.path().join("kawauso.toml"));
    }

    // config[verify discover.ancestors.precedence]
    #[test]
    fn find_in_ancestors_with_files_in_two_directories_returns_the_nearest() {
        let root = tempfile::tempdir().unwrap();
        let working_directory = child_of(root.path());
        std::fs::write(working_directory.join("kawauso.toml"), "port = 8080").unwrap();
        std::fs::write(root.path().join("kawauso.toml"), "port = 9090").unwrap();

        let path = find_in_ancestors(
            Ok(working_directory.clone()),
            &ApplicationName::new("kawauso"),
        )
        .unwrap();

        assert_eq!(path.get(), working_directory.join("kawauso.toml"));
    }

    // config[verify discover.ancestors.order]
    #[test]
    fn find_in_ancestors_without_a_file_lists_the_locations_from_the_bottom_up() {
        let root = tempfile::tempdir().unwrap();
        let working_directory = child_of(root.path());
        let expected = working_directory
            .ancestors()
            .map(|directory| directory.join("kawauso.toml"))
            .collect::<Vec<_>>();

        let error =
            find_in_ancestors(Ok(working_directory), &ApplicationName::new("kawauso")).unwrap_err();

        assert_eq!(locations_of(&error), expected);
    }

    // config[verify discover.error.missing]
    #[test]
    fn find_in_ancestors_without_a_file_reports_every_location() {
        let root = tempfile::tempdir().unwrap();
        let working_directory = child_of(root.path());
        let expected = working_directory
            .ancestors()
            .map(|directory| format!("`{}`", directory.join("kawauso.toml").display()))
            .collect::<Vec<_>>()
            .join(", ");

        let error =
            find_in_ancestors(Ok(working_directory), &ApplicationName::new("kawauso")).unwrap_err();

        assert_eq!(
            error.to_string(),
            format!("no configuration file exists at any of these locations: {expected}")
        );
    }

    // config[verify discover.error]
    #[test]
    fn find_in_ancestors_without_a_file_returns_an_error() {
        let root = tempfile::tempdir().unwrap();

        let result = find_in_ancestors(
            Ok(root.path().to_path_buf()),
            &ApplicationName::new("kawauso"),
        );

        assert!(result.is_err());
    }

    // config[verify discover.ancestors.error.unknown-directory]
    #[test]
    fn find_in_ancestors_without_a_working_directory_returns_an_error() {
        let failure = std::io::Error::from(ErrorKind::NotFound);

        let error = find_in_ancestors(Err(failure), &ApplicationName::new("kawauso")).unwrap_err();

        assert!(matches!(
            error,
            DiscoverConfigurationError::UnknownWorkingDirectory { .. }
        ));
    }

    // The two strategies look for different files in different places. A file
    // that the walk up the ancestors would find is therefore no answer for the
    // search in the directory of the user.
    // config[verify discover.strategy]
    #[test]
    fn find_in_user_directory_with_a_file_of_the_other_strategy_returns_an_error() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("kawauso.toml"), "port = 8080").unwrap();

        let result = find_in_user_directory(
            Some(ConfigurationDirectory::from(root.path())),
            &ApplicationName::new("kawauso"),
        );

        assert!(result.is_err());
    }

    // config[verify discover.user.name]
    #[test]
    fn find_in_user_directory_without_a_file_searches_the_directory_of_the_application() {
        let root = tempfile::tempdir().unwrap();
        let expected = vec![root.path().join("kawauso").join("config.toml")];

        let error = find_in_user_directory(
            Some(ConfigurationDirectory::from(root.path())),
            &ApplicationName::new("kawauso"),
        )
        .unwrap_err();

        assert_eq!(locations_of(&error), expected);
    }

    // config[verify discover.user.error.unknown-directory]
    #[test]
    fn find_in_user_directory_without_a_directory_returns_an_error() {
        let error = find_in_user_directory(None, &ApplicationName::new("kawauso")).unwrap_err();

        assert!(matches!(
            error,
            DiscoverConfigurationError::UnknownConfigurationDirectory { .. }
        ));
    }

    // config[verify discover.load]
    #[test]
    fn load_found_with_a_file_returns_the_configuration() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("kawauso.toml");
        std::fs::write(&path, "port = 8080").unwrap();

        let configuration: Configuration = load_found(
            Ok(ConfigurationPath::new(path)),
            &ApplicationName::new("kawauso"),
        )
        .unwrap();

        assert_eq!(configuration, Configuration { port: 8080 });
    }

    #[test]
    fn position_of_beyond_the_document_returns_the_end() {
        let document = "name = \"kawauso\"\nport = 8080\n";

        let position = position_of(document, usize::MAX);

        assert_eq!(position, Position::new(Line::new(3), Column::new(1)));
    }

    #[test]
    fn position_of_inside_a_later_line_returns_that_line() {
        let document = "name = \"kawauso\"\nport = 8080\n";

        let position = position_of(document, 24);

        assert_eq!(position, Position::new(Line::new(2), Column::new(8)));
    }

    #[test]
    fn position_of_inside_a_multi_byte_character_returns_the_end() {
        let document = "name = \"ä\"\n";

        let position = position_of(document, 9);

        assert_eq!(position, Position::new(Line::new(2), Column::new(1)));
    }

    #[test]
    fn position_of_zero_returns_the_start() {
        let document = "name = \"kawauso\"\n";

        let position = position_of(document, 0);

        assert_eq!(position, Position::new(Line::new(1), Column::new(1)));
    }
}
