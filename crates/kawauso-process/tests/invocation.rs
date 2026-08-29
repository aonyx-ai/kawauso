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
