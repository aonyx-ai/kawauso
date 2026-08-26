//! Whether a search names a marker
//!
//! A search without a marker cannot find a project: the walk tests nothing in
//! every directory and reaches the root of the file system. The state of a
//! search carries this fact in its type, so that the compiler reports the
//! mistake, and not the first run of the application.
//!
//! The state is a type parameter of the search, and this module holds the two
//! types that fill it.

pub mod marked;
pub mod unmarked;

pub use self::marked::Marked;
pub use self::unmarked::Unmarked;

/// Whether a search names a marker
///
/// The crate implements this trait for [`Unmarked`] and [`Marked`], and for
/// no other type. It exists to bound the state of a search, so that a
/// signature which takes any state says so, and a signature which takes a
/// search that can discover a project names [`Marked`].
///
/// The trait has no methods. A state carries no data, only a fact about the
/// search that holds it.
pub trait State {}

impl State for Unmarked {}

impl State for Marked {}
