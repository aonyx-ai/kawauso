//! Errors for the loading of configuration files
//!
//! Every fallible action of the crate returns its own error type, and every
//! error type lives in its own submodule. The variants of an error separate
//! the failures that a caller handles differently. The context that a caller
//! only reads, such as a path or a position, travels in fields and in the
//! message of the error.

pub mod deserialize;
pub mod discover;
pub mod load;

pub use self::deserialize::DeserializeConfigurationError;
pub use self::discover::DiscoverConfigurationError;
pub use self::load::LoadConfigurationError;
