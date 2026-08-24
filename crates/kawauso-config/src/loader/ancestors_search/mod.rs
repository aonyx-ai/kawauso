//! The search that reads the working directory and its ancestors

pub mod subdirectory;

pub use self::subdirectory::Subdirectory;
use crate::loader::ApplicationName;

/// The search of the working directory and its ancestors
///
/// The search needs the name of an application, and it accepts the
/// subdirectories in which that application also keeps its configuration
/// file. It carries what the search needs, and nothing that another source of
/// a configuration needs, so an option of this search cannot reach a loader
/// that reads a file at a path.
///
/// A name alone converts into a search, which is why
/// [`ancestors`][ancestors] accepts `"example"` as well as a value of this
/// type.
///
/// # Examples
///
/// A search that reads only the directories of the walk:
///
/// ```
/// use kawauso_config::AncestorsSearch;
///
/// let search = AncestorsSearch::from("example");
///
/// assert!(search.subdirectories().is_empty());
/// ```
///
/// A search that also reads `.github` in each of them:
///
/// ```
/// use kawauso_config::AncestorsSearch;
///
/// let search = AncestorsSearch::new("example").subdirectory(".github");
///
/// assert_eq!(search.subdirectories().len(), 1);
/// ```
///
/// [ancestors]: crate::Loader::ancestors
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct AncestorsSearch {
    /// The name of the application whose configuration file the search finds
    application: ApplicationName,

    /// The subdirectories that each directory of the walk also contributes
    ///
    /// The order is the order in which the developer named them, because it
    /// decides which location the search reads first.
    subdirectories: Vec<Subdirectory>,
}

impl AncestorsSearch {
    /// Creates a search for the configuration file of an application
    ///
    /// The search reads the directories of the walk and nothing else.
    /// [`subdirectory`][subdirectory] adds a subdirectory to each of them.
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
            subdirectories: Vec::new(),
        }
    }

    /// Adds a subdirectory that each directory of the walk also contributes
    ///
    /// The subdirectory is an addition. The directory itself keeps the
    /// location that it had before this call, and the search reads it first.
    /// A second call adds a second subdirectory, which the search reads after
    /// the first one.
    ///
    /// The value must be a relative path that stays inside the directory to
    /// which it belongs. A value that is absolute, or that leaves the
    /// directory, fails the search instead of this call, so that a value that
    /// the application computes cannot make a constructor fail.
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_config::AncestorsSearch;
    ///
    /// let search = AncestorsSearch::new("example")
    ///     .subdirectory(".github")
    ///     .subdirectory(".config");
    ///
    /// assert_eq!(search.subdirectories().len(), 2);
    /// ```
    #[must_use]
    pub fn subdirectory(mut self, subdirectory: impl Into<Subdirectory>) -> Self {
        self.subdirectories.push(subdirectory.into());

        self
    }

    /// Returns the name of the application whose configuration file the search finds
    pub fn application(&self) -> &ApplicationName {
        &self.application
    }

    /// Returns the subdirectories in the order in which the developer named them
    pub fn subdirectories(&self) -> &[Subdirectory] {
        &self.subdirectories
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

    // The order of the calls is the order in which the subdirectories win, so
    // the type must not sort them or remove a repeat.
    #[test]
    fn subdirectory_keeps_the_order_of_the_calls() {
        let search = AncestorsSearch::new("example")
            .subdirectory(".github")
            .subdirectory(".config");

        assert_eq!(
            search.subdirectories(),
            vec![Subdirectory::from(".github"), Subdirectory::from(".config")]
        );
    }
}
