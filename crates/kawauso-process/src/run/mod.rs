//! The run of a command that the caller observes while it runs
//!
//! A run in one call reports after the command ended. An application that
//! shows the progress of a command, or that turns the output of a command
//! into events of its own, needs the output earlier. This module holds the
//! handle that gives the output to such a caller, one line at a time, and the
//! types of a line.
//!
//! The handle reports the same result as a run in one call, so an application
//! that moves from the one form to the other keeps its handling of the
//! result.

pub mod line;
pub mod stream;
pub mod text;

use std::collections::VecDeque;
use std::future::poll_fn;
use std::io::Error as IoError;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
use std::time::Instant;

use tokio::io::AsyncRead;
use tokio::io::ReadBuf;
use tokio::process::Child;
use tokio::process::ChildStderr;
use tokio::process::ChildStdout;

pub use self::line::Line;
pub use self::stream::Stream;
pub use self::text::Text;
use crate::error::RunCommandError;
use crate::execution::Execution;
use crate::execution::Output;
use crate::invocation::Invocation;
use crate::process_id::ProcessId;

/// The number of bytes that one read takes from a stream
///
/// The value is a compromise. A small buffer reads a stream in many steps,
/// and a large one holds memory that most commands never fill, because a
/// command writes lines and not blocks.
const CHUNK: usize = 8 * 1024;

/// The run of a command that the caller observes while it runs
///
/// The handle starts with the command and ends with the result of the run. It
/// gives the output of the command to the caller as lines, in the order in
/// which the lines arrive, and it collects the same capture that a run in one
/// call collects. A caller that reads no line at all therefore still gets
/// every byte that the command wrote.
///
/// The handle owns the command. A caller that drops the handle ends the
/// command with it, so a timeout, a `select`, and a scheduler that abandons a
/// run leave no process behind.
///
/// # Examples
///
/// ```no_run
/// use kawauso_process::Invocation;
///
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut run = Invocation::new("cargo").arg("build").start()?;
///
/// while let Some(line) = run.next_line().await? {
///     println!("[{}] {line}", line.stream());
/// }
///
/// let execution = run.wait().await?.require_success()?;
///
/// println!("{}", execution.duration().as_secs());
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Run {
    /// The command that runs
    ///
    /// The handle carries it so that a log line and an error message can name
    /// the command without the caller holding it a second time.
    invocation: Invocation,

    /// The command itself
    child: Child,

    /// The standard output of the command
    stdout: Reader<ChildStdout>,

    /// The standard error of the command
    stderr: Reader<ChildStderr>,

    /// The lines that the readers produced and the caller has not taken
    ///
    /// One read of a stream can produce more than one line, and the caller
    /// takes one line per call. The queue holds the rest, in the order in
    /// which the lines arrived.
    lines: VecDeque<Line>,

    /// The moment at which the command started
    started: Instant,
}

impl Run {
    /// Takes the streams of a command that started and builds the handle
    ///
    /// The constructor is not public, because it takes the command as the
    /// asynchronous runtime represents it, and no public signature of this
    /// crate names a type of that runtime. [`start`][start] is the way to a
    /// handle.
    ///
    /// [start]: Invocation::start
    pub(crate) fn new(invocation: Invocation, mut child: Child, started: Instant) -> Self {
        // The command started with a pipe on each stream, so both are here.
        // A reader without a stream reports the end of its stream at once,
        // which leaves the caller with the streams that it can read.
        let stdout = Reader::new(Stream::StandardOutput, child.stdout.take());
        let stderr = Reader::new(Stream::StandardError, child.stderr.take());

        Self {
            invocation,
            child,
            stdout,
            stderr,
            lines: VecDeque::new(),
            started,
        }
    }

    /// Returns the identifier that the operating system gave the command
    ///
    /// An application that reports which program it started, or that gives
    /// the command to a tool of the platform, reads the identifier here. The
    /// handle holds the command, so the value is there for the whole run.
    ///
    /// The value is `None` after the crate collected the status of the
    /// command. [`wait`][wait] collects the status, and it takes the handle,
    /// so a caller that holds a handle has the identifier.
    ///
    /// [wait]: Run::wait
    // process[impl stream.identity]
    pub fn id(&self) -> Option<ProcessId> {
        self.child.id().map(ProcessId::from)
    }

    /// Returns the command that runs
    pub fn invocation(&self) -> &Invocation {
        &self.invocation
    }

    /// Returns the next line of the output of the command
    ///
    /// The method waits until the command writes a line to one of its
    /// streams, and it returns the lines in the order in which they arrive.
    /// It returns `None` when the command closed both of its streams, which a
    /// command does when it ends. A command that writes a last line without
    /// the characters that end a line still reports that line.
    ///
    /// Every line that the method returns stays in the capture of the result
    /// as well, so a caller that shows the output while the command runs and
    /// then reports the whole output reads both from the same run.
    ///
    /// # Errors
    ///
    /// Returns [`IncompleteRun`][incomplete] when a stream of the command
    /// cannot be read.
    ///
    /// # Cancel safety
    ///
    /// The method is cancel safe. A caller often waits for a line and for
    /// another event, such as a timeout or another branch of a `select`. The
    /// caller drops the future of the call when the other event occurs first,
    /// and the call loses no output. The bytes that the call took stay with
    /// the handle, and the call that follows reports the line that they
    /// belong to.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use kawauso_process::Invocation;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut run = Invocation::new("cargo").arg("test").start()?;
    ///
    /// while let Some(line) = run.next_line().await? {
    ///     println!("{line}");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [incomplete]: RunCommandError::IncompleteRun
    // process[impl stream]
    // process[impl stream.cancellation]
    pub async fn next_line(&mut self) -> Result<Option<Line>, RunCommandError> {
        loop {
            if let Some(line) = self.lines.pop_front() {
                return Ok(Some(line));
            }

            if self.stdout.is_closed() && self.stderr.is_closed() {
                return Ok(None);
            }

            // A poll that takes bytes stores them in the reader and in the
            // queue before it returns. The await point below is a poll that
            // took nothing. Every state of the read therefore lives in the
            // handle, and a future that the caller drops takes none of it.
            let read = {
                let Self {
                    stdout,
                    stderr,
                    lines,
                    ..
                } = self;

                poll_fn(|context| poll_streams(stdout, stderr, lines, context)).await
            };

            read.map_err(|source| RunCommandError::IncompleteRun {
                invocation: self.invocation.clone(),
                source,
            })?;
        }
    }

    /// Waits for the end of the command and reports the result of the run
    ///
    /// The method reads what the command still writes, waits for the end of
    /// the command, and reports what a run in one call reports: the exit
    /// status, the capture of both streams, and the time that the run took.
    /// The capture holds everything that the command wrote, whether or not
    /// the caller read a line.
    ///
    /// A command that ends without success is no failure of this method. The
    /// status travels in the result, and
    /// [`require_success`][require-success] turns a status that is not a
    /// success into an error.
    ///
    /// # Errors
    ///
    /// Returns [`IncompleteRun`][incomplete] when a stream of the command
    /// cannot be read, or when the end of the command cannot be waited for.
    ///
    /// # Cancel safety
    ///
    /// The method is not cancel safe. It takes the handle, so a caller that
    /// drops the future of a call ends the command. The caller loses the
    /// result of the run, and the capture of both streams with it.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use kawauso_process::Invocation;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let run = Invocation::new("cargo").arg("build").start()?;
    ///
    /// let execution = run.wait().await?;
    ///
    /// println!("{}", execution.status());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [incomplete]: RunCommandError::IncompleteRun
    /// [require-success]: Execution::require_success
    // process[impl stream.wait]
    pub async fn wait(mut self) -> Result<Execution, RunCommandError> {
        // The capture grows with every read, so the drain fills it whether or
        // not the caller took a single line. The lines that no one took end
        // here.
        while self.next_line().await?.is_some() {}

        // The wait collects the status of the command, and the identifier of
        // a command that ended is gone. The result carries the identifier,
        // so the handle reads it before the wait.
        // process[impl run.identity]
        let id = self.id();

        let status = self
            .child
            .wait()
            .await
            .map_err(|source| RunCommandError::IncompleteRun {
                invocation: self.invocation.clone(),
                source,
            })?;

        Ok(Execution::new(
            self.invocation,
            id,
            status,
            Output::new(self.stdout.capture),
            Output::new(self.stderr.capture),
            self.started.elapsed(),
        ))
    }
}

/// Reads the streams of a command until one of them produced something
///
/// The function polls both streams on every call, so a command that fills the
/// pipe of one stream while it is quiet on the other never stops the run. A
/// stream that produced nothing has registered its interest with the runtime,
/// which wakes the caller when the stream has bytes.
///
/// The function returns `Pending` when both streams have nothing. A stream
/// that reached its end reports nothing forever, so the caller stops before
/// it calls this function with two such streams.
///
/// # Errors
///
/// Returns the error of the stream that could not be read.
fn poll_streams(
    stdout: &mut Reader<ChildStdout>,
    stderr: &mut Reader<ChildStderr>,
    lines: &mut VecDeque<Line>,
    context: &mut Context<'_>,
) -> Poll<Result<(), IoError>> {
    let mut produced = false;

    match stdout.poll(context, lines) {
        Poll::Ready(Err(error)) => return Poll::Ready(Err(error)),
        Poll::Ready(Ok(())) => produced = true,
        Poll::Pending => {}
    }

    match stderr.poll(context, lines) {
        Poll::Ready(Err(error)) => return Poll::Ready(Err(error)),
        Poll::Ready(Ok(())) => produced = true,
        Poll::Pending => {}
    }

    if produced {
        Poll::Ready(Ok(()))
    } else {
        Poll::Pending
    }
}

/// One stream of a command, with the capture and the line that it builds
///
/// The reader keeps three things apart. The capture holds every byte that the
/// stream carried, because the result of the run reports the output as the
/// command wrote it. The pending bytes hold the line that the command has
/// started and not ended. The stream tells a line where it came from.
#[derive(Debug)]
struct Reader<S> {
    /// The stream that this reader reads
    stream: Stream,

    /// The stream itself, and `None` after the stream reached its end
    source: Option<S>,

    /// The bytes of a line that the command has not ended
    pending: Vec<u8>,

    /// The number of pending bytes in which no end of a line was found
    ///
    /// A read looks for the end of a line in the bytes that it added, and not
    /// in the bytes that it has looked at before. A command that writes a
    /// long line therefore reads that line once, and not once per read.
    searched: usize,

    /// Every byte that the stream carried
    capture: Vec<u8>,
}

impl<S: AsyncRead + Unpin> Reader<S> {
    /// Creates the reader of one stream of a command
    fn new(stream: Stream, source: Option<S>) -> Self {
        Self {
            stream,
            source,
            pending: Vec::new(),
            searched: 0,
            capture: Vec::new(),
        }
    }

    /// Returns whether the stream reached its end
    fn is_closed(&self) -> bool {
        self.source.is_none()
    }

    /// Builds one line from the bytes that the command wrote
    ///
    /// A byte that is part of no valid character becomes the replacement
    /// character, because a line is for a display and a display takes text.
    /// The capture keeps the byte, so nothing that the command wrote is lost.
    // process[impl stream.decode]
    fn line(&self, bytes: &[u8]) -> Line {
        Line::new(self.stream, String::from_utf8_lossy(bytes).into_owned())
    }

    /// Reads what the stream has and builds the lines that it completed
    ///
    /// The function returns `Ready` when it read bytes or when the stream
    /// reached its end, and `Pending` when the stream has nothing yet. A
    /// stream that reached its end reports `Pending`, because it has nothing
    /// to report ever again.
    ///
    /// # Errors
    ///
    /// Returns the error of a stream that cannot be read.
    fn poll(
        &mut self,
        context: &mut Context<'_>,
        lines: &mut VecDeque<Line>,
    ) -> Poll<Result<(), IoError>> {
        let Some(source) = self.source.as_mut() else {
            return Poll::Pending;
        };

        let mut bytes = [0; CHUNK];
        let mut buffer = ReadBuf::new(&mut bytes);

        match Pin::new(source).poll_read(context, &mut buffer) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Err(error)) => return Poll::Ready(Err(error)),
            Poll::Ready(Ok(())) => {}
        }

        let read = buffer.filled();

        // A read of no bytes is the end of the stream. The command wrote what
        // it wrote, and a line that it started without an end is a line as
        // well, because nothing follows it.
        if read.is_empty() {
            self.source = None;

            if !self.pending.is_empty() {
                let bytes = std::mem::take(&mut self.pending);

                self.searched = 0;
                lines.push_back(self.line(&bytes));
            }

            return Poll::Ready(Ok(()));
        }

        self.capture.extend_from_slice(read);
        self.pending.extend_from_slice(read);
        self.split(lines);

        Poll::Ready(Ok(()))
    }

    /// Takes every line that the command ended out of the pending bytes
    ///
    /// A command on Windows ends a line with two characters, and a command on
    /// every other platform with one. The line carries neither of them, so a
    /// caller that shows the line, or that compares it, reads the same text
    /// on every platform.
    // process[impl stream.crlf]
    fn split(&mut self, lines: &mut VecDeque<Line>) {
        while let Some(offset) = self.pending[self.searched..]
            .iter()
            .position(|byte| *byte == b'\n')
        {
            let end = self.searched + offset;
            let mut bytes: Vec<u8> = self.pending.drain(..=end).collect();

            bytes.pop();

            if bytes.last() == Some(&b'\r') {
                bytes.pop();
            }

            self.searched = 0;
            lines.push_back(self.line(&bytes));
        }

        self.searched = self.pending.len();
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every
    // test would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    // A caller keeps the handle in a task that another thread drives. This
    // test holds the type to the auto traits that make this possible, because
    // a private field of a later version could take them away without a word
    // from the compiler.
    #[test]
    fn run_is_send_and_sync() {
        fn assert_send_and_sync<T: Send + Sync>() {}

        assert_send_and_sync::<Run>();
    }
}
