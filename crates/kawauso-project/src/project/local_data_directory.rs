//! The directory in which a platform keeps the local data of a user

use std::ffi::OsString;
use std::path::PathBuf;

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
    // A library finds the home directory of the user and the known folders
    // of Windows, because the platform decides where they are. The rules of
    // XDG and of Apple then give the directory from the home directory and
    // one variable. `from_environment` applies them to the values that it
    // gets as arguments, so that a test can give each case to it.
    pub(super) fn of_platform() -> Option<Self> {
        if cfg!(windows) {
            dirs::data_local_dir().map(Self::new)
        } else {
            Self::from_environment(std::env::var_os("XDG_DATA_HOME"), dirs::home_dir())
        }
    }

    /// Returns the local data directory that the convention of Apple gives
    ///
    /// The directory is `Library/Application Support` in `home`. The
    /// convention of Apple does not use `XDG_DATA_HOME`, so the value of
    /// `_xdg_data_home` has no effect. The parameter is there so that this
    /// function and the one for XDG have the same signature.
    ///
    /// Returns [`None`] when `home` is [`None`].
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    fn from_environment(_xdg_data_home: Option<OsString>, home: Option<PathBuf>) -> Option<Self> {
        home.map(|home| Self::new(home.join("Library").join("Application Support")))
    }

    /// Returns the local data directory that the XDG specification gives
    ///
    /// The directory is `xdg_data_home` when it holds an absolute path. When
    /// it is [`None`], empty, or relative, the directory is `.local/share` in
    /// `home`. The specification tells an application to ignore a relative
    /// path, and an empty path is relative.
    ///
    /// Returns [`None`] when `xdg_data_home` holds no absolute path and
    /// `home` is [`None`].
    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    fn from_environment(xdg_data_home: Option<OsString>, home: Option<PathBuf>) -> Option<Self> {
        xdg_data_home
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .or_else(|| home.map(|home| home.join(".local").join("share")))
            .map(Self::new)
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every
    // test would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    // The rule of each platform is in `from_environment`, which gets the
    // value of `XDG_DATA_HOME` and the home directory as arguments. Its
    // tests give each case to it directly. A test that sets an environment
    // variable changes it for every thread of the process, and the tests run
    // in parallel. The tests of a rule run only on the platforms that use
    // it.
    //
    // The tests of `of_platform` check that it reads the environment of the
    // process and gives it to the rule. Their expected value comes from the
    // same environment, so each run reaches only the case that the
    // environment gives.

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    #[test]
    fn from_environment_on_macos_with_a_variable_returns_the_directory_of_apple() {
        let xdg_data_home = Some(OsString::from("/xdg/data"));
        let home = Some(PathBuf::from("/Users/user"));

        let directory = LocalDataDirectory::from_environment(xdg_data_home, home);

        assert_eq!(
            directory,
            Some(LocalDataDirectory::new(PathBuf::from(
                "/Users/user/Library/Application Support"
            )))
        );
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    #[test]
    fn from_environment_on_macos_without_a_home_returns_none() {
        let xdg_data_home = Some(OsString::from("/xdg/data"));

        let directory = LocalDataDirectory::from_environment(xdg_data_home, None);

        assert_eq!(directory, None);
    }

    #[cfg(not(any(target_os = "macos", target_os = "ios", windows)))]
    #[test]
    fn from_environment_on_xdg_systems_with_a_relative_variable_returns_the_default() {
        let xdg_data_home = Some(OsString::from("xdg/data"));
        let home = Some(PathBuf::from("/home/user"));

        let directory = LocalDataDirectory::from_environment(xdg_data_home, home);

        assert_eq!(
            directory,
            Some(LocalDataDirectory::new(PathBuf::from(
                "/home/user/.local/share"
            )))
        );
    }

    #[cfg(not(any(target_os = "macos", target_os = "ios", windows)))]
    #[test]
    fn from_environment_on_xdg_systems_with_an_absolute_variable_returns_the_variable() {
        let xdg_data_home = Some(OsString::from("/xdg/data"));
        let home = Some(PathBuf::from("/home/user"));

        let directory = LocalDataDirectory::from_environment(xdg_data_home, home);

        assert_eq!(
            directory,
            Some(LocalDataDirectory::new(PathBuf::from("/xdg/data")))
        );
    }

    #[cfg(not(any(target_os = "macos", target_os = "ios", windows)))]
    #[test]
    fn from_environment_on_xdg_systems_with_an_empty_variable_returns_the_default() {
        let xdg_data_home = Some(OsString::new());
        let home = Some(PathBuf::from("/home/user"));

        let directory = LocalDataDirectory::from_environment(xdg_data_home, home);

        assert_eq!(
            directory,
            Some(LocalDataDirectory::new(PathBuf::from(
                "/home/user/.local/share"
            )))
        );
    }

    #[cfg(not(any(target_os = "macos", target_os = "ios", windows)))]
    #[test]
    fn from_environment_on_xdg_systems_without_a_home_returns_none() {
        let directory = LocalDataDirectory::from_environment(None, None);

        assert_eq!(directory, None);
    }

    #[cfg(not(any(target_os = "macos", target_os = "ios", windows)))]
    #[test]
    fn from_environment_on_xdg_systems_without_a_variable_returns_the_default() {
        let home = Some(PathBuf::from("/home/user"));

        let directory = LocalDataDirectory::from_environment(None, home);

        assert_eq!(
            directory,
            Some(LocalDataDirectory::new(PathBuf::from(
                "/home/user/.local/share"
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

        let directory = LocalDataDirectory::of_platform();

        assert_eq!(directory, Some(LocalDataDirectory::new(expected)));
    }

    #[cfg(windows)]
    #[test]
    fn of_platform_on_windows_returns_the_local_application_data() {
        let expected = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .expect("the test needs a directory for local application data");

        let directory = LocalDataDirectory::of_platform();

        assert_eq!(directory, Some(LocalDataDirectory::new(expected)));
    }

    #[cfg(not(any(target_os = "macos", target_os = "ios", windows)))]
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
