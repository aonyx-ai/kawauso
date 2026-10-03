//! The data directory of a project

use typed_fields::path;

path!(
    /// The data directory of a project
    ///
    /// The directory is outside the repository, in the local data directory
    /// of the user: `<data>/<application>/projects/<identifier>`. An
    /// application writes the files of a project here that must not be in
    /// the working tree, such as a log for each run.
    ///
    /// The crate creates the directory, and it never deletes anything in it.
    /// Pruning and rotation of the files stay with the application. Two
    /// checkouts with the same identifier share the directory, so an
    /// application whose checkouts run at the same time names its files so
    /// that they do not collide.
    DataDirectory
);
