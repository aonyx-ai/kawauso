//! Errors for the search for a project
//!
//! Every fallible action of the crate returns its own error type, and every
//! error type lives in its own submodule. The variants of an error separate
//! the failures that a caller handles differently. The context that a caller
//! only reads, such as a path or a list of markers, travels in fields and in
//! the message of the error.

pub mod discover;
pub mod load;

pub use self::discover::DiscoverProjectError;
pub use self::load::LoadProjectError;
