/// Round a number to the nearest multiple of another number.
///
/// # Examples
/// ```rust
/// use flipper_utils::round_to;
///
/// assert_eq!(round_to(0, 550), 0);
/// assert_eq!(round_to(1, 550), 0);
/// assert_eq!(round_to(549, 550), 550);
/// assert_eq!(round_to(550, 550), 550);
/// assert_eq!(round_to(551, 550), 550);
/// assert_eq!(round_to(120, 50), 100);
/// assert_eq!(round_to(125, 50), 150);
///
/// assert_eq!(round_to(2972, 550), 2750);
/// ```
pub fn round_to(x: u32, round_to: u32) -> u32 {
    (x + round_to / 2) / round_to * round_to
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_to() {
        // duplicating the examples from the docstring
        assert_eq!(round_to(0, 550), 0);
        assert_eq!(round_to(1, 550), 0);
        assert_eq!(round_to(549, 550), 550);
        assert_eq!(round_to(550, 550), 550);
        assert_eq!(round_to(551, 550), 550);
        assert_eq!(round_to(120, 50), 100);
        assert_eq!(round_to(125, 50), 150);

        assert_eq!(round_to(2972, 550), 2750);
    }
}
