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
/// No project ever holds a value of this type. A project without a
/// configuration file reports `None`.
///
/// [project]: crate::Project
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Deserialize)]
#[non_exhaustive]
pub struct NoConfiguration {}
