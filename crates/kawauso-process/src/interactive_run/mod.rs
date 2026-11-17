//! The run of a program that takes over the terminal
//!
//! An editor, a pager, and `ssh` read the keys of the user and draw on the
//! terminal. Such a program needs the streams of the terminal, and the crate
//! captures nothing that it writes. This module holds the handle of such a
//! run.
//!
//! On a Unix platform, the program runs in a process group of its own, and it
//! holds the foreground of the terminal while it runs. The documentation of
//! [`InteractiveRun`] states what follows from that.

#[cfg(unix)]
mod group;
#[cfg(unix)]
mod terminal;

#[cfg(unix)]
use std::io::Error as IoError;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::process::Command as StdCommand;
use std::process::ExitStatus;
use std::time::Duration;

use tokio::process::Child;
use tokio::process::Command;
#[cfg(unix)]
use tokio::signal::unix::Signal;
#[cfg(unix)]
use tokio::signal::unix::SignalKind;
#[cfg(unix)]
use tokio::signal::unix::signal;
#[cfg(unix)]
use tokio::time::timeout;

#[cfg(unix)]
use self::group::Group;
#[cfg(unix)]
use self::group::Request;
#[cfg(unix)]
use self::terminal::Terminal;
use crate::error::RunCommandError;
use crate::invocation::Invocation;
use crate::process_id::ProcessId;

/// The run of a program that takes over the terminal
///
/// [`start_interactive`][start] returns the handle. The program reads and
/// writes the streams that the caller chose, and the handle reports how the
/// program ended. Nothing captures the output.
///
/// # Process Group
///
/// On a Unix platform, the program leads a process group of its own. A
/// program that it starts joins the group, so a stop reaches a wrapper such
/// as `sh -c` and the editor that the wrapper starts. A program that ends by
/// itself leaves such a program running.
///
/// # Terminal
///
/// When this process is in the foreground of its terminal, the start gives
/// the foreground to the group of the program. Ctrl+C, Ctrl+\\, and Ctrl+Z
/// then reach the program alone, and this process needs no handler for them.
/// The handle takes the terminal back when the program ended, and when a stop
/// or a drop ended it. This process must not read the terminal while the
/// program holds it, because a read from the background stops this process.
///
/// A Ctrl+Z stops the program, and not this process. While a wait or a stop
/// runs, the handle takes the terminal back and stops this process with the
/// same signal. The shell of the user then shows the job as stopped. When the
/// shell continues the job, the handle gives the terminal to the program
/// again and continues it.
///
/// The handle does not save the modes of the terminal. A program that a stop
/// kills can leave the terminal in a mode that the caller must reset.
///
/// # Windows
///
/// Windows has no process group of this kind, and every program on a console
/// receives Ctrl+C. A stop and a drop kill the program at once, and a program
/// that the program started survives the kill.
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

    /// The status of the program, once the handle collected it
    status: Option<ExitStatus>,

    /// The process group that the program leads
    #[cfg(unix)]
    group: Option<Group>,

    /// The terminal that the handle lent to the program
    #[cfg(unix)]
    terminal: Option<Terminal>,

    /// The signals that report a change of a program of this process
    #[cfg(unix)]
    changes: Signal,
}

impl InteractiveRun {
    /// Waits until the program that leads the group has ended, and collects nothing
    ///
    /// The status stays with the operating system, so the identifier of the
    /// group stays reserved. A stop can therefore still signal the group. The
    /// wait follows a program that stops while it waits.
    ///
    /// # Errors
    ///
    /// Returns [`IncompleteRun`][incomplete] when the state of the program
    /// cannot be read.
    ///
    /// [incomplete]: RunCommandError::IncompleteRun
    #[cfg(unix)]
    async fn exit(&mut self) -> Result<(), RunCommandError> {
        let Some(group) = self.group else {
            return Ok(());
        };

        loop {
            if group
                .has_ended()
                .map_err(|source| self.incomplete(source))?
            {
                return Ok(());
            }

            if let Some(terminal) = &self.terminal {
                let stop = group
                    .take_stop()
                    .map_err(|source| self.incomplete(source))?;

                if let Some(signal) = stop {
                    terminal.follow(group, signal);

                    continue;
                }
            }

            if self.changes.recv().await.is_none() {
                let source = IoError::other("the runtime no longer reports the end of programs");

                return Err(self.incomplete(source));
            }
        }
    }

    /// Reports that this platform has no wait that collects nothing
    ///
    /// The collection of the status is the wait on this platform.
    ///
    /// # Errors
    ///
    /// Returns no error. The signature is the signature of the wait on a Unix
    /// platform, so that a caller needs no condition of its own.
    #[cfg(not(unix))]
    async fn exit(&mut self) -> Result<(), RunCommandError> {
        Ok(())
    }

    /// Returns the identifier that the operating system gave the program
    ///
    /// On a Unix platform, the identifier also names the process group of
    /// the program. The value is `None` when the operating system gave the
    /// program no identifier.
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

    /// Lends the terminal to the group of the program, and continues the group
    ///
    /// A program that read the terminal before the loan stopped. The program
    /// continues, and it reads again with the foreground.
    ///
    /// # Errors
    ///
    /// Returns [`UnavailableTerminal`][unavailable] when the terminal refuses
    /// the group of the program.
    ///
    /// [unavailable]: RunCommandError::UnavailableTerminal
    #[cfg(unix)]
    fn lend(&self) -> Result<(), RunCommandError> {
        if let (Some(terminal), Some(group)) = (&self.terminal, self.group) {
            terminal
                .lend(group)
                .map_err(|source| RunCommandError::UnavailableTerminal {
                    invocation: Box::new(self.invocation.clone()),
                    source,
                })?;

            group.resume();
        }

        Ok(())
    }

    /// Takes the terminal back and collects the status of the program
    ///
    /// # Errors
    ///
    /// Returns [`IncompleteRun`][incomplete] when the status cannot be
    /// collected.
    ///
    /// [incomplete]: RunCommandError::IncompleteRun
    async fn reap(&mut self) -> Result<ExitStatus, RunCommandError> {
        #[cfg(unix)]
        if let (Some(terminal), Some(group)) = (&self.terminal, self.group) {
            terminal.reclaim(group);
        }

        let status = self
            .child
            .wait()
            .await
            .map_err(|source| self.incomplete(source))?;

        self.status = Some(status);

        Ok(status)
    }

    /// Ends the program and reports its status
    ///
    /// On a Unix platform, the method sends `SIGTERM` and then `SIGCONT` to
    /// the group of the program. A program that a user stopped therefore
    /// answers as well. The method gives the program the time that the caller
    /// names, and then it kills every process of the group. The kill also
    /// ends a program that the program started and that is still there.
    ///
    /// The method also kills the program itself, so a program that left the
    /// group still ends. A platform without a request of this kind kills the
    /// program at once, and the time then passes unused. A program that ended
    /// already reports its status, and the method kills nothing.
    ///
    /// # Errors
    ///
    /// Returns [`IncompleteRun`][incomplete] when the end of the program
    /// cannot be waited for.
    ///
    /// # Panics
    ///
    /// Panics when the Tokio runtime that drives the future has no timer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    ///
    /// use kawauso_process::Invocation;
    /// use kawauso_process::invocation::Streams;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let run = Invocation::new("less").start_interactive(Streams::inherit())?;
    ///
    /// let status = run.stop(Duration::from_secs(5)).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [incomplete]: RunCommandError::IncompleteRun
    pub async fn stop(mut self, grace: Duration) -> Result<ExitStatus, RunCommandError> {
        if let Some(status) = self.status {
            return Ok(status);
        }

        #[cfg(unix)]
        if let Some(group) = self.group {
            if group.request_end() == Request::Sent {
                if let Ok(exited) = timeout(grace, self.exit()).await {
                    exited?;
                }
            }

            group.kill();
        }

        #[cfg(not(unix))]
        let _ = grace;

        let _ = self.child.start_kill();

        self.exit().await?;
        self.reap().await
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
    /// handle, and it can wait again or stop the program.
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
        if let Some(status) = self.status {
            return Ok(status);
        }

        self.exit().await?;

        self.reap().await
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
    /// start, and [`UnavailableTerminal`][unavailable] when the terminal
    /// refuses the group of the program.
    ///
    /// [unavailable]: RunCommandError::UnavailableTerminal
    /// [unstartable]: RunCommandError::UnstartableCommand
    pub(crate) fn spawn_interactive(
        &self,
        command: StdCommand,
    ) -> Result<InteractiveRun, RunCommandError> {
        let unstartable = |source| RunCommandError::UnstartableCommand {
            invocation: Box::new(self.clone()),
            source,
        };

        #[cfg(unix)]
        let command = {
            let mut command = command;
            command.process_group(0);
            command
        };

        let mut command = Command::from(command);
        command.kill_on_drop(true);

        #[cfg(unix)]
        let changes = signal(SignalKind::child()).map_err(unstartable)?;

        let child = command.spawn().map_err(unstartable)?;
        let id = child.id();

        let run = InteractiveRun {
            invocation: self.clone(),
            child,
            id: id.map(ProcessId::from),
            status: None,
            #[cfg(unix)]
            group: id.and_then(Group::led_by),
            #[cfg(unix)]
            terminal: Terminal::claim(),
            #[cfg(unix)]
            changes,
        };

        #[cfg(unix)]
        run.lend()?;

        Ok(run)
    }
}

/// Kills the program when the caller abandons the run
///
/// On a Unix platform, the kill reaches every process of the group, and the
/// handle then takes the terminal back. A program whose status the handle
/// collected has ended, and the drop kills nothing.
impl Drop for InteractiveRun {
    fn drop(&mut self) {
        if self.status.is_some() {
            return;
        }

        #[cfg(unix)]
        if let Some(group) = self.group {
            group.kill();

            if let Some(terminal) = &self.terminal {
                terminal.reclaim(group);
            }
        }

        let _ = self.child.start_kill();
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
