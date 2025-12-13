use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub enum Notification {
    None,
    Success(String),
    Error(String),
}

#[component]
pub fn NotificationArea(notification: Signal<Notification>, on_close: EventHandler<()>) -> Element {
    match notification() {
        Notification::Success(msg) => rsx! {
            crate::ui::NotificationBanner {
                message: msg,
                notification_type: crate::ui::NotificationType::Success,
                on_close,
            }
        },
        Notification::Error(msg) => rsx! {
            crate::ui::NotificationBanner {
                message: msg,
                notification_type: crate::ui::NotificationType::Error,
                on_close,
            }
        },
        Notification::None => rsx! {},
    }
}
