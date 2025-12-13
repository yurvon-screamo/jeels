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
}
