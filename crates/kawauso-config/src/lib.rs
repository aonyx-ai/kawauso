//! Configuration files for Kawauso applications
//!
//! This crate loads the configuration files of Kawauso applications. Every
//! application finds, loads, and deserializes its configuration file in the
//! same way, and reports failures with the same clear errors.

use serde::de::DeserializeOwned;

/// Deserializes a TOML document into a type that the caller defines
///
/// The caller gives the contents of a configuration file and the type that
/// describes the structure of the file. The type must implement the
/// [`Deserialize`][deserialize] trait of serde, which the derive macro of
/// serde generates.
///
/// # Panics
///
/// This function panics if the contents are not valid TOML, or if the
/// document does not match the type. A later version of the crate returns an
/// error instead.
///
/// # Examples
///
/// ```
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Configuration {
///     port: u16,
/// }
///
/// let configuration: Configuration = kawauso_config::from_str("port = 8080");
///
/// assert_eq!(configuration.port, 8080);
/// ```
///
/// [deserialize]: https://docs.rs/serde/latest/serde/trait.Deserialize.html
// config[impl load.deserialize]
pub fn from_str<T>(contents: &str) -> T
where
    T: DeserializeOwned,
{
    toml::from_str(contents).expect("failed to deserialize the configuration")
}
