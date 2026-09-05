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

    // Сигналы и колбэки живут в реактивном рантайме: без изолированного
    // Owner они попадают в глобальный, общий для параллельных потоков
    // тестов — под нагрузкой полного воркспейса это даёт load-флейк.
    // Паттерн как у соседних on_*-тестов (on_dont_know, on_quiz_select).
    #[test]
    fn quiz_toggle_adds_then_removes_option() {
        let (after_first, after_second) = Owner::new().with(|| {
            let state = RwSignal::new(LessonState::default());
            let toggle = create_on_quiz_toggle(state);

            toggle.run(1);
            let after_first = state.get().selected_quiz_options.contains(&1);

            toggle.run(1);
            let after_second = state.get().selected_quiz_options.contains(&1);

            (after_first, after_second)
        });

        assert!(after_first, "first toggle selects the option");
        assert!(!after_second, "second toggle deselects the option");
    }

    #[test]
    fn quiz_toggle_ignored_while_showing_answer() {
        let contains = Owner::new().with(|| {
            let state = RwSignal::new(LessonState {
                showing_answer: true,
                ..LessonState::default()
            });
            let toggle = create_on_quiz_toggle(state);

            toggle.run(2);
            state.get().selected_quiz_options.contains(&2)
        });

        assert!(
            !contains,
            "toggles must be ignored after the answer is shown"
        );
    }
}
