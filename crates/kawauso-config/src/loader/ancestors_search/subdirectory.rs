//! A subdirectory in which an application accepts its configuration file

use typed_fields::path;

path!(
    /// A subdirectory in which an application accepts its configuration file
    ///
    /// A project does not always keep the configuration of a tool in the
    /// directory itself. `.github` and `.config` are the two conventions that
    /// occur most, and a developer who names one of them adds it to every
    /// directory that the search reads.
    ///
    /// The value is a relative path that stays inside the directory to which
    /// it belongs. It can name more than one level, such as
    /// `.config/example`. A value that is absolute, or that leaves the
    /// directory, moves the search to a place that the search never reaches
    /// on its own, and the search reports it instead.
    ///
    /// The path is stored as the developer gave it. The check happens when
    /// the search runs, so a value that a developer computes cannot make a
    /// constructor fail.
    Subdirectory
);
