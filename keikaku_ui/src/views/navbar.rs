use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Navbar() -> Element {
    rsx! {
        nav { class: "bg-surface shadow-soft sticky top-0 z-50 backdrop-blur-md bg-white/80",
            div { class: "max-w-7xl mx-auto px-8 py-4",
                div { class: "flex items-center justify-between",
                    div { class: "flex items-center gap-2",
                        Link {
                            to: Route::Overview {},
                            class: "text-2xl font-bold bg-clip-text text-transparent bg-rainbow-vibrant",
                            "Keikaku"
                        }
                    }
                    div { class: "flex items-center gap-4",
                        Link {
                            to: Route::Overview {},
                            class: "text-text-main hover:text-accent-pink transition-colors font-medium",
                            "Обзор"
                        }
                        Link {
                            to: Route::Cards {},
                            class: "text-text-main hover:text-accent-purple transition-colors font-medium",
                            "Карточки"
                        }
                        Link {
                            to: Route::Learn {},
                            class: "text-text-main hover:text-accent-cyan transition-colors font-medium",
                            "Обучение"
                        }
                        Link {
                            to: Route::Translate {},
                            class: "text-text-main hover:text-accent-cyan transition-colors font-medium",
                            "Перевод"
                        }
                        Link {
                            to: Route::Kanji {},
                            class: "text-text-main hover:text-accent-cyan transition-colors font-medium",
                            "Кандзи"
                        }
                    }
                }
            }
        }

        Outlet::<Route> {}
    }
}
