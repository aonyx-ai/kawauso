//! Tests for the interactive run through the public API
//!
//! Every test starts a real program, because the obligations of the run exist
//! only when a program runs. They cover the streams, the status, the process
//! group, and the stop.

#![allow(
    clippy::missing_panics_doc,
    reason = "an assertion in a test panics by design, and a section on every test tells the reader nothing"
)]

#[cfg(unix)]
use std::error::Error;
#[cfg(unix)]
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

#[cfg(unix)]
use kawauso_process::InteractiveRun;
use kawauso_process::Invocation;
use kawauso_process::error::RunCommandError;
use kawauso_process::invocation::Streams;
#[cfg(unix)]
use rustix::process::Pid;
#[cfg(unix)]
use rustix::process::Signal;
#[cfg(unix)]
use rustix::process::getpgid;
#[cfg(unix)]
use rustix::process::kill_process;
#[cfg(unix)]
use rustix::process::test_kill_process;
#[cfg(unix)]
use tokio::time::error::Elapsed;
#[cfg(unix)]
use tokio::time::sleep;
#[cfg(unix)]
use tokio::time::timeout;

/// The time in which a program in a test has to reach the state that the test waits for
#[cfg(unix)]
const BOUND: Duration = Duration::from_secs(20);

/// The time that a program gets to end when the test expects it to end
#[cfg(unix)]
const GRACE: Duration = Duration::from_secs(30);

/// Returns the streams of a test, which connect the program to nothing
fn quiet() -> Streams {
    Streams::inherit()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
}

/// Starts a shell script and waits until the script reports that it is ready
///
/// A request that arrives before the script installed its handler kills the
/// script. Every script therefore replaces `READY` with a command that
/// creates a file, at the point where its handler is in place.
///
/// # Errors
///
/// Returns an error when the script does not start or does not report in time.
#[cfg(unix)]
async fn ready(directory: &Path, script: &str) -> Result<InteractiveRun, Box<dyn Error>> {
    let ready = directory.join("ready");
    let script = script.replace("READY", &format!("touch '{}'", ready.display()));
    let run = sh(&script).start_interactive(quiet())?;

    wait_for(|| ready.exists()).await?;

    Ok(run)
}

/// Returns an invocation that gives the script to the shell of a Unix system
#[cfg(unix)]
fn sh(script: &str) -> Invocation {
    Invocation::new("sh").arg("-c").arg(script)
}

/// Returns an invocation that gives the command to the shell of the platform
fn shell(command: &str) -> Invocation {
    if cfg!(windows) {
        Invocation::new("cmd").arg("/C").arg(command)
    } else {
        Invocation::new("sh").arg("-c").arg(command)
    }
}

/// Returns a script that waits for a signal and runs the handler when it arrives
#[cfg(unix)]
fn spin(handler: &str) -> String {
    format!("trap '{handler}' TERM; READY; while :; do sleep 0.05; done")
}

/// Waits until the condition holds
///
/// # Errors
///
/// Returns an error when the condition does not hold in time.
#[cfg(unix)]
async fn wait_for(condition: impl Fn() -> bool) -> Result<(), Elapsed> {
    timeout(BOUND, async {
        while !condition() {
            sleep(Duration::from_millis(20)).await;
        }
    })
    .await
}

/// Waits until no process has the identifier that the file holds
///
/// # Errors
///
/// Returns an error when the file holds no identifier, or when the process
/// does not end in time.
#[cfg(unix)]
async fn wait_for_the_end_of(file: &Path) -> Result<(), Box<dyn Error>> {
    let identifier: i32 = std::fs::read_to_string(file)?.trim().parse()?;
    let process = Pid::from_raw(identifier).ok_or("the file holds no identifier")?;

    wait_for(|| test_kill_process(process).is_err()).await?;

    Ok(())
}

/// A drop kills a program that the program started, because the kill reaches the group
#[cfg(unix)]
#[tokio::test]
async fn drop_kills_a_program_that_the_program_started() {
    let directory = tempfile::tempdir().unwrap();
    let started = directory.path().join("started");
    let script = format!("sleep 300 & echo $! > '{}'; READY; wait", started.display());
    let run = ready(directory.path(), &script).await.unwrap();

    drop(run);

    wait_for_the_end_of(&started).await.unwrap();
}

/// The program leads a group of its own, so a signal to the group reaches its descendants
#[cfg(unix)]
#[tokio::test]
async fn start_interactive_puts_the_program_into_a_group_of_its_own() {
    let run = sh("sleep 300").start_interactive(quiet()).unwrap();
    let process = Pid::from_raw(i32::try_from(run.id().unwrap().get()).unwrap()).unwrap();

    let group = getpgid(Some(process)).unwrap();

    assert_eq!(group, process);
}

/// The program reads the standard input that the caller names
#[cfg(unix)]
#[tokio::test]
async fn start_interactive_reads_the_standard_input_of_the_caller() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("input");
    let output = directory.path().join("output");
    std::fs::write(&input, "hello\n").unwrap();
    let streams = quiet()
        .stdin(std::fs::File::open(&input).unwrap())
        .stdout(std::fs::File::create(&output).unwrap());

    sh("cat")
        .start_interactive(streams)
        .unwrap()
        .wait()
        .await
        .unwrap();

    assert_eq!(std::fs::read_to_string(&output).unwrap(), "hello\n");
}

/// A program that does not exist does not start, and the error says so
#[tokio::test]
async fn start_interactive_with_a_missing_program_returns_an_error() {
    let invocation = Invocation::new("kawauso-no-such-program");

    let error = invocation.start_interactive(quiet()).unwrap_err();

    assert!(matches!(error, RunCommandError::UnstartableCommand { .. }));
}

/// The program writes to the standard output that the caller names
#[tokio::test]
async fn start_interactive_writes_to_the_standard_output_of_the_caller() {
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("output");
    let streams = quiet().stdout(std::fs::File::create(&output).unwrap());

    shell("echo hello")
        .start_interactive(streams)
        .unwrap()
        .wait()
        .await
        .unwrap();

    assert_eq!(std::fs::read_to_string(&output).unwrap().trim(), "hello");
}

/// A stop after a dropped wait still asks the program to end, because the wait leaves the handle
#[cfg(unix)]
#[tokio::test]
async fn stop_after_a_dropped_wait_asks_the_program_to_end() {
    let directory = tempfile::tempdir().unwrap();
    let mut run = ready(directory.path(), &spin("exit 3")).await.unwrap();
    let _ = timeout(Duration::from_millis(100), run.wait()).await;

    let status = run.stop(GRACE).await.unwrap();

    assert_eq!(status.code(), Some(3));
}

/// A stop of a program that ended reports the status of that program
#[tokio::test]
async fn stop_after_a_wait_reports_the_status() {
    let mut run = shell("exit 7").start_interactive(quiet()).unwrap();
    run.wait().await.unwrap();

    let status = run.stop(Duration::from_secs(1)).await.unwrap();

    assert_eq!(status.code(), Some(7));
}

/// A stopped program answers the request, because the stop continues the group
#[cfg(unix)]
#[tokio::test]
async fn stop_continues_a_program_that_a_signal_stopped() {
    let directory = tempfile::tempdir().unwrap();
    let run = ready(directory.path(), &spin("exit 3")).await.unwrap();
    let process = Pid::from_raw(i32::try_from(run.id().unwrap().get()).unwrap()).unwrap();
    kill_process(process, Signal::STOP).unwrap();

    let status = run.stop(GRACE).await.unwrap();

    assert_eq!(status.code(), Some(3));
}

/// A program that ignores the request ends with the kill, and its status names the signal
#[cfg(unix)]
#[tokio::test]
async fn stop_kills_a_program_that_ignores_the_request() {
    use std::os::unix::process::ExitStatusExt;

    let directory = tempfile::tempdir().unwrap();
    let run = ready(directory.path(), &spin("")).await.unwrap();

    let status = run.stop(Duration::from_millis(100)).await.unwrap();

    assert_eq!(status.signal(), Some(9));
}

/// A stop kills a program of the group that ignores the request, after the program ended
#[cfg(unix)]
#[tokio::test]
async fn stop_kills_a_program_that_the_program_started() {
    let directory = tempfile::tempdir().unwrap();
    let started = directory.path().join("started");
    let script = format!(
        "trap 'exit 0' TERM; (trap '' TERM; sleep 300) & echo $! > '{}'; READY; wait",
        started.display()
    );
    let run = ready(directory.path(), &script).await.unwrap();

    run.stop(GRACE).await.unwrap();

    wait_for_the_end_of(&started).await.unwrap();
}

/// A wait reports the status with which the program ended, also when it is no success
#[tokio::test]
async fn wait_reports_the_status_of_the_program() {
    let mut run = shell("exit 3").start_interactive(quiet()).unwrap();

    let status = run.wait().await.unwrap();

    assert_eq!(status.code(), Some(3));
}

/// A second wait reports the status of the first, because the program ended once
#[tokio::test]
async fn wait_twice_reports_the_same_status() {
    let mut run = shell("exit 3").start_interactive(quiet()).unwrap();
    run.wait().await.unwrap();

    let status = run.wait().await.unwrap();

    assert_eq!(status.code(), Some(3));
}
