//! Tests for the description of a command through the public API
//!
//! An invocation is a value that an application builds and reads, so every
//! test here builds one and asserts what it holds or how it renders. Nothing
//! starts a program, because the crate describes a command in this slice and
//! does not run it.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

use std::path::Path;

use kawauso_process::Invocation;
use kawauso_process::invocation::Argument;

/// Returns the name and the value of every variable, in the order of the calls
///
/// A test that makes a statement about the whole environment of an invocation
/// compares it with literals, and the form of a shell, `NAME=value`, is the
/// form that a reader of the test knows.
fn variables(invocation: &Invocation) -> Vec<String> {
    invocation
        .environment()
        .iter()
        .map(|variable| format!("{}={}", variable.name(), variable.value()))
        .collect()
}

/// Returns every name that the caller took out, in the order of the calls
///
/// A test that makes a statement about the whole set of removals compares it
/// with literals, and the name alone is what the invocation carries.
fn removals(invocation: &Invocation) -> Vec<String> {
    invocation
        .removals()
        .iter()
        .map(ToString::to_string)
        .collect()
}

// A character that a shell expands reaches the program as the caller wrote
// it, because no shell reads the command.
// process[verify invocation.arguments]
#[test]
fn arg_with_a_metacharacter_keeps_the_argument() {
    let invocation = Invocation::new("ls").arg("*.rs");

    assert_eq!(invocation.arguments()[0].get(), "*.rs");
}

// A space inside an argument does not separate two arguments, which is what
// a shell would make of it.
// process[verify invocation.arguments]
#[test]
fn arg_with_a_space_keeps_one_argument() {
    let invocation = Invocation::new("git").arg("--message=two words");

    assert_eq!(invocation.arguments().len(), 1);
}

// process[verify invocation.arguments]
#[test]
fn arg_with_two_arguments_keeps_the_order_of_the_calls() {
    let invocation = Invocation::new("git").arg("status").arg("--short");

    let arguments: Vec<_> = invocation.arguments().iter().map(Argument::get).collect();

    assert_eq!(arguments, vec!["status", "--short"]);
}

// process[verify invocation.arguments]
#[test]
fn args_with_a_collection_appends_every_argument() {
    let invocation = Invocation::new("cargo")
        .arg("build")
        .args(["--release", "--locked"]);

    assert_eq!(invocation.arguments().len(), 3);
}

// A space separates the words of a command line, so a program whose path
// holds one would read as a program and an argument.
// process[verify invocation.display]
#[test]
fn display_with_a_program_that_holds_a_space_marks_it() {
    let invocation = Invocation::new("/opt/my tool/bin/tool");

    assert_eq!(invocation.to_string(), "\"/opt/my tool/bin/tool\"");
}

// No shell takes a name out of the environment of one command, and the `env`
// program writes a minus sign in front of the name. A reader of a log line
// sees the same form here.
// process[verify invocation.environment.removal.display]
#[test]
fn display_with_a_removed_name_shows_it_in_front_of_the_program() {
    let invocation = Invocation::new("git").arg("status").env_remove("GIT_DIR");

    assert_eq!(invocation.to_string(), "-GIT_DIR git status");
}

// A line that mixed the two would leave a reader guessing which name the
// command holds and which one it misses, so the removals come first.
// process[verify invocation.environment.removal.display]
#[test]
fn display_with_a_removed_name_and_a_variable_shows_the_removal_first() {
    let invocation = Invocation::new("git")
        .arg("status")
        .env("LC_ALL", "C")
        .env_remove("GIT_DIR");

    assert_eq!(invocation.to_string(), "-GIT_DIR LC_ALL=C git status");
}

// A shell sets a variable for one command in front of the program, and a
// reader of a log line knows that form.
// process[verify invocation.environment.display]
#[test]
fn display_with_a_variable_shows_it_in_front_of_the_program() {
    let invocation = Invocation::new("cargo")
        .arg("build")
        .env("RUSTUP_TOOLCHAIN", "nightly");

    assert_eq!(
        invocation.to_string(),
        "RUSTUP_TOOLCHAIN=nightly cargo build"
    );
}

// A value that holds a space would read as a variable and an argument in a
// line that separates the words with spaces. The marks go around the value
// alone, the way a shell writes them.
// process[verify invocation.environment.display]
#[test]
fn display_with_a_variable_whose_value_holds_a_space_marks_the_value() {
    let invocation = Invocation::new("cargo")
        .arg("build")
        .env("RUSTFLAGS", "-D warnings");

    assert_eq!(
        invocation.to_string(),
        "RUSTFLAGS=\"-D warnings\" cargo build"
    );
}

// A value that is empty would leave the reader with a name and an equals sign,
// and the marks show that the value ends where it starts.
// process[verify invocation.environment.display]
#[test]
fn display_with_a_variable_whose_value_is_empty_marks_the_value() {
    let invocation = Invocation::new("cargo").arg("build").env("NO_COLOR", "");

    assert_eq!(invocation.to_string(), "NO_COLOR=\"\" cargo build");
}

// An argument that holds a space would read as two arguments in a line that
// separates the arguments with spaces.
// process[verify invocation.display]
#[test]
fn display_with_an_argument_that_holds_a_space_marks_it() {
    let invocation = Invocation::new("sh").arg("-c").arg("echo hello world");

    assert_eq!(invocation.to_string(), "sh -c \"echo hello world\"");
}

// process[verify invocation.display]
#[test]
fn display_with_arguments_names_the_program_and_every_argument() {
    let invocation = Invocation::new("git").arg("status").arg("--short");

    assert_eq!(invocation.to_string(), "git status --short");
}

// process[verify invocation.environment.display]
#[test]
fn display_with_two_variables_keeps_the_order_of_the_calls() {
    let invocation = Invocation::new("cargo")
        .env("RUSTUP_TOOLCHAIN", "nightly")
        .env("CARGO_TERM_COLOR", "always");

    assert_eq!(
        invocation.to_string(),
        "RUSTUP_TOOLCHAIN=nightly CARGO_TERM_COLOR=always cargo"
    );
}

// The caller reads back what it took out, which is how an application logs
// or asserts the environment that a command will run in.
// process[verify invocation.environment.removal]
#[test]
fn env_remove_with_a_name_keeps_it() {
    let invocation = Invocation::new("git").env_remove("GIT_DIR");

    assert_eq!(removals(&invocation), vec!["GIT_DIR"]);
}

// A name is either set or taken out. A caller that sets a variable and then
// takes the name out means the command to miss it, so the variable goes.
// process[verify invocation.environment.exclusion]
#[test]
fn env_remove_with_the_name_of_a_variable_takes_the_variable_out() {
    let invocation = Invocation::new("git")
        .env("GIT_DIR", "/elsewhere")
        .env_remove("GIT_DIR");

    assert_eq!(variables(&invocation), Vec::<String>::new());
}

// A caller that names the same variable twice means it once, and the second
// call says nothing new.
// process[verify invocation.environment.removal]
#[test]
fn env_remove_with_a_repeated_name_keeps_one() {
    let invocation = Invocation::new("git")
        .env_remove("GIT_DIR")
        .env_remove("GIT_DIR");

    assert_eq!(removals(&invocation), vec!["GIT_DIR"]);
}

// A caller that takes several names out reads them back in the order that it
// named them, as it does for the variables that it set.
// process[verify invocation.environment.removal]
#[test]
fn env_remove_with_two_names_keeps_the_order_of_the_calls() {
    let invocation = Invocation::new("git")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_DIR");

    assert_eq!(removals(&invocation), vec!["GIT_INDEX_FILE", "GIT_DIR"]);
}

// The last call for a name decides, so a caller that takes a name out and
// then sets it means the command to read the value.
// process[verify invocation.environment.exclusion]
#[test]
fn env_with_the_name_of_a_removed_name_takes_the_name_back() {
    let invocation = Invocation::new("git")
        .env_remove("GIT_DIR")
        .env("GIT_DIR", "/repository");

    assert_eq!(removals(&invocation), Vec::<String>::new());
}

// process[verify invocation.environment]
#[test]
fn env_with_a_variable_keeps_it() {
    let invocation = Invocation::new("cargo").env("RUSTUP_TOOLCHAIN", "nightly");

    assert_eq!(variables(&invocation), vec!["RUSTUP_TOOLCHAIN=nightly"]);
}

// A variable that a later call replaces stays where the first call put it, so
// a caller that reads the environment sees the order of the names as it set
// them.
// process[verify invocation.environment.replacement]
#[test]
fn env_with_the_name_of_an_earlier_variable_keeps_the_place() {
    let invocation = Invocation::new("cargo")
        .env("RUSTUP_TOOLCHAIN", "stable")
        .env("CARGO_TERM_COLOR", "always")
        .env("RUSTUP_TOOLCHAIN", "nightly");

    let names: Vec<_> = invocation
        .environment()
        .iter()
        .map(|variable| variable.name().get())
        .collect();

    assert_eq!(names, vec!["RUSTUP_TOOLCHAIN", "CARGO_TERM_COLOR"]);
}

// A variable holds one value, so a later call with the same name leaves one
// variable with the later value, and not two variables with the same name.
// process[verify invocation.environment.replacement]
#[test]
fn env_with_the_name_of_an_earlier_variable_replaces_the_value() {
    let invocation = Invocation::new("cargo")
        .env("RUSTUP_TOOLCHAIN", "stable")
        .env("RUSTUP_TOOLCHAIN", "nightly");

    assert_eq!(variables(&invocation), vec!["RUSTUP_TOOLCHAIN=nightly"]);
}

// process[verify invocation.environment]
#[test]
fn env_with_two_variables_keeps_the_order_of_the_calls() {
    let invocation = Invocation::new("cargo")
        .env("RUSTUP_TOOLCHAIN", "nightly")
        .env("CARGO_TERM_COLOR", "always");

    assert_eq!(
        variables(&invocation),
        vec!["RUSTUP_TOOLCHAIN=nightly", "CARGO_TERM_COLOR=always"]
    );
}

// process[verify invocation.environment]
#[test]
fn envs_with_a_collection_sets_every_variable() {
    let invocation = Invocation::new("cargo")
        .env("RUSTUP_TOOLCHAIN", "nightly")
        .envs([("CARGO_TERM_COLOR", "always"), ("NO_COLOR", "")]);

    assert_eq!(invocation.environment().len(), 3);
}

// A collection goes through the same steps as one call per variable, so a
// name that appears twice in the collection leaves one variable behind.
// process[verify invocation.environment.replacement]
#[test]
fn envs_with_the_name_of_an_earlier_variable_replaces_the_value() {
    let invocation = Invocation::new("cargo").envs([
        ("RUSTUP_TOOLCHAIN", "stable"),
        ("RUSTUP_TOOLCHAIN", "nightly"),
    ]);

    assert_eq!(variables(&invocation), vec!["RUSTUP_TOOLCHAIN=nightly"]);
}

// process[verify invocation.directory]
#[test]
fn in_directory_with_a_directory_keeps_it() {
    let invocation = Invocation::new("cargo").in_directory("crates/example");

    assert_eq!(
        invocation
            .working_directory()
            .map(|directory| directory.get()),
        Some(Path::new("crates/example"))
    );
}

// process[verify invocation.program]
#[test]
fn new_with_a_program_keeps_the_program() {
    let invocation = Invocation::new("/usr/bin/git");

    assert_eq!(invocation.program().get(), Path::new("/usr/bin/git"));
}

// A command that names no directory of its own is a complete command, and
// not a mistake that the crate reports.
// process[verify invocation.directory]
#[test]
fn new_without_a_directory_reports_no_directory() {
    let invocation = Invocation::new("git").arg("status");

    assert!(invocation.working_directory().is_none());
}
