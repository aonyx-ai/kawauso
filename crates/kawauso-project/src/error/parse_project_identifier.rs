//! The error of parsing the identifier of a project

use thiserror::Error;

/// The error returned when a value is no valid identifier of a project
///
/// The identifier of a project becomes a component of a path, so it must be
/// a name that every platform accepts. The value often comes from a file
/// that the user of the application wrote, and the message states the
/// rules, so that the user can correct the value.
///
/// A later release can add variants, and it can add fields to a variant.
/// Match with a wildcard arm, and bind the fields of a variant with `..`.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ParseProjectIdentifierError {
    /// The value does not follow the rules of a name
    ///
    /// The value can come from a file that the user did not write. The
    /// message therefore quotes it with the escapes of Rust, so that a
    /// control character in it cannot change the terminal or the log that
    /// shows the message.
    ///
    /// The variant carries no cause: the value is the full diagnosis, and no
    /// other operation failed.
    // project[impl name.error]
    #[error(
        "the project identifier {identifier:?} is malformed, because an identifier holds 1 to 250 ASCII letters, digits, and hyphens, and does not name a device on Windows"
    )]
    #[non_exhaustive]
    MalformedIdentifier {
        /// The value that the caller supplied
        identifier: String,
    },
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every
    // test would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    // A caller sends the error between threads and keeps it in a report that
    // another thread reads. This test holds the error to the auto traits that
    // make this possible, because a private field of a later version could
    // take them away without a word from the compiler.
    #[test]
    fn parse_project_identifier_error_is_send_and_sync() {
        fn assert_send_and_sync<T: Send + Sync>() {}

        assert_send_and_sync::<ParseProjectIdentifierError>();
    }
}
