//! The directory in which a platform keeps the configuration of a user

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

impl ConfigurationDirectory {
    /// Returns the configuration directory of the platform for the current user
    ///
    /// On Linux, and on the other systems that follow the [XDG Base
    /// Directory Specification][xdg], the directory is the one that the
    /// environment variable `XDG_CONFIG_HOME` names, and `.config` in the
    /// home directory when the variable holds no absolute path. On macOS, it
    /// is `Library/Application Support` in the home directory, and the
    /// variable of the other platform has no effect. On Windows, it is the
    /// directory for the roaming application data of the user.
    ///
    /// The environment of the process names the directory, and this function
    /// reads that environment on every call.
    ///
    /// Returns [`None`] when the environment does not name the directory,
    /// which happens when the process runs without a home directory.
    ///
    /// [xdg]: https://specifications.freedesktop.org/basedir/latest/
    // The rules of a platform belong to that platform, and a library keeps
    // up with them for us. This type wraps the answer, so that the library
    // stays out of the public API and the crate can replace it without a
    // breaking change.
    // config[impl discover.user.macos]
    // config[impl discover.user.windows]
    // config[impl discover.user.xdg]
    // config[impl discover.user.xdg.default]
    pub fn of_platform() -> Option<Self> {
        dirs::config_dir().map(Self::new)
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every
    // test would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use std::path::PathBuf;

    use super::*;

    // Each of these tests states the rule of one platform, and it runs only on
    // that platform. A test cannot set an environment variable to reach the
    // rule of another one, because a process shares its environment with every
    // thread in it and tests run in parallel. The rule of a platform therefore
    // stays unverified until a machine of that platform runs the tests.
    //
    // The environment that the test got decides which branch of a rule it
    // reaches. On a system that follows the XDG specification, a run with
    // `XDG_CONFIG_HOME` exercises the variable, and a run without it exercises
    // the default.

    // config[verify discover.user.macos]
    #[cfg(target_os = "macos")]
    #[test]
    fn of_platform_on_macos_returns_the_directory_of_apple() {
        let home = std::env::var_os("HOME").expect("the test needs a home directory");
        let expected = PathBuf::from(home)
            .join("Library")
            .join("Application Support");

        let directory = ConfigurationDirectory::of_platform();

        assert_eq!(directory, Some(ConfigurationDirectory::new(expected)));
    }

    // config[verify discover.user.windows]
    #[cfg(windows)]
    #[test]
    fn of_platform_on_windows_returns_the_roaming_application_data() {
        let expected = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .expect("the test needs a directory for roaming application data");

        let directory = ConfigurationDirectory::of_platform();

        assert_eq!(directory, Some(ConfigurationDirectory::new(expected)));
    }

    // config[verify discover.user.xdg]
    // config[verify discover.user.xdg.default]
    #[cfg(not(any(target_os = "macos", windows)))]
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
