use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub enum NotificationType {
    Success,
    Error,
}

#[component]
pub fn NotificationBanner(
    message: String,
    notification_type: NotificationType,
    on_close: EventHandler<()>,
) -> Element {
    let bg_class = match notification_type {
        NotificationType::Success => "bg-emerald-50 border border-emerald-200 text-emerald-800",
        NotificationType::Error => "bg-red-50 border border-red-200 text-red-800",
    };

    rsx! {
        div { class: "fixed top-4 right-4 z-50 px-6 py-4 rounded-xl shadow-lg {bg_class} max-w-md animate-slide-in",
            div { class: "flex items-center gap-2",
                span { class: "font-semibold", {message} }
                button {
                    class: "ml-auto text-current opacity-60 hover:opacity-100",
                    onclick: move |_| on_close.call(()),
                    "×"
                }
            }
        }
    }
}
