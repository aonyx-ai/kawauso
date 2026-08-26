//! An entry that identifies a project

use typed_fields::path;

path!(
    /// An entry that identifies a project
    ///
    /// A project holds an entry at a relative path, such as `.git`,
    /// `src/main.rs`, or `.config/example.toml`. The search tests whether the
    /// entry exists in a directory of the walk, and the first directory that
    /// holds any marker is the project.
    ///
    /// The search never reads the entry. An application that has to examine
    /// its contents, such as a `main.rs` file with a specific macro, examines
    /// the entry in the project that the search found.
    ///
    /// The value is a relative path that stays inside its directory. It can
    /// name more than one level, such as `.config/example.toml`. A value that
    /// is absolute, or that leaves the directory, takes the search outside
    /// the directories that it reads. The search reports such a value
    /// instead.
    ///
    /// The path is stored as the developer gave it. The check happens when
    /// the search runs, so a value that a developer computes cannot make a
    /// constructor fail.
    Marker
);

/// Creates a marker from a path that the application built at runtime
impl From<std::path::PathBuf> for Marker {
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

    // A path that the application builds at runtime arrives as a `PathBuf`,
    // and the marker takes it without a borrow.
    #[test]
    fn from_path_buf_keeps_the_path() {
        let marker = Marker::from(std::path::PathBuf::from(".git"));

        assert_eq!(marker.get(), std::path::Path::new(".git"));
    }
}
