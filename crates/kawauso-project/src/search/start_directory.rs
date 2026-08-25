//! The directory at which the walk of a search starts

use typed_fields::path;

path!(
    /// The directory at which the walk of a search starts
    ///
    /// The search stores the value as the developer gave it. When the search
    /// runs, a relative path resolves against the working directory of the
    /// process, and the `.` and `..` components go, so that the walk never
    /// passes through a directory that the developer did not name. Symbolic
    /// links are not resolved, so the walk sees the tree that the developer
    /// named.
    ///
    /// The value can name a file. The walk then starts at the directory that
    /// holds the file, because the project that governs a file is the project
    /// of that directory.
    ///
    /// An error that names the start shows it in the resolved form, because
    /// that is where the walk began.
    StartDirectory
);

/// Creates a start from a path that the application built at runtime
impl From<std::path::PathBuf> for StartDirectory {
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

    // A path that the user names arrives as a `PathBuf`, and the start takes
    // it without a borrow.
    #[test]
    fn from_path_buf_keeps_the_path() {
        let start = StartDirectory::from(std::path::PathBuf::from("src"));

        assert_eq!(start.get(), std::path::Path::new("src"));
    }
}
