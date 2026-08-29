//! The error of running a command

use thiserror::Error;

use crate::invocation::Invocation;

/// The error returned when a command does not run
///
/// The variants separate what happened to the command. A command that never
/// started needs a program that exists, a working directory that exists, and
/// the rights to start a program. A run that started and did not finish means
/// that the operating system stopped the crate from collecting the result,
/// which a caller reports and can try again.
///
/// A command that ran and ended without success is no failure of the run. The
/// result then carries the status, and the caller decides what it means.
///
/// A later release can add variants, and it can add fields to a variant.
/// Match with a wildcard arm, and bind the fields of a variant with `..`.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RunCommandError {
    /// The command started and the run did not finish
    ///
    /// The program ran, and reading one of its streams or waiting for its end
    /// failed. Nothing of the run survives such a failure, so the crate
    /// reports it instead of a result with holes in it. The command itself
    /// can still have done its work.
    // process[impl run.error.message]
    #[error("failed to complete the run of the command `{invocation}`")]
    #[non_exhaustive]
    IncompleteRun {
        /// The command that did not finish
        invocation: Invocation,

        /// The cause of the failure
        source: std::io::Error,
    },

    /// The command did not start
    ///
    /// No program answers to the name, the working directory does not exist,
    /// or the operating system refused to start the program. Nothing ran, so
    /// there is no status and no output to report.
    // process[impl run.error]
    // process[impl run.error.message]
    #[error("failed to start the command `{invocation}`")]
    #[non_exhaustive]
    UnstartableCommand {
        /// The command that did not start
        invocation: Invocation,

        /// The cause of the failure
        source: std::io::Error,
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
    fn run_command_error_is_send_and_sync() {
        fn assert_send_and_sync<T: Send + Sync>() {}

        assert_send_and_sync::<RunCommandError>();
    }
}
