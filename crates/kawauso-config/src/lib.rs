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
//! [load]: Loader::load

pub mod error;
pub mod loader;

pub use self::loader::Loader;
