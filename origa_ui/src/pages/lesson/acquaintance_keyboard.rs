//! Клавиатура режима знакомства: те же хендлы, что в обычном уроке
//! (спека §8.3): Space = показать/дальше, [1]/[2] = оценка.

use super::acquaintance_state::{AcquaintanceContext, AcquaintanceStage};
use leptos::ev::KeyboardEvent;
use leptos::prelude::*;

/// Действие, разрешённое клавиатурой на текущей стадии и состоянии слайда.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcquaintanceKeyAction {
    /// Space в показе — следующий слайд («Дальше»).
    Advance,
    /// Space в тренировке до раскрытия — «Показать ответ».
    Reveal,
    /// [1] после раскрытия — «Не помню».
    RateDontRemember,
    /// [2] после раскрытия — «Помню».
    RateRemember,
}

/// Чистая функция разрешения клавиши — покрывается host-тестами без
/// браузерного окружения.
pub fn resolve_key_action(
    stage: AcquaintanceStage,
    showing_answer: bool,
    key: &str,
) -> Option<AcquaintanceKeyAction> {
    match stage {
        AcquaintanceStage::Presentation => (key == " ").then_some(AcquaintanceKeyAction::Advance),
        AcquaintanceStage::Training => match (showing_answer, key) {
            (false, " ") => Some(AcquaintanceKeyAction::Reveal),
            (true, "1") => Some(AcquaintanceKeyAction::RateDontRemember),
            (true, "2") => Some(AcquaintanceKeyAction::RateRemember),
            _ => None,
        },
        AcquaintanceStage::Completed | AcquaintanceStage::Inactive => None,
    }
}

/// Колбэки, которые клавиатура дёргает вместо кнопок.
pub struct AcquaintanceKeyboardActions {
    pub on_advance: Box<dyn Fn()>,
    pub on_reveal: Box<dyn Fn()>,
    pub on_rate: Box<dyn Fn(bool)>,
}

/// Обработчик keydown: резолвит действие и исполняет колбэк.
/// Guard на поля ввода — на стороне слушателя (`is_typing_target`).
pub fn create_acquaintance_keyboard_handler(
    ctx: AcquaintanceContext,
    showing_answer: RwSignal<bool>,
    actions: AcquaintanceKeyboardActions,
) -> impl Fn(KeyboardEvent) {
    move |ev: KeyboardEvent| {
        // Автоповтор удержания игнорируем: иначе удержание Space на
        // последнем слайде показа «продавливает» Reveal уже в тренировке.
        if ev.repeat() {
            return;
        }
        let stage = ctx.state.get().stage;
        let Some(action) = resolve_key_action(stage, showing_answer.get_untracked(), &ev.key())
        else {
            return;
        };
        ev.prevent_default();
        match action {
            AcquaintanceKeyAction::Advance => (actions.on_advance)(),
            AcquaintanceKeyAction::Reveal => (actions.on_reveal)(),
            AcquaintanceKeyAction::RateDontRemember => (actions.on_rate)(false),
            AcquaintanceKeyAction::RateRemember => (actions.on_rate)(true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presentation_space_advances() {
        assert_eq!(
            resolve_key_action(AcquaintanceStage::Presentation, false, " "),
            Some(AcquaintanceKeyAction::Advance)
        );
    }

    #[test]
    fn presentation_digits_do_nothing() {
        assert_eq!(
            resolve_key_action(AcquaintanceStage::Presentation, false, "1"),
            None
        );
    }

    #[test]
    fn training_space_before_reveal_reveals() {
        assert_eq!(
            resolve_key_action(AcquaintanceStage::Training, false, " "),
            Some(AcquaintanceKeyAction::Reveal)
        );
    }

    #[test]
    fn training_after_reveal_one_is_dont_remember() {
        assert_eq!(
            resolve_key_action(AcquaintanceStage::Training, true, "1"),
            Some(AcquaintanceKeyAction::RateDontRemember)
        );
    }

    #[test]
    fn training_after_reveal_two_is_remember() {
        assert_eq!(
            resolve_key_action(AcquaintanceStage::Training, true, "2"),
            Some(AcquaintanceKeyAction::RateRemember)
        );
    }

    #[test]
    fn training_space_after_reveal_does_nothing() {
        // После раскрытия оценивание только [1]/[2]: Space не должен
        // случайно скрыть ответ или двинуть ротацию.
        assert_eq!(
            resolve_key_action(AcquaintanceStage::Training, true, " "),
            None
        );
    }

    #[test]
    fn rating_keys_do_nothing_before_reveal() {
        assert_eq!(
            resolve_key_action(AcquaintanceStage::Training, false, "1"),
            None
        );
        assert_eq!(
            resolve_key_action(AcquaintanceStage::Training, false, "2"),
            None
        );
    }

    #[test]
    fn inactive_ignores_all_keys() {
        for key in [" ", "1", "2"] {
            assert_eq!(
                resolve_key_action(AcquaintanceStage::Inactive, false, key),
                None
            );
        }
    }
}
