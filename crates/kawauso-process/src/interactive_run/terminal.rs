//! The terminal that an interactive run lends to its program

use std::fs::File;
use std::fs::OpenOptions;
use std::io::Error as IoError;

use nix::sys::pthread::pthread_kill;
use nix::sys::pthread::pthread_self;
use nix::sys::signal::SigSet;
use nix::sys::signal::SigmaskHow;
use nix::sys::signal::Signal as NixSignal;
use rustix::process::Pid;
use rustix::process::Signal;
use rustix::process::getpgrp;
use rustix::termios::tcgetpgrp;
use rustix::termios::tcsetpgrp;

use super::group::Group;

/// The controlling terminal of this process, while a program holds its foreground
///
/// A terminal sends the keys of the user to one process group, its foreground
/// group. The signals of Ctrl+C, Ctrl+\\, and Ctrl+Z go to that group as
/// well. A program in another group that reads the terminal stops. An
/// interactive run therefore lends the foreground to the group of its
/// program. It takes the foreground back when the program ended.
///
/// The value exists only when this process was in the foreground group at the
/// start. A process in the background, and a process without a terminal, lend
/// nothing. Such a process also takes nothing back.
#[derive(Debug)]
pub(crate) struct Terminal {
    /// The device of the controlling terminal
    device: File,

    /// The process group of this process
    owner: Pid,
}

impl Terminal {
    /// Returns the controlling terminal when this process is in its foreground group
    ///
    /// The value is `None` when this process has no controlling terminal, or
    /// when another group holds its foreground.
    pub(crate) fn claim() -> Option<Self> {
        let device = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")
            .ok()?;
        let owner = getpgrp();

        let holder = tcgetpgrp(&device).ok()?;

        (holder == owner).then_some(Self { device, owner })
    }

    /// Reports whether the group holds the foreground of the terminal
    fn is_held_by(&self, group: Pid) -> bool {
        tcgetpgrp(&self.device).is_ok_and(|holder| holder == group)
    }

    /// Makes the group of the program the foreground group of the terminal
    ///
    /// # Errors
    ///
    /// Returns the error of the operating system when the terminal refuses
    /// the group.
    pub(crate) fn lend(&self, group: Group) -> Result<(), IoError> {
        hand_over(&self.device, group.id())
    }

    /// Follows a program that stopped, and continues it
    ///
    /// A `SIGTTIN` or `SIGTTOU` while the program holds the foreground is a
    /// read or a write before [`lend`](Self::lend) ran. The program continues, and
    /// nothing else happens.
    ///
    /// Any other stop is a stop of the job. The call takes the terminal back,
    /// and it stops this process with the same signal. The shell of the user
    /// then shows the job as stopped. When the shell continues this process,
    /// the call lends the terminal again if this process is in the
    /// foreground, and it continues the program.
    pub(crate) fn follow(&self, group: Group, signal: i32) {
        let lent = self.is_held_by(group.id());
        let early = signal == Signal::TTIN.as_raw() || signal == Signal::TTOU.as_raw();

        if lent && early {
            group.resume();

            return;
        }

        if lent {
            let _ = hand_over(&self.device, self.owner);
        }

        if let Ok(signal) = NixSignal::try_from(signal) {
            let _ = pthread_kill(pthread_self(), signal);
        }

        if self.is_held_by(self.owner) {
            let _ = hand_over(&self.device, group.id());
        }

        group.resume();
    }

    /// Makes the group of this process the foreground group again
    ///
    /// The call takes back only a terminal that the group of the program
    /// holds. A shell that moved this process into the background keeps the
    /// terminal.
    pub(crate) fn reclaim(&self, group: Group) {
        if self.is_held_by(group.id()) {
            let _ = hand_over(&self.device, self.owner);
        }
    }
}

/// Makes a group the foreground group of the terminal
///
/// A process outside the foreground group that makes this change receives
/// `SIGTTOU` and stops, unless it blocks that signal. The call blocks the
/// signal in this thread for the change, and it restores the mask of the
/// thread after it. No disposition of the process changes.
///
/// # Errors
///
/// Returns the error of the operating system when the thread cannot block
/// the signal, or when the terminal refuses the group.
fn hand_over(device: &File, group: Pid) -> Result<(), IoError> {
    let mut blocked = SigSet::empty();
    blocked.add(NixSignal::SIGTTOU);

    let previous = blocked.thread_swap_mask(SigmaskHow::SIG_BLOCK)?;
    let handed = tcsetpgrp(device, group).map_err(IoError::from);
    let restored = previous.thread_set_mask().map_err(IoError::from);

    handed.and(restored)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::missing_panics_doc,
        reason = "an assertion in a test panics by design, and a section on every test tells the reader nothing"
    )]

    use super::*;

    /// Holds the terminal to the auto traits that the handle of a run needs
    #[test]
    fn terminal_is_send_sync_and_unpin() {
        fn assert_auto_traits<T: Send + Sync + Unpin>() {}

        assert_auto_traits::<Terminal>();
    }
}
