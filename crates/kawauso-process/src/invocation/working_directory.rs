//! The directory in which an external command runs

use typed_fields::path;

path!(
    /// The directory in which an external command runs
    ///
    /// Some commands work on the directory that they run in. A build tool
    /// reads the manifest of the project in that directory, and a formatter
    /// reads the configuration file that it finds there. Such a command
    /// carries the directory that it needs.
    ///
    /// A command without a working directory runs where the process runs.
    /// Most commands need no directory of their own, which is why an
    /// invocation does not require one.
    WorkingDirectory
);

/// Creates a working directory from a path that the application built at
/// runtime
impl From<std::path::PathBuf> for WorkingDirectory {
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

    // An application that runs a command in the project that it found holds
    // a `PathBuf`, and the working directory takes it without a borrow.
    #[test]
    fn from_path_buf_keeps_the_path() {
        let directory = WorkingDirectory::from(std::path::PathBuf::from("crates/example"));

        assert_eq!(directory.get(), std::path::Path::new("crates/example"));
    }
}
