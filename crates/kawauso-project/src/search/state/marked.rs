//! The state of a search that names at least one marker

/// The state of a search that names at least one marker
///
/// A search reaches this state with its first marker, and every further
/// marker keeps it. Only a search in this state can discover a project.
///
/// The type has no value that carries information; it only names the state of
/// a search.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[non_exhaustive]
pub struct Marked;
