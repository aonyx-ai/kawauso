//! The directory in which a platform keeps the configuration of a user
//!
//! A caller never names the directory of a platform. A constructor of the
//! loader takes what it wraps, and no error reports it. The module is
//! therefore private, which keeps it out of the public API and leaves the
//! crate free to change it.

use std::ffi::OsString;
use std::path::PathBuf;

use typed_fields::path;

path!(
    /// The directory in which the platform keeps the configuration of a user
    ///
    /// Every platform has one such directory, and it holds the files of all
    /// applications of the user. An application therefore gets a directory
    /// of its own in it, and not a single file.
    ///
    /// Where the directory is differs by platform, and the environment of
    /// the process names it. The user controls that environment, so an
    /// instance of this type says only where the platform keeps the
    /// configuration. It does not say that the directory exists.
    ConfigurationDirectory
);

#[bon::bon]
impl ConfigurationDirectory {
    /// Returns the configuration directory of the platform for the current user
    ///
    /// On Linux and other systems that follow the [XDG Base Directory
    /// Specification][xdg], the directory is the one that `XDG_CONFIG_HOME`
    /// names. When the variable holds no absolute path, the directory is
    /// `.config` in the home directory. On macOS, it is `Library/Application
    /// Support` in the home directory; the `XDG_CONFIG_HOME` variable has no
    /// effect. On Windows, it is the directory for roaming application data.
    ///
    /// The environment of the process names the directory, and this function
    /// reads that environment on every call.
    ///
    /// Returns [`None`] when the environment does not name the directory,
    /// which happens when the process runs without a home directory.
    ///
    /// A library finds the home directory of the user and the known folders
    /// of Windows, because the platform decides where they are. The rules of
    /// XDG and of Apple then give the directory from the home directory and
    /// one variable. `from_environment` applies them to the values that it
    /// gets as arguments, so that a test can give each case to it.
    ///
    /// [xdg]: https://specifications.freedesktop.org/basedir/latest/
    pub(super) fn of_platform() -> Option<Self> {
        if cfg!(windows) {
            dirs::config_dir().map(Self::new)
        } else {
            Self::from_environment()
                .maybe_xdg_config_home(std::env::var_os("XDG_CONFIG_HOME"))
                .maybe_home(dirs::home_dir())
                .call()
        }
    }

    /// Returns the configuration directory that the convention of Apple gives
    ///
    /// The directory is `Library/Application Support` in `home`. The
    /// convention of Apple does not use `XDG_CONFIG_HOME`, so the value of
    /// `_xdg_config_home` has no effect. The parameter is there so that this
    /// function and the one for XDG have the same signature.
    ///
    /// Returns [`None`] when `home` is [`None`].
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    #[builder]
    fn from_environment(_xdg_config_home: Option<OsString>, home: Option<PathBuf>) -> Option<Self> {
        home.map(|home| Self::new(home.join("Library").join("Application Support")))
    }

    /// Returns the configuration directory that the XDG specification gives
    ///
    /// The directory is `xdg_config_home` when it holds an absolute path.
    /// When it is [`None`], empty, or relative, the directory is `.config` in
    /// `home`. The specification tells an application to ignore a relative
    /// path, and an empty path is relative.
    ///
    /// Returns [`None`] when `xdg_config_home` holds no absolute path and
    /// `home` is [`None`].
    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    #[builder]
    fn from_environment(xdg_config_home: Option<OsString>, home: Option<PathBuf>) -> Option<Self> {
        xdg_config_home
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .or_else(|| home.map(|home| home.join(".config")))
            .map(Self::new)
    }
}

/// The rule of each platform is in `from_environment`
///
/// That function gets the value of `XDG_CONFIG_HOME` and the home directory
/// as arguments. Its tests give each case to it directly. A test that sets an
/// environment variable changes it for every thread of the process, and the
/// tests run in parallel. The tests of a rule run only on the platforms that
/// use it.
///
/// The tests of `of_platform` check that it reads the environment of the
/// process and gives it to the rule. Their expected value comes from the
/// same environment, so each run reaches only the case that the
/// environment gives.
#[cfg(test)]
mod tests {
    #![allow(
        clippy::missing_panics_doc,
        reason = "An assertion in a test panics by design. A `# Panics` section on every test would repeat that and give the reader no information."
    )]

    use super::*;

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    #[test]
    fn from_environment_on_macos_with_a_variable_returns_the_directory_of_apple() {
        let xdg_config_home = Some(OsString::from("/xdg/config"));
        let home = Some(PathBuf::from("/Users/user"));

        let directory = ConfigurationDirectory::from_environment()
            .maybe_xdg_config_home(xdg_config_home)
            .maybe_home(home)
            .call();

        assert_eq!(
            directory,
            Some(ConfigurationDirectory::new(PathBuf::from(
                "/Users/user/Library/Application Support"
            )))
        );
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    #[test]
    fn from_environment_on_macos_without_a_home_returns_none() {
        let xdg_config_home = Some(OsString::from("/xdg/config"));

        let directory = ConfigurationDirectory::from_environment()
            .maybe_xdg_config_home(xdg_config_home)
            .maybe_home(None)
            .call();

        assert_eq!(directory, None);
    }

    #[cfg(not(any(target_os = "macos", target_os = "ios", windows)))]
    #[test]
    fn from_environment_on_xdg_systems_with_a_relative_variable_returns_the_default() {
        let xdg_config_home = Some(OsString::from("xdg/config"));
        let home = Some(PathBuf::from("/home/user"));

        let directory = ConfigurationDirectory::from_environment()
            .maybe_xdg_config_home(xdg_config_home)
            .maybe_home(home)
            .call();

        assert_eq!(
            directory,
            Some(ConfigurationDirectory::new(PathBuf::from(
                "/home/user/.config"
            )))
        );
    }

    #[cfg(not(any(target_os = "macos", target_os = "ios", windows)))]
    #[test]
    fn from_environment_on_xdg_systems_with_an_absolute_variable_returns_the_variable() {
        let xdg_config_home = Some(OsString::from("/xdg/config"));
        let home = Some(PathBuf::from("/home/user"));

        let directory = ConfigurationDirectory::from_environment()
            .maybe_xdg_config_home(xdg_config_home)
            .maybe_home(home)
            .call();

        assert_eq!(
            directory,
            Some(ConfigurationDirectory::new(PathBuf::from("/xdg/config")))
        );
    }

    #[cfg(not(any(target_os = "macos", target_os = "ios", windows)))]
    #[test]
    fn from_environment_on_xdg_systems_with_an_empty_variable_returns_the_default() {
        let xdg_config_home = Some(OsString::new());
        let home = Some(PathBuf::from("/home/user"));

        let directory = ConfigurationDirectory::from_environment()
            .maybe_xdg_config_home(xdg_config_home)
            .maybe_home(home)
            .call();

        assert_eq!(
            directory,
            Some(ConfigurationDirectory::new(PathBuf::from(
                "/home/user/.config"
            )))
        );
    }

    #[cfg(not(any(target_os = "macos", target_os = "ios", windows)))]
    #[test]
    fn from_environment_on_xdg_systems_without_a_home_returns_none() {
        let directory = ConfigurationDirectory::from_environment()
            .maybe_xdg_config_home(None)
            .maybe_home(None)
            .call();

        assert_eq!(directory, None);
    }

    #[cfg(not(any(target_os = "macos", target_os = "ios", windows)))]
    #[test]
    fn from_environment_on_xdg_systems_without_a_variable_returns_the_default() {
        let home = Some(PathBuf::from("/home/user"));

        let directory = ConfigurationDirectory::from_environment()
            .maybe_xdg_config_home(None)
            .maybe_home(home)
            .call();

        assert_eq!(
            directory,
            Some(ConfigurationDirectory::new(PathBuf::from(
                "/home/user/.config"
            )))
        );
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    #[test]
    fn of_platform_on_macos_returns_the_directory_of_apple() {
        let home = std::env::var_os("HOME").expect("the test needs a home directory");
        let expected = PathBuf::from(home)
            .join("Library")
            .join("Application Support");

        let directory = ConfigurationDirectory::of_platform();

        assert_eq!(directory, Some(ConfigurationDirectory::new(expected)));
    }

    #[cfg(windows)]
    #[test]
    fn of_platform_on_windows_returns_the_roaming_application_data() {
        let expected = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .expect("the test needs a directory for roaming application data");

        let directory = ConfigurationDirectory::of_platform();

        assert_eq!(directory, Some(ConfigurationDirectory::new(expected)));
    }

    #[cfg(not(any(target_os = "macos", target_os = "ios", windows)))]
    #[test]
    fn of_platform_on_xdg_systems_returns_the_configuration_home() {
        let home = std::env::var_os("HOME").expect("the test needs a home directory");
        let expected = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .unwrap_or_else(|| PathBuf::from(home).join(".config"));

        let directory = ConfigurationDirectory::of_platform();

        assert_eq!(directory, Some(ConfigurationDirectory::new(expected)));
    }
}
