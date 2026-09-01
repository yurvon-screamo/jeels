use crate::i18n::use_i18n;
use crate::store::auth_store::AuthStore;
use crate::ui_components::avatar::AvatarSize;
use crate::ui_components::nav_config::NavRoute;
use crate::ui_components::{Avatar, Logo, LogoSize, derive_test_id};
use crate::utils::display_name::display_name_for;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::A;
use leptos_router::hooks::use_location;
use origa::domain::User;

#[component]
pub fn Sidebar(
    current_user: RwSignal<Option<User>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let auth_store = use_context::<AuthStore>().expect("AuthStore not provided");
    let location = use_location();

    let is_visible = Signal::derive(move || {
        let authenticated = auth_store.is_authenticated().get();
        let path = location.pathname.get();
        let hidden_path = path == "/lesson" || path == "/onboarding";
        let has_user = current_user.with(|u| u.is_some());
        authenticated && !hidden_path && has_user
    });

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let avatar_initials = Signal::derive(move || {
        current_user
            .with(|u| {
                u.as_ref()
                    // Empty/untouched relay names must not render a blank
                    // avatar: fall back to the account email's initial.
                    .map(|u| display_name_for(u.username(), u.email()))
                    .and_then(|name| name.chars().next())
                    .map(|c| c.to_uppercase().to_string())
            })
            .unwrap_or_default()
    });

    let profile_active =
        Signal::derive(move || NavRoute::Profile.is_active(&location.pathname.get()));
    let i18n_profile = use_i18n();
    let profile_label = Signal::derive(move || NavRoute::Profile.label(&i18n_profile));
    let profile_class = Signal::derive(move || {
        if profile_active.get() {
            "sidebar-item active".to_string()
        } else {
            "sidebar-item".to_string()
        }
    });

    view! {
        <Show when=move || is_visible.get()>
            <aside class="sidebar" data-testid=test_id_val>
                <nav class="sidebar-nav pt-6">
                    <For
                        each=NavRoute::sidebar_routes
                        key=|route| format!("{:?}", route)
                        children=move |route: &NavRoute| {
                            let i18n = use_i18n();
                            let location = use_location();
                            let label = Signal::derive(move || route.label(&i18n));
                            let is_active =
                                Signal::derive(move || route.is_active(&location.pathname.get()));
                            let test_id = derive_test_id(test_id, route.test_id_suffix());

                            view! {
                                <SidebarNavItem
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
                <div class="flex-1"></div>
                <div class="sidebar-footer">
                    <A
                        href=NavRoute::Profile.href().to_string()
                        attr:class=profile_class
                        attr:data-testid=derive_test_id(test_id, NavRoute::Profile.test_id_suffix())
                        attr:aria-current=move || if profile_active.get() { "page" } else { "false" }
                    >
                        <Avatar
                            size=Signal::derive(move || AvatarSize::Small)
                            initials=avatar_initials
                            test_id=derive_test_id(test_id, "avatar")
                        />
                        <span class="sidebar-item-label">{profile_label}</span>
                    </A>
                </div>
            </aside>
        </Show>
    }
}

#[component]
fn SidebarNavItem(
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
            "sidebar-item active".to_string()
        } else {
            "sidebar-item".to_string()
        }
    });

    view! {
        <A
            href=href
            attr:class=class
            attr:data-testid=test_id_val
            attr:aria-current=move || if is_active.get() { "page" } else { "false" }
        >
            {if use_logo {
                view! { <Logo size=LogoSize::Md /> }.into_any()
            } else {
                view! { <Icon icon=icon width="20" height="20" /> }.into_any()
            }}
            <span class="sidebar-item-label">{label}</span>
        </A>
    }
}
