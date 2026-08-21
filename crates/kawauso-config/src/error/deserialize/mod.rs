//! Errors for the deserialization of configuration files

pub mod field_path;
pub mod position;

use thiserror::Error;

pub use self::field_path::FieldPath;
pub use self::position::Position;

/// The error returned when a configuration file cannot be deserialized
///
/// The variants separate the failures by the information that exists for
/// each. Contents that are not valid TOML fail in the parser, where fields
/// do not exist yet, so the error can only point to a position in the text.
/// A valid document that does not match the caller's type fails at a known
/// field, so the error points to that field by its path.
///
/// Every variant carries its cause, which [`Error::source`][source] exposes
/// for error reports. The concrete type of the cause is unspecified and can
/// change in any release; do not downcast it.
///
/// A later release can add variants, and it can add fields to a variant.
/// Match with a wildcard arm, and bind the fields of a variant with `..`.
///
/// [source]: https://doc.rust-lang.org/std/error/trait.Error.html#method.source
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DeserializeConfigurationError {
    /// The contents are not valid TOML
    ///
    /// The position marks where parsing stopped: the first character that
    /// the parser could not accept. The actual mistake, for example a quote
    /// that was never closed, can lie before that position.
    // config[impl load.error.syntax]
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
    /// The path points to the value that could not be deserialized, for
    /// example a string where the type expects a number. A key that the type
    /// requires but the document lacks has no path of its own; in that case,
    /// the path points to the table that lacks the key, and the cause names
    /// the key.
    // config[impl load.error.field]
    #[error("failed to deserialize the configuration at `{path}`")]
    #[non_exhaustive]
    MismatchedField {
        /// The path of the value that does not match
        path: FieldPath,

        /// The cause of the failure
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
