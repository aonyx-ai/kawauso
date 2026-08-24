//! A subdirectory in which an application accepts its configuration file

use typed_fields::path;

path!(
    /// A subdirectory in which an application accepts its configuration file
    ///
    /// Some projects do not keep the configuration file in the directory that
    /// the search reads. `.github` and `.config` are the two conventions that
    /// occur most. A developer who names one of them adds it to every
    /// directory that the search reads.
    ///
    /// The value is a relative path that stays inside its directory. It can
    /// name more than one level, such as `.config/example`. A value that is
    /// absolute, or that leaves the directory, takes the search outside the
    /// directories that it reads. The search reports such a value instead.
    ///
    /// The path is stored as the developer gave it. The check happens when
    /// the search runs, so a value that a developer computes cannot make a
    /// constructor fail.
    Subdirectory
);
