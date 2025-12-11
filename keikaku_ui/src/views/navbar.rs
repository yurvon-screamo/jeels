use crate::Route;
use dioxus::prelude::*;

/// The Navbar component that will be rendered on all pages of our app since every page is under the layout.
///
///
/// This layout component wraps the UI of [Route::Home] and [Route::Blog] in a common navbar. The contents of the Home and Blog
/// routes will be rendered under the outlet inside this component
#[component]
pub fn Navbar() -> Element {
    rsx! {
        nav {
            class: "bg-surface shadow-soft sticky top-0 z-50 backdrop-blur-md bg-white/80",
            div {
                class: "max-w-7xl mx-auto px-8 py-4",
                div {
                    class: "flex items-center justify-between",
                    div {
                        class: "flex items-center gap-2",
                        Link {
                            to: Route::Home {},
                            class: "text-2xl font-bold bg-clip-text text-transparent bg-rainbow-vibrant",
                            "Keikaku"
                        }
                    }
                    div {
                        class: "flex items-center gap-4",
                        Link {
                            to: Route::Home {},
                            class: "text-text-main hover:text-accent-pink transition-colors font-medium",
                            "Home"
                        }
                        Link {
                            to: Route::Blog { id: 1 },
                            class: "text-text-main hover:text-accent-purple transition-colors font-medium",
                            "Blog"
                        }
                        Link {
                            to: Route::Showcase {},
                            class: "text-text-main hover:text-accent-cyan transition-colors font-medium",
                            "Showcase"
                        }
                    }
                }
            }
        }

        // The `Outlet` component is used to render the next component inside the layout. In this case, it will render either
        // the [`Home`] or [`Blog`] component depending on the current route.
        Outlet::<Route> {}
    }
}
