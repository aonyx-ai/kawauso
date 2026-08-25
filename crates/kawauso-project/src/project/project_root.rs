//! The directory of a project

use typed_fields::path;

path!(
    /// The directory of a project
    ///
    /// The path is absolute and holds no `.` or `..` component. It is the
    /// directory of the walk in which a marker matched, or the start of the
    /// walk when the search fell back to it. The search observed the
    /// directory; it did not derive it from the path of a file.
    ///
    /// Symbolic links are not resolved, so the path is one that the user
    /// recognizes. A tool anchors the relative paths of its configuration
    /// at this directory, and it writes the files that it generates here.
    ProjectRoot
);
