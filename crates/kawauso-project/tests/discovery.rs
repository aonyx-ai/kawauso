//! Tests for the start of the walk at the working directory
//!
//! A search that names no start reads the working directory of the process.
//! A test that changes it changes it for every thread of the process, and
//! the tests of a binary run in parallel, so such a test would decide what
//! the other tests see.
//!
//! A test here therefore has two halves. The first half writes a marker and
//! starts a child process that has a working directory of its own. The
//! child runs the second half, which is a test of this same binary: it asks
//! for the project and asserts that the search returned the working
//! directory. The first half then asserts that the child reported success.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

use std::process::Command;
use std::process::Output;

/// The name of the application whose project the tests search for
const APPLICATION: &str = "kawauso";

/// The line with which a child process reports one test that passed
///
/// A filter that matches no test is a success for the harness of the tests,
/// so an exit code of zero does not say that the child ran anything. A test
/// that looks for this line therefore fails when the name of its half in the
/// child changes, instead of passing without a search.
const PASSED: &str = "test result: ok. 1 passed";

/// Prepares a child process that runs one test of this binary
///
/// The caller sets the working directory on the command, and the child then
/// searches an environment that the test controls.
///
/// The test that the child runs carries `#[ignore]`, so `--ignored` is what
/// selects it, and `--exact` keeps the name from matching another test.
/// Neither flag matches a test that starts a child, so the child cannot
/// start a child of its own. `--nocapture` sends the report of a failed
/// assertion to the standard error of the child, from which the caller reads
/// it.
fn child(test: &str) -> Command {
    // The lints that ban `unwrap` and `expect` make an exception for a test,
    // and a helper of a test is not one, so the failure gets its own report.
    let Ok(binary) = std::env::current_exe() else {
        panic!("the test needs the path of the binary that runs it");
    };

    let mut command = Command::new(binary);
    command.args(["--exact", test, "--ignored", "--nocapture"]);

    command
}

/// Returns everything that a child process wrote
///
/// The report of the harness of the tests arrives on the standard output,
/// and the reason of a failed assertion on the standard error. A test needs
/// both: the first says that the child ran the test, and the second says why
/// the test failed.
fn report_of(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

// project[verify discover.start.working-directory]
#[test]
fn discover_without_a_start_starts_at_the_working_directory() {
    let working_directory = tempfile::tempdir().unwrap();
    let subdirectory = working_directory.path().join(".config");
    std::fs::create_dir(&subdirectory).unwrap();
    std::fs::write(subdirectory.join(format!("{APPLICATION}.toml")), "").unwrap();

    let output = child("child::discover_without_a_start")
        .current_dir(working_directory.path())
        .output()
        .unwrap();

    let report = report_of(&output);
    assert!(report.contains(PASSED), "{report}");
}

/// The halves of the tests that a child process runs
///
/// Each test here belongs to a test above it, which prepares a marker and
/// starts the child. Only that test can give this one the environment that
/// it needs, so `#[ignore]` keeps a run of the suite from reaching it on its
/// own.
mod child {
    use kawauso_project::Project;

    use super::APPLICATION;

    #[test]
    #[ignore = "needs the working directory that the first half of the test prepares"]
    fn discover_without_a_start() {
        let working_directory = std::env::current_dir().unwrap();

        let project = Project::discover(APPLICATION).unwrap();

        assert_eq!(project.root().get(), working_directory);
    }
}
