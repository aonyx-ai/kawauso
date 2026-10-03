//! The rules for a name that becomes part of a path
//!
//! The name of an application becomes part of each path that a search reads.
//! This module states the rules that such a name follows.

/// The largest number of characters in a name
///
/// Most file systems accept a component of a path with up to 255 bytes. A
/// name holds only ASCII characters, so a character is one byte. The limit
/// fits a component that holds only the name, such as a directory. A file
/// such as `<name>.toml` adds to the name, so a long name can make such a
/// file longer than the limit of the file system.
const MAXIMUM_LENGTH: usize = 255;

/// The names that Windows reserves for a device, without a number
///
/// Windows opens the device instead of a file or a directory with one of
/// these names, in any case.
const DEVICES: [&str; 4] = ["AUX", "CON", "NUL", "PRN"];

/// The names that Windows reserves for a numbered device
///
/// Windows reserves each of these names followed by a digit, such as `COM1`
/// or `LPT0`, in any case.
const NUMBERED_DEVICES: [&str; 2] = ["COM", "LPT"];

/// Reports whether a value is a portable name
///
/// A portable name is exactly one normal component of a path on every
/// platform. It holds from 1 to 255 characters, each of which is an ASCII
/// letter, an ASCII digit, or `-`, and it is not the name of a device on
/// Windows.
///
/// The rules permit fewer names than any platform does. A later release can
/// permit more names without a change to the directory of a name that is
/// valid today. A rule that rejects a name that an earlier release accepted
/// would orphan the files in its directory instead.
///
/// The characters exclude the separators of every platform, the colon of a
/// drive prefix on Windows, and the dot, so `.` and `..` are no names. They
/// also exclude the characters that a file system changes or removes, such
/// as a trailing space on Windows, or that it compares in a normalized form,
/// such as the accented letters on macOS.
// config[impl name.characters]
// config[impl name.reserved]
pub(super) fn is_portable(value: &str) -> bool {
    let characters = !value.is_empty()
        && value.len() <= MAXIMUM_LENGTH
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-');

    characters && !is_device(value)
}

/// Reports whether a value is the name of a device on Windows
///
/// The comparison ignores case, because Windows does. A name that holds only
/// letters, digits, and `-` has no extension, so the rule does not need to
/// remove one before it compares.
fn is_device(value: &str) -> bool {
    let value = value.to_ascii_uppercase();

    if DEVICES.contains(&value.as_str()) {
        return true;
    }

    NUMBERED_DEVICES.iter().any(|device| {
        value.strip_prefix(device).is_some_and(|number| {
            number.len() == 1 && number.bytes().all(|byte| byte.is_ascii_digit())
        })
    })
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every
    // test would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    // config[verify name.characters]
    #[test]
    fn is_portable_beyond_the_maximum_length_returns_false() {
        assert!(!is_portable(&"a".repeat(256)));
    }

    // config[verify name.characters]
    #[test]
    fn is_portable_with_a_backslash_returns_false() {
        assert!(!is_portable(r"example\app"));
    }

    // config[verify name.characters]
    #[test]
    fn is_portable_with_a_colon_returns_false() {
        assert!(!is_portable("C:"));
    }

    // config[verify name.reserved]
    #[test]
    fn is_portable_with_a_device_in_lower_case_returns_false() {
        assert!(!is_portable("nul"));
    }

    // config[verify name.characters]
    #[test]
    fn is_portable_with_a_dot_returns_false() {
        assert!(!is_portable("."));
    }

    // config[verify name.characters]
    #[test]
    fn is_portable_with_a_hyphen_returns_true() {
        assert!(is_portable("example-app"));
    }

    // config[verify name.reserved]
    #[test]
    fn is_portable_with_a_letter_after_the_prefix_of_a_device_returns_true() {
        assert!(is_portable("LPTX"));
    }

    // config[verify name.characters]
    #[test]
    fn is_portable_with_a_letter_outside_ascii_returns_false() {
        assert!(!is_portable("café"));
    }

    // config[verify name.reserved]
    #[test]
    fn is_portable_with_a_numbered_device_in_mixed_case_returns_false() {
        assert!(!is_portable("Com9"));
    }

    // config[verify name.reserved]
    #[test]
    fn is_portable_with_a_numbered_device_returns_false() {
        assert!(!is_portable("LPT0"));
    }

    // A device name with more than one digit is no device, and a name that
    // only starts with a device name is none either.
    // config[verify name.reserved]
    #[test]
    fn is_portable_with_a_prefix_of_a_device_returns_true() {
        assert!(is_portable("COM10"));
    }

    // config[verify name.characters]
    #[test]
    fn is_portable_with_a_slash_returns_false() {
        assert!(!is_portable("example/app"));
    }

    // config[verify name.characters]
    #[test]
    fn is_portable_with_a_space_returns_false() {
        assert!(!is_portable("example "));
    }

    // config[verify name.reserved]
    #[test]
    fn is_portable_with_a_word_that_starts_like_a_device_returns_true() {
        assert!(is_portable("console"));
    }

    // config[verify name.characters]
    #[test]
    fn is_portable_with_an_absolute_path_returns_false() {
        assert!(!is_portable("/example"));
    }

    // config[verify name.characters]
    #[test]
    fn is_portable_with_an_empty_value_returns_false() {
        assert!(!is_portable(""));
    }

    // config[verify name.characters]
    #[test]
    fn is_portable_with_an_underscore_returns_false() {
        assert!(!is_portable("example_app"));
    }

    // config[verify name.characters]
    #[test]
    fn is_portable_with_letters_and_digits_returns_true() {
        assert!(is_portable("Example2"));
    }

    // config[verify name.characters]
    #[test]
    fn is_portable_with_the_maximum_length_returns_true() {
        assert!(is_portable(&"a".repeat(255)));
    }

    // config[verify name.characters]
    #[test]
    fn is_portable_with_two_dots_returns_false() {
        assert!(!is_portable(".."));
    }
}
