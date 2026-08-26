//! The state of a search that names no marker

/// The state of a search that names no marker
///
/// A search in this state cannot discover a project, because a walk without a
/// marker tests nothing and reaches the root of the file system. The type
/// exists so that the compiler reports this, and not the first run of the
/// application.
///
/// A constructor of a search returns this state, and the first marker leaves
/// it. The type has no value that carries information; it only names the
/// state of a search.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[non_exhaustive]
pub struct Unmarked;
