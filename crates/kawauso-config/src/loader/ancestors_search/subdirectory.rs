//! A subdirectory in which an application accepts its configuration file

use typed_fields::path;

path!(
    /// A subdirectory in which an application accepts its configuration file
    ///
    /// A directory can be shared by many tools, and a tool then qualifies its
    /// file with its own name. GitHub, for example, keeps the file of every
    /// tool in the shared directory `.github`. A developer who names such a
    /// subdirectory adds it to every directory that the search reads.
    ///
    /// A directory that one application owns is a different kind of place, and
    /// a subdirectory does not describe it. The dot-config convention of
    /// [`dot_config`][dot-config] gives the application a directory of its
    /// own, where the file does not repeat the name of the application. Use
    /// that method for the directory that only the application uses.
    ///
    /// The value is a relative path that stays inside its directory. It can
    /// name more than one level. A value that is absolute, or that leaves the
    /// directory, takes the search outside the directories that it reads. The
    /// search reports such a value instead.
    ///
    /// The path is stored as the developer gave it. The check happens when
    /// the search runs, so a value that a developer computes cannot make a
    /// constructor fail.
    ///
    /// [dot-config]: crate::AncestorsSearch::dot_config
    Subdirectory
);
