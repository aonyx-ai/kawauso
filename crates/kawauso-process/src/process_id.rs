//! The identifier that the operating system gives a command

use typed_fields::number;

number!(
    /// The identifier that the operating system gives a command
    ///
    /// The operating system gives an identifier to each command that it
    /// starts, and a tool of the platform names a command by this value. An application reads
    /// the value from a run or from its result. It does so to write the
    /// identifier to a log or to give it to such a tool.
    ///
    /// The identifier names the command while the command runs. The operating system
    /// can give the value to another command after the command ended. A caller
    /// that keeps the value therefore holds a name and not a command.
    ProcessId,
    u32
);
