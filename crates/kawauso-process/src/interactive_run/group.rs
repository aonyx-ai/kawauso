//! The process group of an interactive run

use rustix::process::Pid;
use rustix::process::Signal;
use rustix::process::kill_process_group;

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

    /// Kills every process of the group
    ///
    /// A kill that the operating system refuses names a group that has no
    /// process left. Nothing then remains to end, so the refusal carries no
    /// information for the caller.
    pub(crate) fn kill(self) {
        let _ = kill_process_group(self.leader, Signal::KILL);
    }
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
}
