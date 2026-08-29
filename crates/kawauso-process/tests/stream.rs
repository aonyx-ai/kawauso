//! Tests for the handle of a run through the public API
//!
//! Every test here starts a real program, because the obligations of the
//! handle exist only when a program runs: the line that arrives while the
//! command still runs, the capture that grows with it, and the command that
//! ends with the handle. The shell of the platform is the program that every
//! machine has, so most tests give a command line to it.
//!
//! Some tests need a tool that only a Unix system has, and they run there
//! alone. The project has no Windows runner, so no test pins the behavior of
//! the crate on that platform.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

#[cfg(unix)]
use std::time::Duration;

use kawauso_process::Invocation;
use kawauso_process::Run;
use kawauso_process::error::RunCommandError;
use kawauso_process::run::Stream;
#[cfg(unix)]
use tokio::time::sleep;
#[cfg(unix)]
use tokio::time::timeout;

/// The number of bytes that a command writes to each stream in the test of
/// the concurrent read
///
/// A pipe of the operating system holds a few dozen kilobytes, and the
/// command writes far more than that to both of its streams. A handle that
/// read one stream after the other, or that waited for the end before it
/// read, would never finish this command.
#[cfg(unix)]
const FLOOD: usize = 256 * 1024;

/// Returns every line that the command wrote, in the order in which it
/// arrived
///
/// A test that makes a statement about a whole stream reads it to the end,
/// and the handle reports the lines one at a time.
///
/// # Errors
///
/// Returns the error of the handle when a stream cannot be read.
async fn lines(mut run: Run) -> Result<Vec<(Stream, String)>, RunCommandError> {
    let mut lines = Vec::new();

    while let Some(line) = run.next_line().await? {
        lines.push((line.stream(), line.text().get().to_owned()));
    }

    Ok(lines)
}

/// Returns an invocation that gives the commands to the shell of the platform
///
/// A test needs a command that writes what the test expects, and the shell is
/// the program that every machine has. The separator between two commands
/// differs between the platforms, so the helper takes the commands apart and
/// joins them for the shell that runs them.
fn shell(commands: &[&str]) -> Invocation {
    if cfg!(windows) {
        Invocation::new("cmd").arg("/C").arg(commands.join(" & "))
    } else {
        Invocation::new("sh").arg("-c").arg(commands.join("; "))
    }
}

// A caller that abandons the handle, after a timeout for example, drops it.
// The command writes the file after the drop, so a file that exists at the
// end is a command that outlived its handle.
// process[verify stream.abandonment]
#[cfg(unix)]
#[tokio::test]
async fn handle_that_the_caller_drops_ends_the_command() {
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("marker");
    let command = format!("sleep 1; : > \"{}\"", marker.display());

    drop(shell(&[&command]).start().unwrap());
    sleep(Duration::from_secs(2)).await;

    assert!(!marker.exists());
}

// A command writes bytes, and a byte that is part of no character has no
// place in the text of a line.
// process[verify stream.decode]
#[cfg(unix)]
#[tokio::test]
async fn next_line_with_a_byte_that_is_no_character_replaces_the_byte() {
    let run = shell(&[r"printf '\377\n'"]).start().unwrap();

    let lines = lines(run).await.unwrap();

    assert_eq!(lines, [(Stream::StandardOutput, "\u{fffd}".to_owned())]);
}

// The handle exists for the caller that reads the output while the command
// runs. The command sleeps after it wrote the line, so a handle that reported
// the line at the end of the command would run into the timeout.
// process[verify stream]
#[cfg(unix)]
#[tokio::test]
async fn next_line_with_a_command_that_writes_before_it_ends_reports_the_line() {
    let mut run = shell(&["echo hello", "sleep 5"]).start().unwrap();

    let line = timeout(Duration::from_secs(2), run.next_line())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(line.unwrap().text().get(), "hello");
}

// A program separates its result from its diagnostics, and a caller that
// shows the two apart needs the stream of every line. The order between the
// two streams is the order of the operating system, so the test sorts the
// lines before it compares them.
// process[verify stream.tag]
#[tokio::test]
async fn next_line_with_a_command_that_writes_to_both_streams_names_the_stream_of_each_line() {
    let run = shell(&["echo hello", "echo oops 1>&2"]).start().unwrap();

    let mut lines = lines(run).await.unwrap();
    lines.sort();

    assert_eq!(
        lines,
        [
            (Stream::StandardError, "oops".to_owned()),
            (Stream::StandardOutput, "hello".to_owned()),
        ]
    );
}

// A command can end after its last byte, without the character that ends a
// line. The bytes are output, so the caller gets them as a line.
// process[verify stream]
#[cfg(unix)]
#[tokio::test]
async fn next_line_with_a_last_line_that_has_no_end_reports_the_line() {
    let run = shell(&[r"printf 'hello'"]).start().unwrap();

    let lines = lines(run).await.unwrap();

    assert_eq!(lines, [(Stream::StandardOutput, "hello".to_owned())]);
}

// A command on Windows ends a line with two characters, and a caller that
// compares the text of a line reads the same text on every platform.
// process[verify stream.crlf]
#[cfg(unix)]
#[tokio::test]
async fn next_line_with_a_line_that_ends_with_a_carriage_return_removes_it() {
    let run = shell(&[r"printf 'one\r\ntwo\r\n'"]).start().unwrap();

    let lines = lines(run).await.unwrap();

    assert_eq!(
        lines,
        [
            (Stream::StandardOutput, "one".to_owned()),
            (Stream::StandardOutput, "two".to_owned()),
        ]
    );
}

// process[verify stream]
#[tokio::test]
async fn next_line_with_lines_reports_them_in_the_order_of_the_command() {
    let run = shell(&["echo one", "echo two", "echo three"])
        .start()
        .unwrap();

    let lines = lines(run).await.unwrap();

    assert_eq!(
        lines,
        [
            (Stream::StandardOutput, "one".to_owned()),
            (Stream::StandardOutput, "two".to_owned()),
            (Stream::StandardOutput, "three".to_owned()),
        ]
    );
}

// The lines are for a display, and the capture is the output of the command.
// A byte that no line can show is therefore still in the result.
// process[verify stream.decode]
#[cfg(unix)]
#[tokio::test]
async fn wait_with_a_byte_that_is_no_character_keeps_the_byte_in_the_capture() {
    let run = shell(&[r"printf '\377\n'"]).start().unwrap();

    let execution = run.wait().await.unwrap();

    assert_eq!(execution.stdout().get(), b"\xff\n");
}

// process[verify stream.wait]
#[tokio::test]
async fn wait_with_a_command_that_failed_reports_the_exit_code() {
    let run = shell(&["exit 3"]).start().unwrap();

    let execution = run.wait().await.unwrap();

    assert_eq!(execution.status().code(), Some(3));
}

// A command that fills a pipe stops until a reader empties it, so a handle
// that read one stream after the other would never end this command.
// process[verify stream.wait]
#[cfg(unix)]
#[tokio::test]
async fn wait_with_a_command_that_fills_both_pipes_captures_everything() {
    let run = shell(&[
        &format!("head -c {FLOOD} /dev/zero"),
        &format!("head -c {FLOOD} /dev/zero >&2"),
    ])
    .start()
    .unwrap();

    let execution = run.wait().await.unwrap();

    assert_eq!(
        (
            execution.stdout().get().len(),
            execution.stderr().get().len()
        ),
        (FLOOD, FLOOD)
    );
}

// A caller that wants the result and not the progress uses the handle as
// well, and it gets what a run in one call reports.
// process[verify stream.wait]
#[tokio::test]
async fn wait_without_a_line_that_the_caller_read_returns_the_whole_capture() {
    let run = shell(&["echo hello", "echo oops 1>&2"]).start().unwrap();

    let execution = run.wait().await.unwrap();

    assert_eq!(
        (
            execution.stdout().to_string_lossy().trim(),
            execution.stderr().to_string_lossy().trim()
        ),
        ("hello", "oops")
    );
}
