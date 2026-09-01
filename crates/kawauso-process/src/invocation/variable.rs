//! One environment variable of an external command

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

use super::variable_name::VariableName;
use super::variable_value::VariableValue;

/// One environment variable of an external command
///
/// A variable carries a name and a value. The command reads the variable in
/// its environment, next to the variables that it inherits from the process
/// that runs it. Some programs take a decision from a variable and from no
/// flag, such as a test runner that selects the format of its report with
/// one.
///
/// A pair of a name and a value becomes a variable, so a caller that holds
/// its variables as pairs gives them to an invocation as they are.
///
/// # Examples
///
/// ```
/// use kawauso_process::invocation::Variable;
///
/// let variable = Variable::from(("RUSTUP_TOOLCHAIN", "stable"));
///
/// assert_eq!(variable.name().get(), "RUSTUP_TOOLCHAIN");
/// ```
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Variable {
    /// The name under which the command reads the variable
    name: VariableName,

    /// The value that the command reads
    value: VariableValue,
}

impl Variable {
    /// Creates a variable from a name and a value
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_process::invocation::Variable;
    ///
    /// let variable = Variable::new("CARGO_TERM_COLOR", "always");
    ///
    /// assert_eq!(variable.value().get(), "always");
    /// ```
    pub fn new(name: impl Into<VariableName>, value: impl Into<VariableValue>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }

    /// Returns the name of the variable
    pub fn name(&self) -> &VariableName {
        &self.name
    }

    /// Returns the value of the variable
    pub fn value(&self) -> &VariableValue {
        &self.value
    }
}

/// Shows the variable for a reader
///
/// The text is the name, an equals sign, and the value, which is the form in
/// which a shell sets a variable. A value that holds a space shows without
/// marks, because the caller that renders a whole command line adds the marks
/// where the line needs them.
impl Display for Variable {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "{}={}", self.name, self.value)
    }
}

/// Creates a variable from a pair of a name and a value
impl<N: Into<VariableName>, V: Into<VariableValue>> From<(N, V)> for Variable {
    fn from((name, value): (N, V)) -> Self {
        Self::new(name, value)
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every
    // test would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    // A caller that holds its variables as pairs gives them to an invocation
    // as they are, and the pair keeps its two parts in their places.
    #[test]
    fn from_pair_keeps_the_name_and_the_value() {
        let variable = Variable::from(("RUSTUP_TOOLCHAIN", "stable"));

        assert_eq!(
            (
                variable.name().get().to_str(),
                variable.value().get().to_str()
            ),
            (Some("RUSTUP_TOOLCHAIN"), Some("stable"))
        );
    }

    // The text of a variable reaches a log line through `Display`, and a
    // caller that shows one variable gets it in the form of a shell.
    #[test]
    fn to_string_returns_the_name_and_the_value() {
        let variable = Variable::new("RUSTUP_TOOLCHAIN", "stable");

        assert_eq!(variable.to_string(), "RUSTUP_TOOLCHAIN=stable");
    }
}
