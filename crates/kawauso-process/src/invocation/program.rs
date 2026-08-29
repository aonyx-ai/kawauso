//! The program that an external command starts

use typed_fields::path;

path!(
    /// The program that an external command starts
    ///
    /// The value is the path of an executable file, such as
    /// `/usr/bin/git` or `./scripts/build.sh`, or the bare name of one, such
    /// as `git`. The operating system resolves a bare name when the command
    /// runs, with the rules of the platform that it runs on.
    ///
    /// The value is stored as the caller gave it. A name that no program on
    /// the machine answers to is therefore a valid program, and the failure
    /// appears when the command runs.
    Program
);

/// Creates a program from a path that the application built at runtime
impl From<std::path::PathBuf> for Program {
    fn from(path: std::path::PathBuf) -> Self {
        Self::new(path)
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every
    // test would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    // An application that resolves the program itself, through a version
    // manager for example, holds a `PathBuf`, and the program takes it
    // without a borrow.
    #[test]
    fn from_path_buf_keeps_the_path() {
        let program = Program::from(std::path::PathBuf::from("/usr/bin/git"));

        assert_eq!(program.get(), std::path::Path::new("/usr/bin/git"));
    }
}
