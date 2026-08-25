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
/// Projects, from the crate `kawauso-project`
///
/// An application that wants this capability on its own, without the rest of
/// the framework, depends on `kawauso-project` and reaches the same types.
///
/// # Examples
///
/// ```no_run
/// use kawauso::project::Project;
/// use kawauso::project::ProjectSearch;
///
/// // Finds the first directory at or above the working directory that holds
/// // `.config/kawauso.toml` or `.git`
/// let search = ProjectSearch::new("kawauso").marker(".git");
/// let project = Project::discover(search)?;
///
/// assert!(project.root().get().is_absolute());
/// # Ok::<(), kawauso::project::error::DiscoverProjectError>(())
/// ```
// kawauso[impl facade.module]
// kawauso[impl facade.identity]
#[doc(inline)]
pub use kawauso_project as project;
