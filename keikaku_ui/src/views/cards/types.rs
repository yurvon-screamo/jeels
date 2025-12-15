#[derive(Clone, PartialEq)]
pub enum FilterStatus {
    All,
    Due,
    NotDue,
}

#[derive(Clone, PartialEq)]
pub enum SortBy {
    Date,
    Question,
    Answer,
}

#[derive(Clone, PartialEq)]
pub struct UiCard {
    pub id: String,
    pub question: String,
    pub answer: String,
    pub next_review: String,
    pub due: bool,
    pub is_new: bool,
    pub is_in_progress: bool,
    pub is_learned: bool,
    pub is_low_stability: bool,
}
