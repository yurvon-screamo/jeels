use dioxus::prelude::*;

#[component]
pub fn LoadingState(message: Option<String>) -> Element {
    let default_message = message.unwrap_or_else(|| "Загрузка...".to_string());

    rsx! {
        div { class: "flex flex-col items-center justify-center py-12 space-y-4",
            div { class: "w-8 h-8 border-4 border-pink-200 border-t-pink-600 rounded-full animate-spin" }
            p { class: "text-slate-600 text-center", {default_message} }
        }
    }
}
