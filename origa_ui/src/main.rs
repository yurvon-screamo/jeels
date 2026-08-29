// Deeply nested Leptos view types push rustc's layout-query depth past the
// default limit of 128 when this bin monomorphises the whole `<App/>` tree
// ("queries overflow the depth limit!" during full codegen only — check-mode
// passes). Mirrors the lib crate's attribute; safe since dev-profile
// debuginfo was capped (see root Cargo.toml) so oversized artifacts no
// longer kill link.exe.
#![recursion_limit = "512"]

use leptos::prelude::*;
use leptos_meta::MetaTags;
use leptos_router::components::Router;
use origa_ui::app::*;
use origa_ui::i18n::I18nContextProvider;

fn main() {
    origa_ui::init_tracing();

    mount_to_body(|| {
        view! {
            <MetaTags />
            <I18nContextProvider cookie_options=origa_ui::i18n::persistent_locale_cookie_options()>
                <Router>
                    <App/>
                </Router>
            </I18nContextProvider>
        }
    })
}
