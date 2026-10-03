//! Errors of the fallible actions of the crate
//!
//! Every fallible action of the crate returns its own error type, and every
//! error type lives in its own submodule. The variants of an error separate
//! the failures that a caller handles differently. The context that a caller
//! only reads, such as a path or a list of markers, travels in fields and in
//! the message of the error.

pub mod create_data_directory;
pub mod discover;
pub mod load;
pub mod parse_application_name;
pub mod parse_project_identifier;

pub use self::create_data_directory::CreateDataDirectoryError;
pub use self::discover::DiscoverProjectError;
pub use self::load::LoadProjectError;
pub use self::parse_application_name::ParseApplicationNameError;
pub use self::parse_project_identifier::ParseProjectIdentifierError;
