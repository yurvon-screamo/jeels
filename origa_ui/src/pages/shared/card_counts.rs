#[derive(Clone, Copy, PartialEq, Default)]
pub struct CardCounts {
    pub total: usize,
    pub new: usize,
    pub hard: usize,
    pub in_progress: usize,
    pub learned: usize,
    pub favorite: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_card_counts_are_zero() {
        let counts = CardCounts::default();
        assert_eq!(counts.total, 0);
        assert_eq!(counts.new, 0);
        assert_eq!(counts.hard, 0);
        assert_eq!(counts.in_progress, 0);
        assert_eq!(counts.learned, 0);
        assert_eq!(counts.favorite, 0);
    }
}
