//! The error of parsing the name of an application

use thiserror::Error;

/// The error returned when a value is no valid name of an application
///
/// The name of an application becomes a component of a path, so it must be
/// a name that every platform accepts. The message states the rules, so
/// that the developer can correct the value.
///
/// A later release can add variants, and it can add fields to a variant.
/// Match with a wildcard arm, and bind the fields of a variant with `..`.
///
/// # Examples
///
/// ```
/// use kawauso_project::error::ParseApplicationNameError;
/// use kawauso_project::project::ApplicationName;
///
/// let result = "not/a-name".parse::<ApplicationName>();
///
/// assert!(matches!(
///     result,
///     Err(ParseApplicationNameError::MalformedName { .. })
/// ));
/// ```
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ParseApplicationNameError {
    /// The value does not follow the rules of a name
    ///
    /// The message quotes the value with the escapes of Rust. A control
    /// character in it therefore cannot change the terminal or the log that
    /// shows the message.
    ///
    /// The variant carries no cause: the value is the full diagnosis, and no
    /// other operation failed.
    #[error(
        "the application name {name:?} is malformed, because a name holds 1 to 250 ASCII letters, digits, and hyphens, and does not name a device on Windows"
    )]
    #[non_exhaustive]
    MalformedName {
        /// The value that the developer supplied
        name: String,
    },
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::missing_panics_doc,
        reason = "An assertion in a test panics by design. A `# Panics` section on every test would repeat that and give the reader no information."
    )]

    use super::*;

    /// Shows that `ParseApplicationNameError` is Send and Sync
    ///
    /// A caller sends the error between threads and keeps it in a report that
    /// another thread reads. This test holds the error to the auto traits that
    /// make this possible. The test exists because a private field of a later
    /// version could take them away without a word from the compiler.
    #[test]
    fn parse_application_name_error_is_send_and_sync() {
        fn assert_send_and_sync<T: Send + Sync>() {}

        assert_send_and_sync::<ParseApplicationNameError>();
    }

    #[test]
    fn parse_application_name_error_is_send_sync_and_unpin() {
        fn assert_send_sync_and_unpin<T: Send + Sync + Unpin>() {}

        assert_send_sync_and_unpin::<ParseApplicationNameError>();
    }
}
