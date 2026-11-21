//! The configuration of a project that has none

use serde::Deserialize;

/// The configuration of a project that has none
///
/// A project whose developer named no configuration file still needs a type
/// in the place where the configuration goes. This type fills that place, and
/// it is the default of [`Project`][project]. An application that wants only
/// the directory of its project therefore writes `Project` and names no type
/// of its own.
///
/// The type holds no field, and it accepts no field. A project that reads a
/// file with contents into this type therefore fails. This tells the developer
/// that the application reads a file that it never described. An
/// application that wants only the directory declares this with
/// [`without_configuration`][without-configuration]. The project then reads
/// no file at all.
///
/// # Examples
///
/// ```
/// use kawauso_project::project::NoConfiguration;
///
/// let result = toml::from_str::<NoConfiguration>("port = 8080");
///
/// assert!(result.is_err());
/// ```
///
/// [project]: crate::Project
/// [without-configuration]: crate::project::ProjectBuilder::without_configuration
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Deserialize)]
#[non_exhaustive]
#[serde(deny_unknown_fields)]
pub struct NoConfiguration {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_configuration_is_send_sync_and_unpin() {
        fn assert_send_sync_and_unpin<T: Send + Sync + Unpin>() {}

        assert_send_sync_and_unpin::<NoConfiguration>();
    }
}
