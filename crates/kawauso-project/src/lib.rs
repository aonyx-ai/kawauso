#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]

/// Returns the sum of two unsigned 64-bit integers
///
/// The crate has no capability of its own yet. This function is a placeholder
/// that gives the crate something to build and to test. The first capability
/// of the crate removes it.
///
/// # Panics
///
/// Panics when the sum does not fit into a `u64` and the build checks for an
/// overflow.
// project[impl placeholder.add]
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every
    // test would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    // project[verify placeholder.add]
    #[test]
    fn add_two_and_two_returns_four() {
        let result = add(2, 2);

        assert_eq!(result, 4);
    }
}
