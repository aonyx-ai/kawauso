//! Tests for the stop of a run through the public API
//!
//! Every test here starts a real program, because the obligations of the stop
//! exist only when a program runs: the request that arrives, the time that
//! the program has, and the kill that ends a program which ignores the
//! request. A shell script is the program, because it can install a handler
//! for the signal in one line.
//!
//! Only a Unix system has the request that a stop sends, so every test runs
//! there alone. The project has no Windows runner, and no test pins the
//! behavior of the crate on that platform.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]
// Only a Unix system has the request that a stop sends, so the whole file
// builds there alone.
#![cfg(unix)]

use std::os::unix::process::ExitStatusExt;
use std::time::Duration;

use kawauso_process::Invocation;
use kawauso_process::Run;
use kawauso_process::error::RunCommandError;
use tokio::time::timeout;

/// The time in which a stop has to report
///
/// The time is far longer than a stop needs, and far shorter than the time
/// that a program of the command holds the streams. A stop that follows such
/// a program therefore fails the test instead of passing slowly.
const BOUND: Duration = Duration::from_secs(5);

/// The number of bytes that the command writes to each stream after it
/// received the request to end
///
/// A pipe of the operating system holds a few dozen kilobytes, and the
/// command writes far more than that to both of its streams. A stop that
/// stopped reading while it waits would never see the end of this command.
const FLOOD: usize = 256 * 1024;

/// The time that a command gets to end when the test expects it to end
///
/// The time is long enough for a machine under load, because a stop that
/// killed a command which was about to end would make the test report a
/// failure that the crate does not have.
const GRACE: Duration = Duration::from_secs(30);

/// The number of seconds that a program of the command holds the streams
///
/// The value is far longer than the time in which a stop has to report. A
/// stop that follows the streams instead of the command therefore runs into
/// the bound of the test.
const HOLD: u64 = 60;

/// Starts a script and reads its output until the script is ready
///
/// A request that arrives before the script installed its handler kills the
/// script, and the test then makes a statement about the wrong thing. Every
/// script therefore writes `ready` when its handler is in place, and the
/// stop follows that line.
///
/// # Errors
///
/// Returns the error of the handle when the script does not start, or when a
/// stream of the script cannot be read.
async fn ready(script: &str) -> Result<Run, RunCommandError> {
    let mut run = Invocation::new("sh").arg("-c").arg(script).start()?;

    while let Some(line) = run.next_line().await? {
        if line.text().get() == "ready" {
            break;
        }
    }

    Ok(run)
}

/// Returns a script that waits for the request to end
///
/// The script installs the handler that the test needs, reports that it is
/// ready, and then waits. The sleep is short, because a shell runs a handler
/// after the command that runs when the signal arrives.
fn script(handler: &str) -> String {
    format!("trap '{handler}' TERM; echo ready; while :; do sleep 0.05; done")
}

/// Returns a script that starts a program of its own and outlives itself
///
/// The program that the script starts inherits both streams of the script,
/// and it holds them open for as long as it runs. The script therefore ends
/// while its streams stay open, which is the shape that a build tool has when
/// it starts a compiler.
fn script_that_starts_a_program(handler: &str) -> String {
    format!("trap '{handler}' TERM; sleep {HOLD} & echo ready; wait")
}

// A caller waits for the end of a command, and for a token that cancels the
// work. The caller drops the wait when the token comes first. The handle stays
// with that caller, so it asks the command to end in good order. A wait that
// takes the handle leaves that caller with the kill of a drop.
// process[verify stream.end]
#[tokio::test]
async fn stop_after_a_dropped_wait_for_the_end_asks_the_command_to_end() {
    let mut run = ready(&script("echo stopped; exit 0")).await.unwrap();
    let dropped = timeout(Duration::from_millis(100), run.wait_for_end()).await;

    let execution = run.stop(GRACE).await.unwrap();

    assert_eq!(
        (
            dropped.is_err(),
            execution.stdout().to_string_lossy().contains("stopped")
        ),
        (true, true)
    );
}

// A caller can wait for the end of a command and then stop it. The command
// ended, and the crate collected its status, so the stop asks nothing and
// kills nothing. It reports the result of the run that ended.
// process[verify stream.end]
#[tokio::test]
async fn stop_after_a_wait_for_the_end_reports_the_status() {
    let mut run = Invocation::new("sh")
        .arg("-c")
        .arg("exit 7")
        .start()
        .unwrap();
    run.wait_for_end().await.unwrap();

    let execution = run.stop(GRACE).await.unwrap();

    assert_eq!(execution.status().code(), Some(7));
}

// A program that answers the request writes a line that a killed program
// never writes. The line in the capture is therefore the request that
// arrived.
// process[verify stop.request]
#[tokio::test]
async fn stop_asks_the_command_to_end() {
    let run = ready(&script("echo stopped; exit 0")).await.unwrap();

    let execution = run.stop(GRACE).await.unwrap();

    assert!(execution.stdout().to_string_lossy().contains("stopped"));
}

// A program that the command started holds the streams of the command open
// long after the command ended. The stop follows the command and not the
// streams, so it reports while that program still runs.
// process[verify stop.bound]
#[tokio::test]
async fn stop_ends_while_a_program_of_the_command_holds_the_streams() {
    let run = ready(&script_that_starts_a_program("exit 0"))
        .await
        .unwrap();

    let stopped = timeout(BOUND, run.stop(GRACE)).await;

    assert!(stopped.is_ok());
}

// A command that ignores the request runs until something ends it, and the
// kill is what ends it. The operating system names the signal of a killed
// program in its status.
// process[verify stop.kill]
#[tokio::test]
async fn stop_kills_a_command_that_ignores_the_request() {
    let run = ready(&script("")).await.unwrap();

    let execution = run.stop(Duration::from_millis(100)).await.unwrap();

    assert_eq!(execution.status().signal(), Some(9));
}

// A command that answers the request ends with a status of its own. A stop
// that killed it before the time was over would report the signal instead.
// process[verify stop.grace]
#[tokio::test]
async fn stop_lets_a_command_that_answers_the_request_end() {
    let run = ready(&script("exit 3")).await.unwrap();

    let execution = run.stop(GRACE).await.unwrap();

    assert_eq!(execution.status().code(), Some(3));
}

// A command that fills a pipe stops until a reader empties it. A stop that
// waited without a read would therefore wait for a command that cannot end,
// and the kill after the time would cut the output.
// process[verify stop.grace]
#[tokio::test]
async fn stop_reads_the_streams_of_a_command_that_fills_both_pipes() {
    let handler = format!("head -c {FLOOD} /dev/zero; head -c {FLOOD} /dev/zero >&2; exit 0");
    let run = ready(&script(&handler)).await.unwrap();

    let execution = run.stop(GRACE).await.unwrap();

    assert_eq!(
        (
            execution.stdout().get().len(),
            execution.stderr().get().len()
        ),
        (FLOOD + "ready\n".len(), FLOOD)
    );
}

// A stop reports what a wait reports, so the output that the command wrote
// before the request is in the result. The caller that stops a command reads
// it there.
// process[verify stop]
#[tokio::test]
async fn stop_reports_the_capture_of_the_run() {
    let run = ready(&script("exit 0")).await.unwrap();

    let execution = run.stop(GRACE).await.unwrap();

    assert_eq!(execution.stdout().to_string_lossy(), "ready\n");
}

// A command that answers the request writes before it ends, and those bytes
// are in the pipe when the command ends. The stop takes what the pipe holds,
// so the line is in the capture although nothing closed the stream.
// process[verify stop.bound]
#[tokio::test]
async fn stop_takes_the_last_output_of_a_command_whose_streams_stay_open() {
    let run = ready(&script_that_starts_a_program("echo stopped; exit 0"))
        .await
        .unwrap();

    let execution = timeout(BOUND, run.stop(GRACE)).await.unwrap().unwrap();

    assert!(execution.stdout().to_string_lossy().contains("stopped"));
}

// A command can end before the caller stops it. Nothing runs then, and the
// stop reports the result of the run that ended. The test reads both streams
// to their end first, because a command closes them when it ends.
// process[verify stop]
#[tokio::test]
async fn stop_of_a_command_that_already_ended_reports_its_status() {
    let mut run = Invocation::new("sh")
        .arg("-c")
        .arg("exit 7")
        .start()
        .unwrap();
    while run.next_line().await.unwrap().is_some() {}

    let execution = run.stop(GRACE).await.unwrap();

    assert_eq!(execution.status().code(), Some(7));
}
