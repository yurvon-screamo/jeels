use dioxus::prelude::*;
use ulid::Ulid;

use keikaku::{
    application::UserRepository,
    domain::{
        User,
        value_objects::{JapaneseLevel, NativeLanguage},
    },
    settings::ApplicationEnvironment,
};

use views::{
    Anki, Cards, Duolingo, Jlpt, Kanji, Learn, Migii, Navbar, Overview, Profile, Rebuild, Translate,
};

pub const DEFAULT_USERNAME: &str = "yurvon_screamo";

pub async fn ensure_user(
    env: &'static ApplicationEnvironment,
    username: &str,
) -> Result<Ulid, String> {
    let repo = env.get_repository().await.map_err(to_error)?;
    if let Some(user) = repo.find_by_username(username).await.map_err(to_error)? {
        return Ok(user.id());
    }
    let new_user = User::new(
        username.to_string(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
        7,
    );
    let id = new_user.id();
    repo.save(&new_user).await.map_err(to_error)?;
    Ok(id)
}

pub fn to_error(err: impl std::fmt::Display) -> String {
    err.to_string()
}

mod domain;
mod ui;
mod views;

const APP_ICON: Asset = asset!("/assets/icons/32x32.png");
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
        Learn {},
        #[route("/overview")]
        Overview {},
        #[route("/cards")]
        Cards {},
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
        #[route("/profile")]
        Profile {},
        #[route("/anki")]
        Anki {},
        #[route("/rebuild")]
        Rebuild {},
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: APP_ICON }
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
