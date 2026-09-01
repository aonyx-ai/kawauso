//! Tests for the run of a command through the public API
//!
//! Every test here starts a real program, because the obligations of a run
//! exist only when a program runs: the capture of a stream, the status of a
//! command, and the pipes that never block. The shell of the platform is the
//! program that every machine has, so most tests give a command line to it.
//!
//! Some tests need a tool that only a Unix system has, and they run there
//! alone. The project has no Windows runner, so no test pins the behavior of
//! the crate on that platform.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use std::time::Duration;

use kawauso_process::Invocation;
use kawauso_process::error::RequireSuccessError;
use kawauso_process::error::RunCommandError;
#[cfg(unix)]
use tokio::time::sleep;
#[cfg(unix)]
use tokio::time::timeout;

/// The number of bytes that a command writes to each stream in the test of
/// the concurrent read
///
/// A pipe of the operating system holds a few dozen kilobytes, and the
/// command writes far more than that to both of its streams. A run that read
/// one stream after the other, or that waited for the end before it read,
/// would never finish this command.
#[cfg(unix)]
const FLOOD: usize = 256 * 1024;

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

// process[verify run.success.message]
#[tokio::test]
async fn require_success_with_a_command_that_failed_names_the_command() {
    let invocation = shell(&["exit 3"]);
    let execution = invocation.run().await.unwrap();

    let error = execution.require_success().unwrap_err();

    assert!(error.to_string().contains(&invocation.to_string()));
}

// A program states on its standard error why it stopped, and the reader of a
// report has only the message of the error.
// process[verify run.success.message]
#[tokio::test]
async fn require_success_with_a_command_that_failed_names_the_standard_error() {
    let execution = shell(&["echo oops 1>&2", "exit 3"]).run().await.unwrap();

    let error = execution.require_success().unwrap_err();

    assert!(error.to_string().contains("oops"));
}

// process[verify run.success.message]
#[tokio::test]
async fn require_success_with_a_command_that_failed_names_the_status() {
    let execution = shell(&["exit 3"]).run().await.unwrap();
    let status = execution.status();

    let error = execution.require_success().unwrap_err();

    assert!(error.to_string().contains(&status.to_string()));
}

// process[verify run.success]
#[tokio::test]
async fn require_success_with_a_command_that_failed_returns_an_error() {
    let execution = shell(&["exit 3"]).run().await.unwrap();

    let error = execution.require_success().unwrap_err();

    assert!(matches!(
        error,
        RequireSuccessError::UnsuccessfulCommand { .. }
    ));
}

// The check returns the result of the run, so a caller that required success
// reads the output of the command from the value that it got back.
// process[verify run.success]
#[tokio::test]
async fn require_success_with_a_command_that_succeeded_returns_the_execution() {
    let execution = shell(&["echo hello"]).run().await.unwrap();

    let execution = execution.require_success().unwrap();

    assert_eq!(execution.stdout().to_string_lossy().trim(), "hello");
}

// A timeout, and every other caller that abandons a run, drops the future of
// the run. The command writes the file after the drop, so a file that exists
// at the end is a command that outlived its run.
// process[verify run.abandonment]
#[cfg(unix)]
#[tokio::test]
async fn run_that_the_caller_drops_ends_the_command() {
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("marker");
    let command = format!("sleep 1; : > \"{}\"", marker.display());

    let _ = timeout(Duration::from_millis(100), shell(&[&command]).run()).await;
    sleep(Duration::from_secs(2)).await;

    assert!(!marker.exists());
}

// The crate gives the program to the operating system as the caller wrote it,
// and the platform resolves a bare name. The crate searches no path itself.
// process[verify run.resolution]
#[cfg(unix)]
#[tokio::test]
async fn run_with_a_bare_program_name_starts_the_program() {
    let invocation = Invocation::new("true");

    let execution = invocation.run().await.unwrap();

    assert!(execution.status().success());
}

// A command that ends without success is a result and not an error of the
// run, so the run reports the code that the command chose.
// process[verify run.exit]
#[tokio::test]
async fn run_with_a_command_that_fails_reports_the_exit_code() {
    let invocation = shell(&["exit 3"]);

    let execution = invocation.run().await.unwrap();

    assert_eq!(execution.status().code(), Some(3));
}

// process[verify run.drain]
#[cfg(unix)]
#[tokio::test]
async fn run_with_a_command_that_fills_both_pipes_captures_everything() {
    let invocation = shell(&[
        &format!("head -c {FLOOD} /dev/zero"),
        &format!("head -c {FLOOD} /dev/zero >&2"),
    ]);

    let execution = invocation.run().await.unwrap();

    assert_eq!(
        (
            execution.stdout().get().len(),
            execution.stderr().get().len()
        ),
        (FLOOD, FLOOD)
    );
}

// The command reads a variable that the process holds, and it reports the
// same value, so the command runs in the environment of the process.
// process[verify run.environment]
#[cfg(unix)]
#[tokio::test]
async fn run_with_a_command_that_reads_the_environment_sees_the_environment_of_the_process() {
    let invocation = shell(&["echo \"$PATH\""]);

    let execution = invocation.run().await.unwrap();

    assert_eq!(
        execution.stdout().to_string_lossy().trim(),
        std::env::var("PATH").unwrap()
    );
}

// The command counts the bytes of its standard input. A command that read the
// input of the process would report another number, and a command that waited
// for input would never end.
// process[verify run.stdin]
#[cfg(unix)]
#[tokio::test]
async fn run_with_a_command_that_reads_the_standard_input_sees_the_end_of_the_input() {
    let invocation = Invocation::new("wc").arg("-c");

    let execution = invocation.run().await.unwrap();

    assert_eq!(execution.stdout().to_string_lossy().trim(), "0");
}

// process[verify run]
// process[verify run.exit]
#[tokio::test]
async fn run_with_a_command_that_succeeds_reports_a_successful_status() {
    let invocation = shell(&["exit 0"]);

    let execution = invocation.run().await.unwrap();

    assert!(execution.status().success());
}

// The command sleeps for the time that the assertion names, so a run that
// reported the time of the call, or no time at all, fails here.
// process[verify run.duration]
#[cfg(unix)]
#[tokio::test]
async fn run_with_a_command_that_takes_time_reports_the_duration() {
    let invocation = shell(&["sleep 0.2"]);

    let execution = invocation.run().await.unwrap();

    assert!(execution.duration() >= Duration::from_millis(200));
}

// An application that reports which program ran needs the identifier of the
// command, and the run is the only place that has it. The shell writes its
// own identifier, so the test compares the value of the operating system with
// the value of the result.
// process[verify run.identity]
#[cfg(unix)]
#[tokio::test]
async fn run_with_a_command_that_writes_its_identifier_reports_the_same_identifier() {
    let execution = shell(&["echo $$"]).run().await.unwrap();

    let id = execution.id().unwrap();

    assert_eq!(
        id.get().to_string(),
        execution.stdout().to_string_lossy().trim()
    );
}

// process[verify run.output]
#[tokio::test]
async fn run_with_a_command_that_writes_to_the_standard_error_captures_it() {
    let invocation = shell(&["echo oops 1>&2"]);

    let execution = invocation.run().await.unwrap();

    assert_eq!(
        (
            execution.stdout().to_string_lossy().trim(),
            execution.stderr().to_string_lossy().trim()
        ),
        ("", "oops")
    );
}

// process[verify run.output]
#[tokio::test]
async fn run_with_a_command_that_writes_to_the_standard_output_captures_it() {
    let invocation = shell(&["echo hello"]);

    let execution = invocation.run().await.unwrap();

    assert_eq!(
        (
            execution.stdout().to_string_lossy().trim(),
            execution.stderr().to_string_lossy().trim()
        ),
        ("hello", "")
    );
}

// process[verify run.error.message]
#[tokio::test]
async fn run_with_a_program_that_does_not_exist_names_the_command() {
    let invocation = Invocation::new("kawauso-no-such-program").arg("--version");

    let error = invocation.run().await.unwrap_err();

    assert!(
        error
            .to_string()
            .contains("kawauso-no-such-program --version")
    );
}

// process[verify run.error]
#[tokio::test]
async fn run_with_a_program_that_does_not_exist_returns_an_error() {
    let invocation = Invocation::new("kawauso-no-such-program");

    let error = invocation.run().await.unwrap_err();

    assert!(matches!(error, RunCommandError::UnstartableCommand { .. }));
}

// A program that takes a decision from a variable, and from no flag, has to
// see the variable that the caller set for the command. The command writes
// the value that it reads, so a run that dropped the variable reports an
// empty line.
// process[verify invocation.environment]
#[cfg(unix)]
#[tokio::test]
async fn run_with_a_variable_gives_the_command_the_variable() {
    let invocation =
        shell(&["echo \"$KAWAUSO_PROCESS_TEST\""]).env("KAWAUSO_PROCESS_TEST", "value");

    let execution = invocation.run().await.unwrap();

    assert_eq!(execution.stdout().to_string_lossy().trim(), "value");
}

// The variables of the caller come on top of the environment of the process,
// and not in its place. A command that reads a variable of the process
// therefore sees the same value as the process, although the caller set a
// variable of its own.
// process[verify invocation.environment]
#[cfg(unix)]
#[tokio::test]
async fn run_with_a_variable_keeps_the_environment_of_the_process() {
    let invocation = shell(&["echo \"$PATH\""]).env("KAWAUSO_PROCESS_TEST", "value");

    let execution = invocation.run().await.unwrap();

    assert_eq!(
        execution.stdout().to_string_lossy().trim(),
        std::env::var("PATH").unwrap()
    );
}

// A variable of the caller with the name of a variable of the process holds
// the value of the caller for the command. Every process has a home
// directory in its environment, and the command reports the one that the
// caller set.
// process[verify invocation.environment]
#[cfg(unix)]
#[tokio::test]
async fn run_with_a_variable_that_the_process_holds_replaces_it_for_the_command() {
    let invocation = shell(&["echo \"$HOME\""]).env("HOME", "/kawauso");

    let execution = invocation.run().await.unwrap();

    assert_eq!(execution.stdout().to_string_lossy().trim(), "/kawauso");
}

// A command that works on a directory has to start in that directory, and not
// where the process runs.
// process[verify invocation.directory]
#[cfg(unix)]
#[tokio::test]
async fn run_with_a_working_directory_starts_the_command_in_it() {
    let directory = tempfile::tempdir().unwrap();
    let invocation = Invocation::new("pwd").in_directory(directory.path().to_path_buf());

    let execution = invocation.run().await.unwrap();

    assert_eq!(
        Path::new(execution.stdout().to_string_lossy().trim())
            .canonicalize()
            .unwrap(),
        directory.path().canonicalize().unwrap()
    );
}
