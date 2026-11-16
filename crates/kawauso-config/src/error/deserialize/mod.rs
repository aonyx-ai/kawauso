//! Errors for the deserialization of configuration files

pub mod field_path;
pub mod position;

use thiserror::Error;

pub use self::field_path::FieldPath;
pub use self::position::Position;

/// The error returned when a configuration file cannot be deserialized
///
/// The variants separate the failures by the information that exists for each.
/// Contents that are not valid TOML fail in the parser.
/// Fields do not exist yet, so the error can only point to a position in the text.
/// A valid document that does not match the caller's type fails at a known field.
/// The error points to that field by its path.
///
/// Every variant carries its cause, which [`Error::source`][source] exposes
/// for error reports. The concrete type of the cause is unspecified and can
/// change in any release; do not downcast it.
///
/// A later release can add variants, and it can add fields to a variant.
/// Match with a wildcard arm, and bind the fields of a variant with `..`.
///
/// [source]: https://doc.rust-lang.org/std/error/trait.Error.html#method.source
///
/// # Examples
///
/// ```
/// use serde::Deserialize;
///
/// use kawauso_config::Loader;
/// use kawauso_config::error::DeserializeConfigurationError;
/// use kawauso_config::error::LoadConfigurationError;
///
/// #[derive(Deserialize)]
/// struct Configuration {
///     port: u16,
/// }
///
/// let result = Loader::contents("port = \"eighty\"").load::<Configuration>();
///
/// assert!(matches!(
///     result,
///     Err(LoadConfigurationError::InvalidContents {
///         source: DeserializeConfigurationError::MismatchedField { .. },
///         ..
///     })
/// ));
/// ```
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DeserializeConfigurationError {
    /// The contents are not valid TOML
    ///
    /// The position marks where parsing stopped: the first character that
    /// the parser could not accept. The actual mistake, for example a quote
    /// that was never closed, can lie before that position. The message
    /// gives the line and the column of the position.
    #[error("failed to parse the configuration at {position}")]
    #[non_exhaustive]
    MalformedDocument {
        /// Where parsing stopped
        position: Position,

        /// The cause of the failure
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// The document is valid TOML, but it does not match the caller's type
    ///
    /// The path points to the value that could not be deserialized.
    /// For example, a string where the type expects a number.
    /// A key that the type requires but the document lacks has no path of its own.
    /// In that case, the path points to the table that lacks the key.
    /// The cause names the key, and the message gives the path like `server.port`.
    #[error("failed to deserialize the configuration at `{path}`")]
    #[non_exhaustive]
    MismatchedField {
        /// The path of the value that does not match
        path: FieldPath,

        /// The cause of the failure
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::missing_panics_doc,
        reason = "An assertion in a test panics by design. A `# Panics` section on every test would repeat that and give the reader no information."
    )]

    use super::*;

    /// Shows that `DeserializeConfigurationError` is Send and Sync
    ///
    /// A caller sends the error between threads and keeps it in a report that
    /// another thread reads. This test holds the error to the auto traits that
    /// make this possible. The test exists because a private field of a later
    /// version could take them away without a word from the compiler.
    #[test]
    fn deserialize_configuration_error_is_send_and_sync() {
        fn assert_send_and_sync<T: Send + Sync>() {}

        assert_send_and_sync::<DeserializeConfigurationError>();
    }

    #[test]
    fn deserialize_configuration_error_is_send_sync_and_unpin() {
        fn assert_send_sync_and_unpin<T: Send + Sync + Unpin>() {}

        assert_send_sync_and_unpin::<DeserializeConfigurationError>();
    }
}
