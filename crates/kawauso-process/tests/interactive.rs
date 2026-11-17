//! Tests for the interactive run through the public API
//!
//! Every test starts a real program, because the obligations of the run exist
//! only when a program runs. They cover the streams and the status.

#![allow(
    clippy::missing_panics_doc,
    reason = "an assertion in a test panics by design, and a section on every test tells the reader nothing"
)]

use std::process::Stdio;

use kawauso_process::Invocation;
use kawauso_process::error::RunCommandError;
use kawauso_process::invocation::Streams;

/// Returns the streams of a test, which connect the program to nothing
fn quiet() -> Streams {
    Streams::inherit()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
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
