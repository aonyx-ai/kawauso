//! The directory in which a platform keeps the local data of a user

use typed_fields::path;

path!(
    /// The directory in which the platform keeps the local data of a user
    ///
    /// The common platforms have one such directory, and it holds the files
    /// of all applications of the user. The data in it stays on one machine,
    /// and it does not roam to the other machines of the user on Windows.
    ///
    /// Where the directory is differs by platform, and it is below the home
    /// directory of the user. An instance of this type says only where the
    /// platform keeps the data. It does not say that the directory exists.
    LocalDataDirectory
);

impl LocalDataDirectory {
    /// Returns the local data directory of the platform for the current user
    ///
    /// On Linux, and on the other systems that follow the [XDG Base
    /// Directory Specification][xdg], the directory is the one that the
    /// environment variable `XDG_DATA_HOME` names, and `.local/share` in the
    /// home directory when the variable holds no absolute path. On macOS, it
    /// is `Library/Application Support` in the home directory, and the
    /// variable `XDG_DATA_HOME` has no effect. On Windows, it is the
    /// directory for the local application data of the user.
    ///
    /// The function determines the directory again on every call, so a
    /// change to `XDG_DATA_HOME` or to `HOME` takes effect at the next call.
    ///
    /// Returns [`None`] when the platform has no local data directory, or
    /// when the function cannot determine the home directory of the user.
    ///
    /// [xdg]: https://specifications.freedesktop.org/basedir/latest/
    // The rules of a platform belong to that platform, and a library keeps
    // up with them for us. This type wraps the answer, so that the library
    // stays out of the public API and the crate can replace it without a
    // breaking change.
    // project[impl data.base.macos]
    // project[impl data.base.windows]
    // project[impl data.base.xdg]
    // project[impl data.base.xdg.default]
    pub(super) fn of_platform() -> Option<Self> {
        dirs::data_local_dir().map(Self::new)
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
    // `XDG_DATA_HOME` exercises the variable, and a run without it exercises
    // the default.

    // project[verify data.base.macos]
    #[cfg(target_os = "macos")]
    #[test]
    fn of_platform_on_macos_returns_the_directory_of_apple() {
        let home = std::env::var_os("HOME").expect("the test needs a home directory");
        let expected = PathBuf::from(home)
            .join("Library")
            .join("Application Support");

        let directory = LocalDataDirectory::of_platform();

        assert_eq!(directory, Some(LocalDataDirectory::new(expected)));
    }

    // project[verify data.base.windows]
    #[cfg(windows)]
    #[test]
    fn of_platform_on_windows_returns_the_local_application_data() {
        let expected = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .expect("the test needs a directory for local application data");

        let directory = LocalDataDirectory::of_platform();

        assert_eq!(directory, Some(LocalDataDirectory::new(expected)));
    }

    // project[verify data.base.xdg]
    // project[verify data.base.xdg.default]
    #[cfg(not(any(target_os = "macos", windows)))]
    #[test]
    fn of_platform_on_xdg_systems_returns_the_data_home() {
        let home = std::env::var_os("HOME").expect("the test needs a home directory");
        let expected = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .unwrap_or_else(|| PathBuf::from(home).join(".local").join("share"));

        let directory = LocalDataDirectory::of_platform();

        assert_eq!(directory, Some(LocalDataDirectory::new(expected)));
    }
}
