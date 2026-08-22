//! A toolkit for building Rust applications
//!
//! Kawauso is a framework, and each part of it is also a crate that an
//! application can take on its own. This crate is the framework as a whole: it
//! re-exports the other crates as modules, so that an application needs one
//! dependency and one version requirement to reach all of them.
//!
//! A module carries the name of its crate without the prefix `kawauso-`, and
//! the module is that crate rather than a copy of it. `kawauso::config::Loader`
//! and `kawauso_config::Loader` are one type, so an application that depends
//! on this crate and a library that depends on the single crate can pass
//! values to each other.
//!
//! This crate has no features. It brings the whole framework, and an
//! application that wants a part of it depends on the crates that hold that
//! part.

/// Configuration files, from the crate `kawauso-config`
///
/// An application that wants this capability on its own, without the rest of
/// the framework, depends on `kawauso-config` and reaches the same types.
///
/// # Examples
///
/// ```
/// use std::collections::BTreeMap;
///
/// use kawauso::config::Loader;
///
/// let configuration: BTreeMap<String, u16> = Loader::contents("port = 8080").load()?;
///
/// assert_eq!(configuration["port"], 8080);
/// # Ok::<(), kawauso::config::error::LoadConfigurationError>(())
/// ```
// kawauso[impl facade.module]
// kawauso[impl facade.identity]
#[doc(inline)]
pub use kawauso_config as config;
