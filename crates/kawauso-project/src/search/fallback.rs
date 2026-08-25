//! What a search reports when no marker matches

/// What a search reports when no marker matches
///
/// A walk that reaches the root of the file system without a match has
/// found no project. The error is the right default: a tool that generates
/// files must not write them into a directory that nothing marks as a
/// project. Some tools run outside any project as well, with the default of
/// their configuration, and for them the start directory is the project.
///
/// A later release can add variants. Match with a wildcard arm.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[non_exhaustive]
pub enum Fallback {
    /// The search reports an error that names the start and the markers
    #[default]
    Error,

    /// The search reports the start directory as a project without a marker
    Start,
}
