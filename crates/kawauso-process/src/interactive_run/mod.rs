//! The run of a program that takes over the terminal
//!
//! An editor, a pager, and `ssh` read the keys of the user and draw on the
//! terminal. Such a program needs the streams of the terminal, and the crate
//! captures nothing that it writes. This module holds the handle of such a
//! run.

use std::process::Command as StdCommand;
use std::process::ExitStatus;

use tokio::process::Child;
use tokio::process::Command;

use crate::error::RunCommandError;
use crate::invocation::Invocation;
use crate::process_id::ProcessId;

/// The run of a program that takes over the terminal
///
/// [`start_interactive`][start] returns the handle. The program reads and
/// writes the streams that the caller chose, and the handle reports how the
/// program ended. Nothing captures the output.
///
/// The handle owns the program. A caller that drops the handle kills the
/// program at once.
///
/// # Examples
///
/// ```no_run
/// use kawauso_process::Invocation;
/// use kawauso_process::invocation::Streams;
///
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut run = Invocation::new("vi")
///     .arg("notes.md")
///     .start_interactive(Streams::inherit())?;
///
/// let status = run.wait().await?;
///
/// println!("{status}");
/// # Ok(())
/// # }
/// ```
///
/// [start]: Invocation::start_interactive
#[derive(Debug)]
pub struct InteractiveRun {
    /// The command that runs
    invocation: Invocation,

    /// The program itself
    child: Child,

    /// The identifier that the operating system gave the program
    id: Option<ProcessId>,
}

impl InteractiveRun {
    /// Returns the identifier that the operating system gave the program
    ///
    /// The value is `None` when the operating system gave the program no
    /// identifier.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use kawauso_process::Invocation;
    /// use kawauso_process::invocation::Streams;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let run = Invocation::new("less").start_interactive(Streams::inherit())?;
    ///
    /// let id = run.id();
    /// # Ok(())
    /// # }
    /// ```
    pub fn id(&self) -> Option<ProcessId> {
        self.id
    }

    /// Returns the error of a run whose program cannot be followed
    fn incomplete(&self, source: std::io::Error) -> RunCommandError {
        RunCommandError::IncompleteRun {
            invocation: Box::new(self.invocation.clone()),
            source,
        }
    }

    /// Returns the command that runs
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use kawauso_process::Invocation;
    /// use kawauso_process::invocation::Streams;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let run = Invocation::new("less").start_interactive(Streams::inherit())?;
    ///
    /// println!("{}", run.invocation());
    /// # Ok(())
    /// # }
    /// ```
    pub fn invocation(&self) -> &Invocation {
        &self.invocation
    }

    /// Waits for the end of the program and reports its status
    ///
    /// A program that ends without success is no failure of this method. An
    /// editor that ends with a failure, for example, is how a user abandons
    /// an edit. The method reports the same status on every call.
    ///
    /// # Errors
    ///
    /// Returns [`IncompleteRun`][incomplete] when the end of the program
    /// cannot be waited for.
    ///
    /// # Cancel safety
    ///
    /// The method is cancel safe. A caller that drops the future keeps the
    /// handle, and it can wait again or drop it.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use kawauso_process::Invocation;
    /// use kawauso_process::invocation::Streams;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut run = Invocation::new("vi").start_interactive(Streams::inherit())?;
    ///
    /// let saved = run.wait().await?.success();
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [incomplete]: RunCommandError::IncompleteRun
    pub async fn wait(&mut self) -> Result<ExitStatus, RunCommandError> {
        self.child
            .wait()
            .await
            .map_err(|source| self.incomplete(source))
    }
}

/// Starts the program of an interactive run
///
/// The method lives next to the handle, because it builds the handle from the
/// parts that only this module can see.
impl Invocation {
    /// Starts the program of a command whose streams the caller connected
    ///
    /// # Errors
    ///
    /// Returns [`UnstartableCommand`][unstartable] when the program does not
    /// start.
    ///
    /// [unstartable]: RunCommandError::UnstartableCommand
    pub(crate) fn spawn_interactive(
        &self,
        command: StdCommand,
    ) -> Result<InteractiveRun, RunCommandError> {
        let unstartable = |source| RunCommandError::UnstartableCommand {
            invocation: Box::new(self.clone()),
            source,
        };

        let mut command = Command::from(command);
        command.kill_on_drop(true);

        let child = command.spawn().map_err(unstartable)?;
        let id = child.id();

        Ok(InteractiveRun {
            invocation: self.clone(),
            child,
            id: id.map(ProcessId::from),
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::missing_panics_doc,
        reason = "an assertion in a test panics by design, and a section on every test tells the reader nothing"
    )]

    use super::*;

    /// Holds the handle to the auto traits that a caller needs to move it into a task
    #[test]
    fn interactive_run_is_send_sync_and_unpin() {
        fn assert_auto_traits<T: Send + Sync + Unpin>() {}

        assert_auto_traits::<InteractiveRun>();
    }
}
