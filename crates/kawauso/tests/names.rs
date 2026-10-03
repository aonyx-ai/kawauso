//! Agreement of the rules for the name of an application
//!
//! `kawauso-config` and `kawauso-project` each check the name of an
//! application, and each crate keeps a private copy of the rules. An
//! application that uses both crates gives both the same name, and on macOS
//! both crates put it into the same directory. A name that one crate accepts
//! and the other rejects would therefore stop the application.
//!
//! The facade is the one crate that depends on both, so it holds the test that
//! the two copies agree.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

/// Characters on both sides of the rule on characters
///
/// The set holds a letter in each case, a digit, the hyphen, and characters
/// that the rule rejects: the dot, separators, a colon, a space, an
/// underscore, and a letter outside ASCII.
const CHARACTERS: [char; 11] = ['a', 'Z', '0', '-', '.', '_', '/', '\\', ':', ' ', 'é'];

/// The names of devices on Windows, and the prefixes of numbered devices
const DEVICES: [&str; 6] = ["AUX", "CON", "NUL", "PRN", "COM", "LPT"];

/// Values on both sides of each rule of a name
///
/// The values are every string of up to three characters from
/// [`CHARACTERS`], each device in three cases with each suffix that the rule
/// on devices tells apart, and the lengths at the limit.
fn values() -> Vec<String> {
    let mut values = vec![String::new()];

    let mut shorter = vec![String::new()];
    for _ in 0..3 {
        shorter = shorter
            .iter()
            .flat_map(|prefix| CHARACTERS.map(|character| format!("{prefix}{character}")))
            .collect();
        values.extend(shorter.iter().cloned());
    }

    let suffixes = ["", "0", "9", "10", "X", "-"];
    for device in DEVICES {
        let cases = [
            device.to_owned(),
            device.to_ascii_lowercase(),
            format!("{}{}", &device[..1], device[1..].to_ascii_lowercase()),
        ];
        for case in cases {
            for suffix in suffixes {
                values.push(format!("{case}{suffix}"));
            }
        }
    }

    for length in 254..=256 {
        values.push("a".repeat(length));
    }

    values
}

#[test]
fn application_name_obeys_the_same_rules_in_config_and_project() {
    let values = values();

    let disagreements: Vec<&String> = values
        .iter()
        .filter(|value| {
            let config = value.parse::<kawauso::config::ApplicationName>().is_ok();
            let project = value
                .parse::<kawauso::project::project::ApplicationName>()
                .is_ok();

            config != project
        })
        .collect();

    assert_eq!(disagreements, Vec::<&String>::new());
}
