#[derive(Clone, PartialEq)]
pub enum SessionState {
    Settings,
    Loading,
    Active,
    Completed,
}

#[derive(Clone, PartialEq)]
pub enum LearnStep {
    Question,
    Answer,
}

#[derive(Clone, PartialEq)]
pub struct LearnCard {
    pub id: String,
    pub question: String,
    pub answer: String,
}
