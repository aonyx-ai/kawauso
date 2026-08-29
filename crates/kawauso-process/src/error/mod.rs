//! Errors for the run of an external command
//!
//! Every fallible action of the crate returns its own error type, and every
//! error type lives in its own submodule. The variants of an error separate
//! the failures that a caller handles differently. The context that a caller
//! only reads, such as the command line or the status of a command, travels
//! in fields and in the message of the error.
//!
//! A command that ran and ended without success is no error of the run. Its
//! status is part of the result, and only the caller that requires success
//! turns the status into an error.

pub mod require_success;
pub mod run;

pub use self::require_success::RequireSuccessError;
pub use self::run::RunCommandError;
