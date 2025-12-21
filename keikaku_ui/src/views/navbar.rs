use crate::Route;
use crate::components::avatar::{Avatar, AvatarFallback, AvatarImage};
use crate::components::navbar::{Navbar as DxNavbar, NavbarItem};
use crate::components::toast::ToastProvider;
use dioxus::prelude::*;

const NAVBAR_LOGO: Asset = asset!("/assets/logo_white.png");

#[component]
pub fn Navbar() -> Element {
    rsx! {
        ToastProvider {
            nav {
                class: "bg-surface shadow-soft fixed top-0 left-0 right-0 z-50 backdrop-blur-md bg-white/80",
                style: "height: var(--kk-navbar-height);",
                div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-full",
                    div { class: "flex items-center justify-between h-full",
                        div { class: "flex items-center gap-2",
                            Link {
                                to: Route::Learn {},
                                class: "flex items-center gap-3",
                                Avatar { class: "h-10 w-10",
                                    AvatarImage { src: NAVBAR_LOGO, alt: "Keikaku" }
                                    AvatarFallback { "K" }
                                }
                                span { class: "text-2xl font-bold bg-clip-text text-transparent bg-rainbow-vibrant",
                                    "Keikaku"
                                }
                            }
                        }

                        DxNavbar {
                            aria_label: "Keikaku navigation",
                            class: "flex items-center gap-2",

                            NavbarItem {
                                index: 0usize,
                                value: "learn".to_string(),
                                to: Route::Learn {},
                                class: Some("px-3 py-2 rounded-md".to_string()),
                                active_class: Some("bg-muted".to_string()),
                                "Учиться"
                            }
                            NavbarItem {
                                index: 1usize,
                                value: "cards".to_string(),
                                to: Route::Cards {},
                                class: Some("px-3 py-2 rounded-md".to_string()),
                                active_class: Some("bg-muted".to_string()),
                                "Карточки"
                            }
                            NavbarItem {
                                index: 2usize,
                                value: "import".to_string(),
                                to: Route::Import {},
                                class: Some("px-3 py-2 rounded-md".to_string()),
                                active_class: Some("bg-muted".to_string()),
                                "Импорт"
                            }
                            NavbarItem {
                                index: 3usize,
                                value: "translate".to_string(),
                                to: Route::Translate {},
                                class: Some("px-3 py-2 rounded-md".to_string()),
                                active_class: Some("bg-muted".to_string()),
                                "Перевод"
                            }
                            NavbarItem {
                                index: 4usize,
                                value: "kanji".to_string(),
                                to: Route::Kanji {},
                                class: Some("px-3 py-2 rounded-md".to_string()),
                                active_class: Some("bg-muted".to_string()),
                                "Кандзи"
                            }
                            NavbarItem {
                                index: 5usize,
                                value: "profile".to_string(),
                                to: Route::Profile {},
                                class: Some("px-3 py-2 rounded-md".to_string()),
                                active_class: Some("bg-muted".to_string()),
                                "Профиль"
                            }
                        }
                    }
                }
            }

            main { style: "padding-top: var(--kk-navbar-height);", Outlet::<Route> {} }
        }
    }
}
