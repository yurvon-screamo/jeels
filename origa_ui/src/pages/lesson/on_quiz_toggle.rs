use super::lesson_state::LessonState;
use leptos::prelude::*;

pub fn create_on_quiz_toggle(lesson_state: RwSignal<LessonState>) -> Callback<usize> {
    Callback::new(move |option_index: usize| {
        let state = lesson_state.get();
        if state.showing_answer {
            return;
        }

        lesson_state.update(|state| {
            if state.selected_quiz_options.contains(&option_index) {
                state.selected_quiz_options.remove(&option_index);
            } else {
                state.selected_quiz_options.insert(option_index);
            }
        });
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiz_toggle_adds_then_removes_option() {
        let state = RwSignal::new(LessonState::default());
        let toggle = create_on_quiz_toggle(state);

        toggle.run(1);
        assert!(
            state.get().selected_quiz_options.contains(&1),
            "first toggle selects the option"
        );

        toggle.run(1);
        assert!(
            !state.get().selected_quiz_options.contains(&1),
            "second toggle deselects the option"
        );
    }

    #[test]
    fn quiz_toggle_ignored_while_showing_answer() {
        let state = RwSignal::new(LessonState {
            showing_answer: true,
            ..LessonState::default()
        });
        let toggle = create_on_quiz_toggle(state);

        toggle.run(2);
        assert!(
            !state.get().selected_quiz_options.contains(&2),
            "toggles must be ignored after the answer is shown"
        );
    }
}
