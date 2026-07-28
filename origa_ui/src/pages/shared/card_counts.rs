#[derive(Clone, Copy, PartialEq, Default)]
pub struct CardCounts {
    pub total: usize,
    pub new: usize,
    pub hard: usize,
    pub in_progress: usize,
    pub learned: usize,
    pub favorite: usize,
}
