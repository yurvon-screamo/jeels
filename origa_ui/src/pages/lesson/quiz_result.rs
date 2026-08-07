use origa::domain::MultiQuizResult;

use super::yesno_card_view::YesNoResult;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum QuizResult {
    #[default]
    None,
    Correct,
    Incorrect,
    DontKnow,
    MultiCorrect,
    MultiPartial,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum OptionDisplay {
    Neutral,
    Correct,
    Wrong,
    Missed,
    Dimmed,
    /// Completely hidden — used to remove incorrect/unselected options
    /// after the user answers, collapsing the answer grid to only the
    /// selected answer and the correct one.
    Hidden,
}

impl QuizResult {
    pub fn option_display(&self, is_correct: bool, is_selected: bool) -> OptionDisplay {
        match self {
            QuizResult::None => OptionDisplay::Neutral,
            QuizResult::Correct | QuizResult::Incorrect if is_correct => OptionDisplay::Correct,
            QuizResult::Correct => OptionDisplay::Hidden,
            QuizResult::Incorrect if is_selected => OptionDisplay::Wrong,
            QuizResult::Incorrect => OptionDisplay::Hidden,
            QuizResult::DontKnow if is_correct => OptionDisplay::Correct,
            QuizResult::DontKnow => OptionDisplay::Hidden,
            QuizResult::MultiCorrect | QuizResult::MultiPartial => OptionDisplay::Neutral,
        }
    }

    pub fn multi_option_display(
        is_correct: bool,
        is_selected: bool,
        multi_result: &MultiQuizResult,
        index: usize,
    ) -> OptionDisplay {
        if !is_correct && !is_selected {
            return OptionDisplay::Dimmed;
        }
        if is_correct && is_selected {
            return OptionDisplay::Correct;
        }
        if is_correct && !is_selected {
            let is_missed = multi_result.missed.contains(&index);
            return if is_missed {
                OptionDisplay::Missed
            } else {
                OptionDisplay::Dimmed
            };
        }
        OptionDisplay::Wrong
    }

    pub fn option_class(&self, is_correct: bool, is_selected: bool) -> &'static str {
        match self.option_display(is_correct, is_selected) {
            OptionDisplay::Neutral => "quiz-option-neutral",
            OptionDisplay::Correct => "quiz-option-correct",
            OptionDisplay::Wrong => "quiz-option-wrong",
            OptionDisplay::Dimmed => "quiz-option-dimmed",
            OptionDisplay::Missed => "quiz-option-missed",
            OptionDisplay::Hidden => "quiz-option-hidden",
        }
    }

    pub fn from_multi_result(result: &MultiQuizResult) -> Self {
        if result.is_perfect {
            QuizResult::MultiCorrect
        } else {
            QuizResult::MultiPartial
        }
    }

    pub fn from_multi_result_lenient(result: &MultiQuizResult) -> Self {
        if result.is_lenient_pass() {
            QuizResult::MultiCorrect
        } else {
            QuizResult::MultiPartial
        }
    }
}

impl From<YesNoResult> for QuizResult {
    fn from(value: YesNoResult) -> Self {
        match value {
            YesNoResult::None => QuizResult::None,
            YesNoResult::Correct => QuizResult::Correct,
            YesNoResult::Incorrect => QuizResult::Incorrect,
            YesNoResult::DontKnow => QuizResult::DontKnow,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_quiz_hides_incorrect_unselected_options_after_answer() {
        // User answered incorrectly. Options that are neither correct nor
        // selected by the user should be Hidden — not Dimmed — so the grid
        // collapses to just the user's answer + the correct one.
        let result = QuizResult::Incorrect;
        assert_eq!(
            result.option_display(false, false),
            OptionDisplay::Hidden,
            "incorrect unselected options should be Hidden after answering"
        );
    }

    #[test]
    fn single_quiz_keeps_selected_wrong_visible() {
        let result = QuizResult::Incorrect;
        assert_eq!(
            result.option_display(false, true),
            OptionDisplay::Wrong,
            "the user's selected wrong answer must stay visible"
        );
    }

    #[test]
    fn single_quiz_correct_answer_keeps_selected_visible() {
        let result = QuizResult::Correct;
        assert_eq!(result.option_display(true, true), OptionDisplay::Correct);
    }

    #[test]
    fn single_quiz_correct_answer_hides_unselected() {
        let result = QuizResult::Correct;
        assert_eq!(
            result.option_display(false, false),
            OptionDisplay::Hidden,
            "when answered correctly, unselected options should collapse"
        );
    }

    #[test]
    fn dont_know_hides_all_wrong_keeps_correct() {
        let result = QuizResult::DontKnow;
        assert_eq!(result.option_display(true, false), OptionDisplay::Correct);
        assert_eq!(result.option_display(false, false), OptionDisplay::Hidden);
    }
}
