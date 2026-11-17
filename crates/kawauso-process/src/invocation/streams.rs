//! The standard streams that an interactive run connects to a command

use std::process::Stdio;

/// The standard streams that an interactive run connects to a command
///
/// A run that captures the output of a command pipes its streams. A program
/// that takes over the terminal needs other streams. An editor reads the keys
/// of the user and draws on the terminal, so it needs the streams of the
/// terminal. The caller names each stream here, and an interactive run
/// connects them.
///
/// Each stream that the caller does not name is the stream of the process
/// that starts the command. A stream accepts what [`Stdio`] accepts, such as
/// a file or a stream of another command.
///
/// # Examples
///
/// ```
/// use std::process::Stdio;
///
/// use kawauso_process::invocation::Streams;
///
/// let streams = Streams::inherit().stdout(Stdio::null());
/// ```
#[derive(Debug)]
pub struct Streams {
    /// The stream from which the command reads
    stdin: Stdio,

    /// The stream to which the command writes its output
    stdout: Stdio,

    /// The stream to which the command writes its diagnostics
    stderr: Stdio,
}

impl Streams {
    /// Returns the streams of the process that starts the command
    ///
    /// The command then reads and writes where this process reads and writes.
    /// An editor that this process starts from a terminal therefore uses that
    /// terminal.
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_process::invocation::Streams;
    ///
    /// let streams = Streams::inherit();
    /// ```
    pub fn inherit() -> Self {
        Self {
            stdin: Stdio::inherit(),
            stdout: Stdio::inherit(),
            stderr: Stdio::inherit(),
        }
    }

    /// Returns the three streams, in the order of their numbers
    ///
    /// A start takes the streams apart, because a command takes each stream
    /// in a call of its own.
    pub(crate) fn into_parts(self) -> (Stdio, Stdio, Stdio) {
        (self.stdin, self.stdout, self.stderr)
    }

    /// Sets the stream to which the command writes its diagnostics
    ///
    /// # Examples
    ///
    /// ```
    /// use std::process::Stdio;
    ///
    /// use kawauso_process::invocation::Streams;
    ///
    /// let streams = Streams::inherit().stderr(Stdio::null());
    /// ```
    pub fn stderr(mut self, stream: impl Into<Stdio>) -> Self {
        self.stderr = stream.into();

        self
    }

    /// Sets the stream from which the command reads
    ///
    /// # Examples
    ///
    /// ```
    /// use std::process::Stdio;
    ///
    /// use kawauso_process::invocation::Streams;
    ///
    /// let streams = Streams::inherit().stdin(Stdio::null());
    /// ```
    pub fn stdin(mut self, stream: impl Into<Stdio>) -> Self {
        self.stdin = stream.into();

        self
    }

    /// Sets the stream to which the command writes its output
    ///
    /// A caller that keeps its own standard output for the result of its work
    /// sends the output of the program elsewhere. A program on the terminal
    /// then still shows, and the result stays clean.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::process::Stdio;
    ///
    /// use kawauso_process::invocation::Streams;
    ///
    /// let streams = Streams::inherit().stdout(Stdio::null());
    /// ```
    pub fn stdout(mut self, stream: impl Into<Stdio>) -> Self {
        self.stdout = stream.into();

        self
    }
}

/// Returns the streams of the process that starts the command
impl Default for Streams {
    fn default() -> Self {
        Self::inherit()
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::missing_panics_doc,
        reason = "an assertion in a test panics by design, and a section on every test tells the reader nothing"
    )]

    use super::*;

    /// Holds the type to the auto traits that a caller needs to move it into a task
    #[test]
    fn streams_is_send_sync_and_unpin() {
        fn assert_auto_traits<T: Send + Sync + Unpin>() {}

        assert_auto_traits::<Streams>();
    }
}
