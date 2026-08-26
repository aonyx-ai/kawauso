//! The directory of a project

use typed_fields::path;

path!(
    /// The directory of a project
    ///
    /// The path is canonical: it is absolute, it holds no `.` or `..`
    /// component, and its symbolic links are resolved. It is the directory of
    /// the walk in which a marker matched. The search observed the directory,
    /// and it did not derive the directory from the path of a file.
    ///
    /// An application anchors the relative paths of its resources at this
    /// directory, and it writes the files that it creates here. A path that
    /// the application joins onto this directory therefore reaches the entry
    /// that the walk saw.
    ///
    /// The path can differ from the path that the user typed, because a
    /// symbolic link on the way carries another name. An application that
    /// reports the path of its user keeps that path itself.
    ProjectRoot
);
