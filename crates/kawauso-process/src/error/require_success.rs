//! The error of requiring the success of a command

use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use std::process::ExitStatus;

use crate::execution::Output;
use crate::invocation::Invocation;

/// The error returned when a command ended without success
///
/// The error carries the three answers that the reader of a report needs:
/// which command ran, how it ended, and what it said. A program states why it
/// stopped on its standard error, so the error carries that capture and its
/// message shows it.
///
/// The command ran. Nothing failed inside the crate, and the error exists
/// because the caller stated that only success is acceptable.
///
/// A later release can add variants, and it can add fields to a variant.
/// Match with a wildcard arm, and bind the fields of a variant with `..`.
#[derive(Clone, Eq, PartialEq, Debug)]
#[non_exhaustive]
pub enum RequireSuccessError {
    /// The command ended without success
    ///
    /// The exit status says how the command ended. A command that ran to its
    /// end reports a code, and a command that a signal stopped reports no
    /// code at all.
    #[non_exhaustive]
    UnsuccessfulCommand {
        /// The command that ended without success
        invocation: Invocation,

        /// The status with which the command ended
        status: ExitStatus,

        /// What the command wrote to its standard error
        stderr: Output,
    },
}

/// States which command ended how, and what it wrote about it
///
/// The message holds the capture of the standard error, because the reader of
/// a report has only this message and the command wrote its reason there. A
/// command that wrote nothing says so, so that no reader looks for a text
/// that does not exist.
// process[impl run.success.message]
impl Display for RequireSuccessError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        match self {
            Self::UnsuccessfulCommand {
                invocation,
                status,
                stderr,
            } => {
                write!(formatter, "the command `{invocation}` ended with {status}")?;

                let text = stderr.to_string_lossy();
                let reason = text.trim();

                if reason.is_empty() {
                    formatter.write_str(", and it wrote nothing to its standard error")
                } else {
                    write!(formatter, ", and it wrote `{reason}` to its standard error")
                }
            }
        }
    }
}

/// Reports the error to the tools of the ecosystem
///
/// The error has no source. The command ran, and no other operation failed
/// under this one.
impl Error for RequireSuccessError {}

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
    fn require_success_error_is_send_and_sync() {
        fn assert_send_and_sync<T: Send + Sync>() {}

        assert_send_and_sync::<RequireSuccessError>();
    }
}
