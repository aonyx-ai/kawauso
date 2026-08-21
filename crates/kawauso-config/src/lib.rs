//! Configuration files for Kawauso applications
//!
//! This crate loads the configuration files of Kawauso applications. Every
//! application finds, loads, and deserializes its configuration file in the
//! same way, and reports failures with the same clear errors.
//!
//! [`Loader`] is the entry point of the crate. A constructor selects
//! the source of the configuration, and [`load`][load] deserializes the
//! configuration into a type that the caller defines.
//!
//! The source can be a search. An application whose configuration belongs to
//! a project uses [`ancestors`][ancestors], which walks from the working
//! directory up to the root of the file system. An application whose
//! configuration belongs to a user uses [`user`][user], which reads the
//! directory that the platform defines for the configuration of a user.
//!
//! ```
//! use serde::Deserialize;
//!
//! use kawauso_config::Loader;
//!
//! #[derive(Deserialize)]
//! struct Configuration {
//!     port: u16,
//! }
//!
//! let configuration: Configuration = Loader::contents("port = 8080").load()?;
//!
//! assert_eq!(configuration.port, 8080);
//! # Ok::<(), kawauso_config::error::LoadConfigurationError>(())
//! ```
//!
//! [ancestors]: Loader::ancestors
//! [load]: Loader::load
//! [user]: Loader::user

pub mod error;
pub mod loader;

pub use self::loader::Loader;
