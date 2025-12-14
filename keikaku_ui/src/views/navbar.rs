use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Navbar() -> Element {
    let route = use_route::<Route>();

    rsx! {
        nav { class: "bg-surface shadow-soft sticky top-0 z-50 backdrop-blur-md bg-white/80",
            div { class: "max-w-7xl mx-auto px-8 py-4",
                div { class: "flex items-center justify-between",
                    div { class: "flex items-center gap-2",
                        Link {
                            to: Route::Learn {},
                            class: "text-2xl font-bold bg-clip-text text-transparent bg-rainbow-vibrant",
                            "Keikaku"
                        }
                    }
                    div { class: "flex items-center gap-4",
                        Link {
                            to: Route::Learn {},
                            class: if matches!(route, Route::Learn {}) { "text-text-main hover:text-accent-cyan transition-colors font-medium pb-1 border-b-2 border-accent-cyan" } else { "text-text-main hover:text-accent-cyan transition-colors font-medium" },
                            "Обучение"
                        }
                        Link {
                            to: Route::Overview {},
                            class: if matches!(route, Route::Overview {}) { "text-text-main hover:text-accent-pink transition-colors font-medium pb-1 border-b-2 border-accent-pink" } else { "text-text-main hover:text-accent-pink transition-colors font-medium" },
                            "Статистика"
                        }
                        Link {
                            to: Route::Cards {},
                            class: "text-text-main hover:text-accent-purple transition-colors font-medium",
                            "Карточки"
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
                        Link {
                            to: Route::Jlpt {},
                            class: "text-text-main hover:text-accent-cyan transition-colors font-medium",
                            "JLPT"
                        }
                        Link {
                            to: Route::Duolingo {},
                            class: "text-text-main hover:text-accent-cyan transition-colors font-medium",
                            "Duolingo"
                        }
                        Link {
                            to: Route::Migii {},
                            class: "text-text-main hover:text-accent-cyan transition-colors font-medium",
                            "Migii"
                        }
                        Link {
                            to: Route::Anki {},
                            class: "text-text-main hover:text-accent-cyan transition-colors font-medium",
                            "Anki"
                        }
                        Link {
                            to: Route::Rebuild {},
                            class: "text-text-main hover:text-accent-cyan transition-colors font-medium",
                            "Перестроить"
                        }
                        Link {
                            to: Route::Profile {},
                            class: "text-text-main hover:text-accent-cyan transition-colors font-medium",
                            "Профиль"
                        }
                    }
                }
            }
        }

        Outlet::<Route> {}
    }
}
