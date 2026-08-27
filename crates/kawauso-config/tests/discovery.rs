//! Tests for the search for a configuration file
//!
//! A search reads the environment of the process: the walk up the ancestors
//! starts at the working directory, and the search of the user directory
//! starts at the home directory. A test that changes one of them changes it
//! for every thread of the process, and the tests of a binary run in
//! parallel, so such a test would decide what the other tests see.
//!
//! A test here therefore has two halves. The first half writes a
//! configuration file and starts a child process that has a working
//! directory and a home directory of its own. The child runs the second
//! half, which is a test of this same binary: it asks the loader for the
//! configuration and asserts that the loader returned what the file holds.
//! The first half then asserts that the child reported success.
//!
//! A search that finds nothing needs no environment of its own, and the
//! tests for it live with the other tests of the loader.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

use std::process::Command;
use std::process::Output;

use serde::Deserialize;

/// The name of the application whose configuration the tests search for
const APPLICATION: &str = "kawauso";

/// The configuration that [`CONTENTS`] describes
const CONFIGURATION: Configuration = Configuration { port: 8080 };

/// The contents of the configuration file that the tests write
const CONTENTS: &str = "port = 8080";

/// The line with which a child process reports one test that passed
///
/// A filter that matches no test is a success for the harness of the tests,
/// so an exit code of zero does not say that the child ran anything. A test
/// that looks for this line therefore fails when the name of its half in the
/// child changes, instead of passing without a search.
const PASSED: &str = "test result: ok. 1 passed";

/// The configuration of an imaginary application
///
/// A caller of the crate defines a type like this one for its own
/// configuration file. One field is enough here, because these tests are
/// about the file that the search finds, not about the shape of a
/// configuration.
#[derive(Eq, PartialEq, Debug, Deserialize)]
struct Configuration {
    port: u16,
}

/// Prepares a child process that runs one test of this binary
///
/// The caller sets the working directory or the home directory on the
/// command, and the child then searches an environment that the test
/// controls.
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

/// Returns the directory in which the platform keeps the configuration of a user
///
/// A test writes the configuration file where the crate looks for it, and it
/// must know that place without a call into the crate. This function is
/// therefore the rule of the platform, stated a second time.
///
/// The home directory decides the answer, so the caller can hand a child
/// process a directory of the platform that holds nothing but the file of
/// the test. Windows names the directory through an interface that no
/// environment redirects, which is why the rule of that platform is missing
/// here and no test calls this function there.
#[cfg(not(windows))]
fn configuration_directory(home: &std::path::Path) -> std::path::PathBuf {
    if cfg!(target_os = "macos") {
        home.join("Library").join("Application Support")
    } else {
        home.join(".config")
    }
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

// The tool that this case comes from keeps its configuration in
// `.github`, and its users ran it without a flag and got a report that no
// configuration file exists. The search has to reach the file that the
// project already has.
// config[verify discover.ancestors.subdirectories]
#[test]
fn load_with_ancestors_and_a_file_in_a_subdirectory_returns_the_configuration() {
    let working_directory = tempfile::tempdir().unwrap();
    let subdirectory = working_directory.path().join(".github");
    std::fs::create_dir(&subdirectory).unwrap();
    std::fs::write(subdirectory.join(format!("{APPLICATION}.toml")), CONTENTS).unwrap();

    let output = child("child::load_with_ancestors_and_a_subdirectory")
        .current_dir(working_directory.path())
        .output()
        .unwrap();

    let report = report_of(&output);
    assert!(report.contains(PASSED), "{report}");
}

// The dot-config convention also keeps the file with the name of the
// application in the shared directory `.config`, and the search finds it
// there.
// config[verify discover.ancestors.dot]
#[test]
fn load_with_ancestors_and_a_file_in_dot_config_file_returns_the_configuration() {
    let working_directory = tempfile::tempdir().unwrap();
    let directory = working_directory.path().join(".config");
    std::fs::create_dir(&directory).unwrap();
    std::fs::write(directory.join(format!("{APPLICATION}.toml")), CONTENTS).unwrap();

    let output = child("child::load_with_ancestors_and_dot_config_file")
        .current_dir(working_directory.path())
        .output()
        .unwrap();

    let report = report_of(&output);
    assert!(report.contains(PASSED), "{report}");
}

// The dot-config convention gives the application a directory of its own,
// and its configuration file sits inside it.
// config[verify discover.ancestors.dot]
#[test]
fn load_with_ancestors_and_a_file_in_the_dot_config_directory_returns_the_configuration() {
    let working_directory = tempfile::tempdir().unwrap();
    let directory = working_directory.path().join(".config").join(APPLICATION);
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("config.toml"), CONTENTS).unwrap();

    let output = child("child::load_with_ancestors_and_dot_config_directory")
        .current_dir(working_directory.path())
        .output()
        .unwrap();

    let report = report_of(&output);
    assert!(report.contains(PASSED), "{report}");
}

// config[verify discover.ancestors.working-directory]
#[test]
fn load_with_ancestors_and_a_file_in_the_working_directory_returns_the_configuration() {
    let working_directory = tempfile::tempdir().unwrap();
    let path = working_directory.path().join(format!("{APPLICATION}.toml"));
    std::fs::write(path, CONTENTS).unwrap();

    let output = child("child::load_with_ancestors")
        .current_dir(working_directory.path())
        .output()
        .unwrap();

    let report = report_of(&output);
    assert!(report.contains(PASSED), "{report}");
}

// The tests of the crate must not read the configuration directory of the
// user who runs them, so the child gets a home directory of its own.
// `XDG_CONFIG_HOME` would name a directory outside that home, and the child
// inherits the environment of this process, so the variable has to go.
// config[verify discover.user.name]
#[cfg(not(windows))]
#[test]
fn load_with_user_directory_and_a_file_returns_the_configuration() {
    let home = tempfile::tempdir().unwrap();
    let directory = configuration_directory(home.path()).join(APPLICATION);
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("config.toml"), CONTENTS).unwrap();

    let output = child("child::load_with_user_directory")
        .env("HOME", home.path())
        .env_remove("XDG_CONFIG_HOME")
        .output()
        .unwrap();

    let report = report_of(&output);
    assert!(report.contains(PASSED), "{report}");
}

/// The halves of the tests that a child process runs
///
/// Each test here belongs to a test above it, which prepares a configuration
/// file and starts the child. Only that test can give this one the
/// environment that it needs, so `#[ignore]` keeps a run of the suite from
/// reaching it on its own.
mod child {
    use kawauso_config::AncestorsSearch;
    use kawauso_config::Loader;

    use super::APPLICATION;
    use super::CONFIGURATION;
    use super::Configuration;

    #[test]
    #[ignore = "needs the working directory that the first half of the test prepares"]
    fn load_with_ancestors() {
        let configuration: Configuration = Loader::ancestors(APPLICATION).load().unwrap();

        assert_eq!(configuration, CONFIGURATION);
    }

    #[test]
    #[ignore = "needs the working directory that the first half of the test prepares"]
    fn load_with_ancestors_and_a_subdirectory() {
        let search = AncestorsSearch::new(APPLICATION).subdirectory(".github");

        let configuration: Configuration = Loader::ancestors(search).load().unwrap();

        assert_eq!(configuration, CONFIGURATION);
    }

    #[test]
    #[ignore = "needs the working directory that the first half of the test prepares"]
    fn load_with_ancestors_and_dot_config_directory() {
        let search = AncestorsSearch::new(APPLICATION).dot_config();

        let configuration: Configuration = Loader::ancestors(search).load().unwrap();

        assert_eq!(configuration, CONFIGURATION);
    }

    #[test]
    #[ignore = "needs the working directory that the first half of the test prepares"]
    fn load_with_ancestors_and_dot_config_file() {
        let search = AncestorsSearch::new(APPLICATION).dot_config();

        let configuration: Configuration = Loader::ancestors(search).load().unwrap();

        assert_eq!(configuration, CONFIGURATION);
    }

    #[cfg(not(windows))]
    #[test]
    #[ignore = "needs the home directory that the first half of the test prepares"]
    fn load_with_user_directory() {
        let configuration: Configuration = Loader::user(APPLICATION).load().unwrap();

        assert_eq!(configuration, CONFIGURATION);
    }
}
