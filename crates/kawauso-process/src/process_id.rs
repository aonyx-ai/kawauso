//! The identifier that the operating system gives a command

use typed_fields::number;

number!(
    /// The identifier that the operating system gives a command
    ///
    /// The operating system gives an identifier to each command that it
    /// starts, and a tool of the platform names a command by this value. An
    /// application that writes the identifier to a log, or that gives it to
    /// such a tool, reads the value from a run or from its result.
    ///
    /// The identifier names the command while the command runs. The
    /// operating system can give the value to another command after the
    /// command ended, so a caller that keeps the value holds a name and not
    /// a command.
    ProcessId,
    u32
);
