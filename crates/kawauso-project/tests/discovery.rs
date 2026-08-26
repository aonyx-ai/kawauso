//! Tests for the discovery of a project through the public API
//!
//! The unit tests of the crate reach the walk and the resolution of the start
//! directly, so that each of them can name a start and a working directory of
//! its own. A test here goes through [`Project::discover`], which puts the
//! two together.
//!
//! A search that names no directory reads the working directory of the
//! process. A test that changes it changes it for every thread of the
//! process, and the tests of a binary run in parallel, so such a test would
//! decide what the other tests see.
//!
//! A test that needs a working directory therefore has two halves. The first
//! half writes a marker and starts a child process that has a working
//! directory of its own. The child runs the second half, which is a test of
//! this same binary: it asks for the project and asserts that the search
//! returned the working directory. The first half then asserts that the child
//! reported success.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

use std::process::Command;
use std::process::Output;

use kawauso_project::Project;
use kawauso_project::Search;

/// The marker that identifies the projects of these tests
const MARKER: &str = ".git";

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
/// Neither flag matches a test that starts a child, so the child cannot start
/// a child of its own. `--nocapture` sends the report of a failed assertion to
/// the standard error of the child, from which the caller reads it.
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
/// The report of the harness of the tests arrives on the standard output, and
/// the reason of a failed assertion on the standard error. A test needs both:
/// the first says that the child ran the test, and the second says why the
/// test failed.
fn report_of(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

// project[verify discover.start.caller]
#[test]
fn discover_with_a_start_in_one_of_two_projects_returns_that_project() {
    let directory = tempfile::tempdir().unwrap();
    let first = directory.path().join("first");
    let second = directory.path().join("second");
    std::fs::create_dir_all(first.join(MARKER)).unwrap();
    std::fs::create_dir_all(second.join(MARKER)).unwrap();
    std::fs::create_dir_all(second.join("src")).unwrap();

    let search = Search::start(second.join("src")).marker(MARKER);
    let project = Project::discover(&search).unwrap();

    assert_eq!(project.root().get(), second);
}

// project[verify discover.start.working-directory]
#[test]
fn discover_with_the_working_directory_returns_the_working_directory() {
    let working_directory = tempfile::tempdir().unwrap();
    std::fs::create_dir(working_directory.path().join(MARKER)).unwrap();

    let output = child("child::discover_with_the_working_directory")
        .current_dir(working_directory.path())
        .output()
        .unwrap();

    let report = report_of(&output);
    assert!(report.contains(PASSED), "{report}");
}

/// The halves of the tests that a child process runs
///
/// Each test here belongs to a test above it, which prepares a marker and
/// starts the child. Only that test can give this one the working directory
/// that it needs, so `#[ignore]` keeps a run of the suite from reaching it on
/// its own.
mod child {
    use kawauso_project::Project;
    use kawauso_project::Search;

    use super::MARKER;

    #[test]
    #[ignore = "needs the working directory that the first half of the test prepares"]
    fn discover_with_the_working_directory() {
        let search = Search::working_directory().marker(MARKER);

        let project = Project::discover(&search).unwrap();

        assert_eq!(project.root().get(), std::env::current_dir().unwrap());
    }
}
