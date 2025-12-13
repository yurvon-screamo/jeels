use dioxus::prelude::*;

use views::{
    Anki, Cards, Duolingo, Jlpt, Kanji, Learn, Migii, Navbar, Overview, Rebuild, Translate,
};

mod components;
mod domain;
mod hooks;
mod keikaku_api;
mod ui;
mod utils;
mod views;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const UI_STYLES: Asset = asset!("/assets/ui-styles.css");

fn main() {
    dioxus::launch(App);
}

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
        #[route("/")]
        Overview {},
        #[route("/cards")]
        Cards {},
        #[route("/learn")]
        Learn {},
        #[route("/translate")]
        Translate {},
        #[route("/kanji")]
        Kanji {},
        #[route("/jlpt")]
        Jlpt {},
        #[route("/duolingo")]
        Duolingo {},
        #[route("/migii")]
        Migii {},
        #[route("/anki")]
        Anki {},
        #[route("/rebuild")]
        Rebuild {},
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "stylesheet", href: UI_STYLES }
        style { {global_styles()} }

        Router::<Route> {}
    }
}

fn global_styles() -> &'static str {
    r#"
                html, body {
                    margin: 0;
                    padding: 0;
                    border: none;
                    outline: none;
                    box-sizing: border-box;
                }
                
                html {
                    background-color: #F8F9FD;
                    font-family: 'Maple Mono', monospace;
                }
                
                body {
                    background-color: #F8F9FD;
                    color: #334155;
                    font-family: 'Maple Mono', monospace;
                    margin: 0;
                    padding: 0;
                    min-height: 100vh;
                    overflow-x: hidden;
                }
                
                * {
                    box-sizing: border-box;
                }
    "#
}
