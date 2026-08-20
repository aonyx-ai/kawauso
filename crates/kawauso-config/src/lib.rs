//! Configuration files for Kawauso applications
//!
//! This crate loads the configuration files of Kawauso applications. Every
//! application finds, loads, and deserializes its configuration file in the
//! same way, and reports failures with the same clear errors.

mod error;

use serde::de::DeserializeOwned;

pub use crate::error::DeserializeConfigurationError;
pub use crate::error::field_path::FieldPath;
pub use crate::error::position::Position;

/// Deserializes a TOML document into a type that the caller defines
///
/// The caller gives the contents of a configuration file and a type that
/// describes the expected structure of the file. The type must implement the
/// [`Deserialize`][deserialize] trait of serde, which its derive macro
/// generates.
///
/// # Errors
///
/// Returns [`MalformedDocument`][malformed] when the contents are not valid
/// TOML. The message of the error names the line and the column at which
/// parsing stopped.
///
/// Returns [`MismatchedField`][mismatched] when the document is valid TOML
/// but does not match the type. The message of the error names the path of
/// the field, such as `server.port`.
///
/// # Examples
///
/// ```
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Configuration {
///     port: u16,
/// }
///
/// let configuration: Configuration = kawauso_config::from_str("port = 8080")?;
///
/// assert_eq!(configuration.port, 8080);
/// # Ok::<(), kawauso_config::DeserializeConfigurationError>(())
/// ```
///
/// [deserialize]: https://docs.rs/serde/latest/serde/trait.Deserialize.html
/// [malformed]: DeserializeConfigurationError::MalformedDocument
/// [mismatched]: DeserializeConfigurationError::MismatchedField
// config[impl load.deserialize]
// config[impl load.error]
pub fn from_str<T>(contents: &str) -> Result<T, DeserializeConfigurationError>
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

    Position::new(line, column)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_of_beyond_the_document_returns_the_end() {
        let document = "name = \"kawauso\"\nport = 8080\n";

        let position = position_of(document, usize::MAX);

        assert_eq!(position, Position::new(3, 1));
    }

    #[test]
    fn position_of_inside_a_later_line_returns_that_line() {
        let document = "name = \"kawauso\"\nport = 8080\n";

        let position = position_of(document, 24);

        assert_eq!(position, Position::new(2, 8));
    }

    #[test]
    fn position_of_inside_a_multi_byte_character_returns_the_end() {
        let document = "name = \"ä\"\n";

        let position = position_of(document, 9);

        assert_eq!(position, Position::new(2, 1));
    }

    #[test]
    fn position_of_zero_returns_the_start() {
        let document = "name = \"kawauso\"\n";

        let position = position_of(document, 0);

        assert_eq!(position, Position::new(1, 1));
    }
}
