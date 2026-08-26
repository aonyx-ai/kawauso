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
/// file with contents into this type therefore fails, which tells the
/// developer that the application reads a file that it never described. An
/// application that wants only the directory declares this with
/// [`without_configuration`][without-configuration], and the project then
/// reads no file at all.
///
/// [project]: crate::Project
/// [without-configuration]: crate::project::ProjectBuilder::without_configuration
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Deserialize)]
#[non_exhaustive]
#[serde(deny_unknown_fields)]
pub struct NoConfiguration {}
