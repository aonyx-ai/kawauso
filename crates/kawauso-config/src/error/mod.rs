//! The failures that the deserialization of a configuration file reports

pub mod field_path;
pub mod position;

use thiserror::Error;

use crate::error::field_path::FieldPath;
use crate::error::position::Position;

/// A configuration file that the crate cannot deserialize
///
/// The two variants divide the failures by what the crate knows. A file that
/// is not TOML fails before an entry exists, and the crate can give only the
/// place in the text. A file that is TOML can still hold the wrong entries
/// for the type. Such a file fails at a known entry, and the crate gives the
/// path of that entry.
///
/// Each variant holds the cause of the failure. Read the cause with
/// [`Error::source`][source] to make a full report. Do not match on the type
/// of the cause, because the crate can change the cause while this enum stays
/// the same.
///
/// A later version can add more variants, and it can add more fields to a
/// variant. Match this enum with a wildcard arm, and bind the fields of a
/// variant with `..`.
///
/// [source]: https://doc.rust-lang.org/std/error/trait.Error.html#method.source
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DeserializeConfigurationError {
    /// The contents are not a TOML document
    ///
    /// The text does not obey the rules of TOML, and the crate can thus read
    /// no entry of the file. The position shows the first place where the
    /// text is no longer TOML. The mistake is often some characters before
    /// that place.
    // config[impl load.error.syntax]
    #[error("failed to parse the configuration at {position}")]
    #[non_exhaustive]
    MalformedDocument {
        /// The place where the text is no longer TOML
        position: Position,

        /// The cause of the failure
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// The document is TOML, but it does not agree with the type
    ///
    /// The entry at the path is absent, or the entry holds a value that the
    /// type cannot accept. An entry that is absent has no place of its own in
    /// the file. For such an entry, the path shows the table that does not
    /// have the key, and the cause gives the name of the key.
    // config[impl load.error.field]
    #[error("failed to deserialize the configuration at `{path}`")]
    #[non_exhaustive]
    MismatchedField {
        /// The place in the file that does not agree with the type
        path: FieldPath,

        /// The cause of the failure
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
