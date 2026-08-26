//! What a search reports when no marker matches

/// What a search reports when no marker matches
///
/// A walk that reaches the root of the file system found no project. An error
/// is the safe answer, because an application that creates files must not
/// write them into a directory that nothing identifies as a project. This is
/// why [`Error`][error] is the default.
///
/// Some applications also run outside a project, with default settings. Their
/// developer selects [`Start`][start], and the search reports the start
/// directory as a project without a marker.
///
/// A later release can add variants. Match with a wildcard arm.
///
/// [error]: Fallback::Error
/// [start]: Fallback::Start
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[non_exhaustive]
pub enum Fallback {
    /// The search reports an error that names the start and the markers
    #[default]
    Error,

    /// The search reports the start directory as a project without a marker
    Start,
}
