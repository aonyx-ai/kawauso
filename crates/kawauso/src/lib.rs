// kawauso[impl placeholder.add]
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    // kawauso[verify placeholder.add]
    #[test]
    fn add_two_and_two_returns_four() {
        let result = add(2, 2);

        assert_eq!(result, 4);
    }
}
