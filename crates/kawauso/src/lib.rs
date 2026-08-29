#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]

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
/// The project that an application runs in, from the crate
/// `kawauso-project`
///
/// An application that wants this capability on its own, without the rest of
/// the framework, depends on `kawauso-project` and reaches the same types.
///
/// # Examples
///
/// ```
/// use kawauso::project::Search;
///
/// let search = Search::working_directory().marker(".git");
///
/// assert_eq!(search.markers().len(), 1);
/// ```
// kawauso[impl facade.module]
// kawauso[impl facade.identity]
#[doc(inline)]
pub use kawauso_project as project;
