//! Loading of configuration files
//!
//! This module turns a configuration into a value of the caller's type.
//! [`Loader`] is the entry point of the crate: a constructor selects
//! the source of the configuration, and [`load`][load] reads the source,
//! parses its contents as TOML, and deserializes them.
//!
//! [load]: Loader::load

pub mod configuration_path;
pub mod contents;

use std::io::ErrorKind;
use std::path::PathBuf;

use serde::de::DeserializeOwned;

pub use self::configuration_path::ConfigurationPath;
pub use self::contents::Contents;
use crate::error::DeserializeConfigurationError;
use crate::error::LoadConfigurationError;
use crate::error::deserialize::FieldPath;
use crate::error::deserialize::Position;
use crate::error::deserialize::position::Column;
use crate::error::deserialize::position::Line;

/// Loads a configuration from one source
///
/// A loader owns one source of a configuration, and each constructor of the
/// loader names one source: [`contents`][contents] for contents that the
/// caller supplies, and [`path`][path] for a file at a caller-supplied path.
/// The constructor takes everything that its source needs, so a loader
/// without a source cannot exist, and [`load`][load] fails only for reasons
/// that come from the source.
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
/// [contents]: Loader::contents
/// [load]: Loader::load
/// [path]: Loader::path
#[derive(Clone, Debug)]
pub struct Loader {
    /// The source from which the loader obtains the configuration
    source: Source,
}

impl Loader {
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
    /// [deserialize]: https://docs.rs/serde/latest/serde/trait.Deserialize.html
    /// [invalid-contents]: LoadConfigurationError::InvalidContents
    /// [invalid-file]: LoadConfigurationError::InvalidFile
    /// [missing]: LoadConfigurationError::MissingFile
    /// [unreadable]: LoadConfigurationError::UnreadableFile
    // config[impl load.file]
    pub fn load<T>(&self) -> Result<T, LoadConfigurationError>
    where
        T: DeserializeOwned,
    {
        match &self.source {
            Source::Contents(contents) => deserialize(contents.get())
                .map_err(|source| LoadConfigurationError::InvalidContents { source }),
            Source::Path(path) => {
                let contents = read(path)?;

                deserialize(&contents).map_err(|source| LoadConfigurationError::InvalidFile {
                    path: path.clone(),
                    source,
                })
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
    /// Contents that the caller supplies directly
    Contents(Contents),

    /// A file at a caller-supplied path
    Path(ConfigurationPath),
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

    use super::*;

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
