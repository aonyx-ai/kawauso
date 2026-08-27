//! The search that reads the working directory and its ancestors

mod location;
pub mod subdirectory;

pub(crate) use self::location::Location;
pub use self::subdirectory::Subdirectory;
use crate::loader::ApplicationName;

/// The search of the working directory and its ancestors
///
/// The search needs the name of an application. It also needs the locations
/// in which that application keeps its configuration file, such as a
/// subdirectory that the developer names.
///
/// The type holds the options of this one search. An option therefore cannot
/// reach a loader that reads a file at a path, where a location has no
/// meaning.
///
/// A name alone becomes a search. [`ancestors`][ancestors] therefore accepts
/// `"example"` as well as a value of this type.
///
/// # Examples
///
/// A name alone, which reads the directories themselves:
///
/// ```
/// use kawauso_config::AncestorsSearch;
///
/// let search = AncestorsSearch::from("example");
///
/// assert_eq!(search.application().get(), "example");
/// ```
///
/// A search that adds a location:
///
/// ```no_run
/// use serde::Deserialize;
///
/// use kawauso_config::AncestorsSearch;
/// use kawauso_config::Loader;
///
/// #[derive(Deserialize)]
/// struct Configuration {
///     port: u16,
/// }
///
/// // In each directory, reads `example.toml`, then `.github/example.toml`
/// let search = AncestorsSearch::new("example").subdirectory(".github");
/// let configuration: Configuration = Loader::ancestors(search).load()?;
/// # Ok::<(), kawauso_config::error::LoadConfigurationError>(())
/// ```
///
/// [ancestors]: crate::Loader::ancestors
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct AncestorsSearch {
    /// The name of the application whose configuration file the search finds
    application: ApplicationName,

    /// The locations that the search reads in each directory
    ///
    /// The order is the order in which the developer named them, because it
    /// decides which location the search reads first.
    locations: Vec<Location>,
}

impl AncestorsSearch {
    /// Creates a search for the configuration file of an application
    ///
    /// The search reads the working directory and its ancestors, and nothing
    /// else. [`subdirectory`][subdirectory] adds a subdirectory to each of
    /// these directories. A name alone also becomes a search through
    /// [`From`], which gives the same result as this method.
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_config::AncestorsSearch;
    ///
    /// let search = AncestorsSearch::new("example");
    ///
    /// assert_eq!(search.application().get(), "example");
    /// ```
    ///
    /// [subdirectory]: AncestorsSearch::subdirectory
    pub fn new(application: impl Into<ApplicationName>) -> Self {
        Self {
            application: application.into(),
            locations: Vec::new(),
        }
    }

    /// Adds a subdirectory that the search reads in each directory
    ///
    /// The subdirectory is an addition. Each directory keeps the location
    /// that it had before this call, and the search reads that location
    /// first. A second call adds a second location, which the search reads
    /// after the first one.
    ///
    /// The value must be a relative path that stays inside its directory. A
    /// value that is absolute, or that leaves the directory, fails the search
    /// and not this call. A value that the application computes therefore
    /// cannot make a constructor fail.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use serde::Deserialize;
    ///
    /// use kawauso_config::AncestorsSearch;
    /// use kawauso_config::Loader;
    ///
    /// #[derive(Deserialize)]
    /// struct Configuration {
    ///     port: u16,
    /// }
    ///
    /// // In each directory, reads `example.toml`, then `.github/example.toml`
    /// let search = AncestorsSearch::new("example").subdirectory(".github");
    /// let configuration: Configuration = Loader::ancestors(search).load()?;
    /// # Ok::<(), kawauso_config::error::LoadConfigurationError>(())
    /// ```
    #[must_use]
    pub fn subdirectory(mut self, subdirectory: impl Into<Subdirectory>) -> Self {
        self.locations
            .push(Location::Subdirectory(subdirectory.into()));

        self
    }

    /// Returns the name of the application whose configuration file the search finds
    pub fn application(&self) -> &ApplicationName {
        &self.application
    }

    /// Returns the locations in the order in which the developer named them
    pub(crate) fn locations(&self) -> &[Location] {
        &self.locations
    }
}

/// Creates a search from the name of an application
impl From<ApplicationName> for AncestorsSearch {
    fn from(application: ApplicationName) -> Self {
        Self::new(application)
    }
}

/// Creates a search from the name of an application
impl From<&str> for AncestorsSearch {
    fn from(application: &str) -> Self {
        Self::new(application)
    }
}

/// Creates a search from the name of an application
impl From<String> for AncestorsSearch {
    fn from(application: String) -> Self {
        Self::new(application)
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every
    // test would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    // An application that already holds its name in the newtype must not have
    // to take the string out of it again.
    #[test]
    fn from_application_name_keeps_the_name() {
        let search = AncestorsSearch::from(ApplicationName::new("example"));

        assert_eq!(search.application().get(), "example");
    }

    // A name that the application builds at runtime arrives as a `String`,
    // and the search takes it without a borrow.
    #[test]
    fn from_string_keeps_the_name() {
        let search = AncestorsSearch::from(String::from("example"));

        assert_eq!(search.application().get(), "example");
    }

    // The order of the calls is the order in which the locations win, so the
    // type must not sort them or remove a repeat.
    #[test]
    fn subdirectory_keeps_the_order_of_the_calls() {
        let search = AncestorsSearch::new("example")
            .subdirectory(".github")
            .subdirectory(".config");

        assert_eq!(
            search.locations(),
            vec![
                Location::Subdirectory(Subdirectory::from(".github")),
                Location::Subdirectory(Subdirectory::from(".config"))
            ]
        );
    }
}
