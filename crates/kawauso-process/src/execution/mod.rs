//! The result of one run of an external command
//!
//! A run produces one value that describes what happened: how the command
//! ended, what it wrote to each of its streams, and how long it took. This
//! module holds that value and the type of a capture.
//!
//! A command that ends without success is not a failure of the run. The exit
//! status is part of the result, and the caller decides what it means.

pub mod output;

use std::process::ExitStatus;
use std::time::Duration;

pub use self::output::Output;
use crate::error::RequireSuccessError;
use crate::invocation::Invocation;
use crate::process_id::ProcessId;

/// The result of one run of an external command
///
/// The value holds the command that ran, the exit status of the command, what
/// the command wrote to its standard output and to its standard error, and
/// the time that the run took. It also holds the identifier that the
/// operating system gave the command. The two streams stay apart, because a
/// program separates its result from its diagnostics.
///
/// A status that is not a success is data. The check mode of a formatter ends
/// without success when it finds a file to format, and that is the answer
/// that the caller asked for. A caller that must not accept such a status
/// calls [`require_success`][require-success].
///
/// # Examples
///
/// ```no_run
/// use kawauso_process::Invocation;
///
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let execution = Invocation::new("git").arg("status").run().await?;
///
/// println!("{}", execution.stdout());
/// # Ok(())
/// # }
/// ```
///
/// [require-success]: Execution::require_success
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Execution {
    /// The command that ran
    ///
    /// The result carries it so that a log line and an error message can name
    /// the command without the caller holding it a second time.
    invocation: Invocation,

    /// The identifier that the operating system gave the command
    ///
    /// `None` when no command produced the result, which is the case for a
    /// result that a caller built.
    id: Option<ProcessId>,

    /// The status with which the command ended
    status: ExitStatus,

    /// What the command wrote to its standard output
    stdout: Output,

    /// What the command wrote to its standard error
    stderr: Output,

    /// The time that the run took
    duration: Duration,
}

impl Execution {
    /// Creates the result of a run
    ///
    /// A run builds this value, and a caller builds one where a test stands
    /// in for a command that no one starts. The identifier is the one that
    /// the operating system gave the command, and a result that no command
    /// produced carries none.
    pub fn new(
        invocation: Invocation,
        id: Option<ProcessId>,
        status: ExitStatus,
        stdout: Output,
        stderr: Output,
        duration: Duration,
    ) -> Self {
        Self {
            invocation,
            id,
            status,
            stdout,
            stderr,
            duration,
        }
    }

    /// Returns the time that the run took
    ///
    /// The time covers the whole run, from the start of the program to the
    /// end of the capture. A caller that reports a slow step, or that logs
    /// how long a tool ran, reads it here instead of measuring the call.
    // process[impl run.duration]
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Returns the identifier that the operating system gave the command
    ///
    /// An application that reports which program ran, or that writes the
    /// identifier to a log, reads it here. The value is `None` when no run
    /// produced the result, such as a result that a test built.
    ///
    /// The command of a result has ended, and the operating system can give
    /// the identifier to another command. The value therefore names the
    /// command that ran. It does not name a command that runs now.
    // process[impl run.identity]
    pub fn id(&self) -> Option<ProcessId> {
        self.id
    }

    /// Returns the command that ran
    pub fn invocation(&self) -> &Invocation {
        &self.invocation
    }

    /// Returns the result of a run that ended with success
    ///
    /// A caller that must not accept a command that failed makes the check
    /// with this method instead of reading the status itself. The result
    /// travels on, so a caller can take the output of the command from it.
    ///
    /// # Errors
    ///
    /// Returns [`UnsuccessfulCommand`][unsuccessful] when the command ended
    /// without success. The error names the command, the status, and what the
    /// command wrote to its standard error, which is where a program states
    /// why it stopped.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use kawauso_process::Invocation;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let execution = Invocation::new("git")
    ///     .arg("rev-parse")
    ///     .arg("HEAD")
    ///     .run()
    ///     .await?
    ///     .require_success()?;
    ///
    /// let commit = execution.stdout().to_string_lossy().trim().to_owned();
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [unsuccessful]: RequireSuccessError::UnsuccessfulCommand
    // process[impl run.success]
    pub fn require_success(self) -> Result<Self, RequireSuccessError> {
        if self.status.success() {
            Ok(self)
        } else {
            Err(RequireSuccessError::UnsuccessfulCommand {
                invocation: self.invocation,
                status: self.status,
                stderr: self.stderr,
            })
        }
    }

    /// Returns the status with which the command ended
    ///
    /// A status that is not a success is a normal result. A command that ran
    /// and failed reports its failure here, and the run itself returns an
    /// error only when the command could not run at all.
    // process[impl run.exit]
    pub fn status(&self) -> ExitStatus {
        self.status
    }

    /// Returns what the command wrote to its standard error
    // process[impl run.output]
    pub fn stderr(&self) -> &Output {
        &self.stderr
    }

    /// Returns what the command wrote to its standard output
    // process[impl run.output]
    pub fn stdout(&self) -> &Output {
        &self.stdout
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every
    // test would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    // A caller keeps the result of a run in a report that another thread
    // reads. This test holds the type to the auto traits that make this
    // possible, because a private field of a later version could take them
    // away without a word from the compiler.
    #[test]
    fn execution_is_send_and_sync() {
        fn assert_send_and_sync<T: Send + Sync>() {}

        assert_send_and_sync::<Execution>();
    }
}
