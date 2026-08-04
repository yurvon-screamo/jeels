use crate::i18n::use_i18n;
use crate::store::auth_store::AuthStore;
use crate::ui_components::nav_config::NavRoute;
use crate::ui_components::{Logo, LogoSize, derive_test_id};
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

#[component]
pub fn BottomTabBar(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let auth_store = use_context::<AuthStore>().expect("AuthStore not provided");
    let location = use_location();

    let is_visible = Signal::derive(move || {
        let authenticated = auth_store.is_authenticated().get();
        let path = location.pathname.get();
        let hidden_path = path == "/lesson" || path == "/onboarding";
        authenticated && !hidden_path
    });

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <Show when=move || is_visible.get()>
            <nav class="bottom-tab-bar lg:hidden" data-testid=test_id_val>
                <For
                    each=NavRoute::all
                    key=|route| format!("{:?}", route)
                    children=move |route: &NavRoute| {
                        let i18n = use_i18n();
                        let location = use_location();
                        let label = Signal::derive(move || route.label(&i18n));
                        let is_active =
                            Signal::derive(move || route.is_active(&location.pathname.get()));
                        let test_id = derive_test_id(test_id, route.test_id_suffix());

                        view! {
                            <BottomTabItem
                                href=route.href().to_string()
                                icon=route.icon()
                                use_logo=route.use_logo()
                                label=label
                                is_active=is_active
                                test_id=test_id
                            />
                        }
                    }
                />
            </nav>
        </Show>
    }
}

#[component]
fn BottomTabItem(
    #[prop(into)] href: String,
    icon: icondata::Icon,
    #[prop(optional)] use_logo: bool,
    #[prop(into)] label: Signal<String>,
    #[prop(into)] is_active: Signal<bool>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let class = Signal::derive(move || {
        if is_active.get() {
            "bottom-tab-item active".to_string()
        } else {
            "bottom-tab-item".to_string()
        }
    });

    view! {
        <A href=href attr:class=class attr:data-testid=test_id_val attr:aria-current=move || if is_active.get() { "page" } else { "false" }>
            {if use_logo {
                view! {
                    <span class="bottom-tab-item-logo">
                        <Logo size=LogoSize::Sm />
                    </span>
                }.into_any()
            } else {
                view! { <Icon icon=icon width="26" height="26" /> }.into_any()
            }}
            <span class="bottom-tab-item-label">{label}</span>
        </A>
    }
}
