//! The process group of an interactive run

use std::io::Error as IoError;

use rustix::process::Pid;
use rustix::process::Signal;
use rustix::process::WaitId;
use rustix::process::WaitIdOptions;
use rustix::process::kill_process_group;
use rustix::process::waitid;

/// The process group in which an interactive run starts its program
///
/// The program leads the group, so the identifier of the group is the
/// identifier of the program. A program that the program starts joins the
/// group, unless it starts a group of its own. A signal to the group therefore
/// reaches a wrapper such as `sh -c` and the program that the wrapper starts.
///
/// The operating system keeps the identifier of the group while the program
/// exists, and a program that ended exists until its status is collected. A
/// caller signals the group only before it collects that status. A signal
/// after the collection can reach a group that another program leads.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) struct Group {
    /// The identifier of the program, which is also the identifier of the group
    leader: Pid,
}

impl Group {
    /// Returns the group that the program with this identifier leads
    ///
    /// The value is `None` when the identifier names no process, which the
    /// operating system reports for a program that it has not started.
    pub(crate) fn led_by(identifier: u32) -> Option<Self> {
        let identifier = i32::try_from(identifier).ok()?;

        Pid::from_raw(identifier).map(|leader| Self { leader })
    }

    /// Returns the identifier of the group
    pub(crate) fn id(self) -> Pid {
        self.leader
    }

    /// Kills every process of the group
    ///
    /// A kill that the operating system refuses names a group that has no
    /// process left. Nothing then remains to end, so the refusal carries no
    /// information for the caller.
    pub(crate) fn kill(self) {
        let _ = kill_process_group(self.leader, Signal::KILL);
    }

    /// Asks every process of the group to end, and reports whether the request went out
    ///
    /// The request is `SIGTERM`. A stopped process does not answer a request,
    /// so `SIGCONT` follows it. A program that a user stopped with Ctrl+Z then
    /// answers as well.
    pub(crate) fn request_end(self) -> Request {
        match kill_process_group(self.leader, Signal::TERM) {
            Ok(()) => {
                self.resume();

                Request::Sent
            }
            Err(_) => Request::Unsent,
        }
    }

    /// Continues every stopped process of the group
    ///
    /// A refusal names a group that has no process left, and such a group has
    /// nothing to continue.
    pub(crate) fn resume(self) {
        let _ = kill_process_group(self.leader, Signal::CONT);
    }

    /// Reports whether the program that leads the group has ended
    ///
    /// The check collects no status. The status stays with the operating
    /// system, so the identifier of the group stays reserved, and a wait that
    /// follows still reports the status.
    ///
    /// # Errors
    ///
    /// Returns the error of the operating system when it cannot report the
    /// state of the program.
    pub(crate) fn has_ended(self) -> Result<bool, IoError> {
        let options = WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT;

        let status = waitid(WaitId::Pid(self.leader), options)?;

        Ok(status.is_some())
    }

    /// Returns the signal that stopped the program, when it stopped since the last call
    ///
    /// The call takes the report of the stop. A later call therefore reports
    /// the next stop, and not this stop a second time.
    ///
    /// # Errors
    ///
    /// Returns the error of the operating system when it cannot report the
    /// state of the program.
    pub(crate) fn take_stop(self) -> Result<Option<i32>, IoError> {
        let options = WaitIdOptions::STOPPED | WaitIdOptions::NOHANG;

        let status = waitid(WaitId::Pid(self.leader), options)?;

        Ok(status.and_then(|status| status.stopping_signal()))
    }
}

/// Whether the request to end reached the group
///
/// A stop that asked nothing must not wait for an answer, because no answer
/// can arrive.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum Request {
    /// The group received the request to end
    Sent,

    /// Nothing asked the group to end
    Unsent,
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::missing_panics_doc,
        reason = "an assertion in a test panics by design, and a section on every test tells the reader nothing"
    )]

    use super::*;

    /// Holds the group to the auto traits that the handle of a run needs
    #[test]
    fn group_is_send_sync_and_unpin() {
        fn assert_auto_traits<T: Send + Sync + Unpin>() {}

        assert_auto_traits::<Group>();
    }

    /// Rejects an identifier that no process can have
    #[test]
    fn led_by_with_zero_returns_none() {
        let group = Group::led_by(0);

        assert!(group.is_none());
    }

    /// Holds the request to the auto traits that the handle of a run needs
    #[test]
    fn request_is_send_sync_and_unpin() {
        fn assert_auto_traits<T: Send + Sync + Unpin>() {}

        assert_auto_traits::<Request>();
    }
}
