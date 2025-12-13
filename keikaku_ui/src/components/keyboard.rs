use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub enum KeyAction {
    ShowAnswer,
    RateEasy,
    RateGood,
    RateHard,
    RateAgain,
    Skip,
}

pub fn handle_key_event<F>(key: Key, callback: F)
where
    F: Fn(KeyAction),
{
    let action = match key {
        Key::Character(c) if c == " " => Some(KeyAction::ShowAnswer),
        Key::Character(c) if c == "1" => Some(KeyAction::RateEasy),
        Key::Character(c) if c == "2" => Some(KeyAction::RateGood),
        Key::Character(c) if c == "3" => Some(KeyAction::RateHard),
        Key::Character(c) if c == "4" => Some(KeyAction::RateAgain),
        Key::Escape => Some(KeyAction::Skip),
        _ => None,
    };

    if let Some(action) = action {
        callback(action);
    }
}
