//! WASM render tests for i18n and context-dependent components:
//! `CardActionBar`, `CollapsibleDescription`, `DeleteConfirmModal`,
//! `Dropdown`, `NativeLanguageToggle`, `legal_links`, `SelectedCount`,
//! `UpdateDrawer`, `Toast`/`ToastContainer`, `TranslatorText`,
//! `ConnectivityBanner`, `OfflineBundleCard`, `Sidebar`, `BottomTabBar`.

#![cfg(all(target_arch = "wasm32", test))]

use leptos::prelude::*;
use leptos::task::tick;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

use crate::test_support::{
    create_wrapper, mount_to_wrapper, mount_with_i18n, mount_with_router_and_stores,
    mount_with_stores, shared_cell, test_user,
};
use crate::ui_components::{
    BottomTabBar, CardActionBar, CollapsibleDescription, ConnectivityBanner, DeleteConfirmModal,
    Dropdown, DropdownItem, NativeLanguageToggle, SelectedCount, Sidebar, Toast, ToastContainer,
    ToastData, ToastType, TranslatorText, UpdateDrawer,
};

wasm_bindgen_test_configure!(run_in_browser);

// ═══════════════════════════════════════════════════════════════════════
// CardActionBar
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn card_action_bar_all_actions_rendered() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! {
            <CardActionBar
                tag_variant=Signal::derive(|| crate::ui_components::TagVariant::Olive)
                tag_label=Signal::derive(|| "Kanji".to_string())
                is_favorite=Signal::from(false)
                on_toggle_favorite=Callback::new(|()| {})
                on_mark_as_known=Callback::new(|()| {})
                show_mark_as_known=Signal::from(true)
                on_delete=Callback::new(|()| {})
                test_id="cab1"
                show_tag=Signal::from(true)
            />
        }
        .into_any()
    });
    tick().await;

    for suffix in ["favorite-btn", "mark-known-btn", "delete-btn"] {
        let el = wrapper.query_selector(&format!("[data-testid=\"cab1-{suffix}\"]"));
        assert!(
            el.is_ok_and(|e| e.is_some()),
            "{suffix} must render when its callback is provided"
        );
    }
    // Tag shown
    let tag = wrapper.query_selector(".card-action-status .tag");
    assert!(tag.is_ok_and(|t| t.is_some()), "tag must render");
}

#[wasm_bindgen_test]
async fn card_action_bar_without_callbacks_renders_no_buttons() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! {
            <CardActionBar
                tag_variant=Signal::derive(|| crate::ui_components::TagVariant::Default)
                tag_label=Signal::derive(|| String::new())
                is_favorite=Signal::from(false)
                test_id="cab2"
                show_tag=Signal::from(false)
            />
        }
        .into_any()
    });
    tick().await;

    for suffix in ["favorite-btn", "mark-known-btn", "delete-btn"] {
        let el = wrapper.query_selector(&format!("[data-testid=\"cab2-{suffix}\"]"));
        assert!(
            el.is_ok_and(|e| e.is_none()),
            "{suffix} must NOT render without a callback"
        );
    }
}

#[wasm_bindgen_test]
async fn card_action_bar_mark_known_hidden_when_learned() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! {
            <CardActionBar
                tag_variant=Signal::derive(|| crate::ui_components::TagVariant::Default)
                tag_label=Signal::derive(|| String::new())
                is_favorite=Signal::from(false)
                on_mark_as_known=Callback::new(|()| {})
                show_mark_as_known=Signal::from(false)
                test_id="cab3"
                show_tag=Signal::from(false)
            />
        }
        .into_any()
    });
    tick().await;

    // When the card is learned the wrapper span stays in the DOM but hides
    // the button via `style:display: none` (the button sits inside a Tooltip
    // container, so the span is its grandparent).
    let button = wrapper
        .query_selector("[data-testid=\"cab3-mark-known-btn\"]")
        .ok()
        .flatten()
        .expect("button node stays in the DOM (visual hiding is CSS-driven)");
    let span = button
        .parent_element()
        .and_then(|p| p.parent_element())
        .expect("tooltip container must be wrapped by the hiding span");
    let style = span.get_attribute("style").unwrap_or_default();
    assert!(
        style.contains("none"),
        "learned card must hide the mark-known span via display:none; got: {style}"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// CollapsibleDescription
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn collapsible_default_collapsed_applies_clamp() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! {
            <CollapsibleDescription test_id="col1">
                <p>"Some long description text"</p>
            </CollapsibleDescription>
        }
        .into_any()
    });
    tick().await;

    let clamped = wrapper.query_selector("[data-testid=\"col1\"] .line-clamp-3");
    assert!(
        clamped.is_ok_and(|c| c.is_some()),
        "default_collapsed=true must apply line-clamp-3"
    );
}

#[wasm_bindgen_test]
async fn collapsible_expanded_default_no_clamp() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! {
            <CollapsibleDescription default_collapsed=false test_id="col2">
                <p>"Text"</p>
            </CollapsibleDescription>
        }
        .into_any()
    });
    tick().await;

    let clamped = wrapper.query_selector("[data-testid=\"col2\"] .line-clamp-3");
    assert!(
        clamped.is_ok_and(|c| c.is_none()),
        "default_collapsed=false must not clamp"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// DeleteConfirmModal
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn delete_confirm_modal_open_shows_cancel_and_confirm() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let is_open = RwSignal::new(true);
        view! {
            <DeleteConfirmModal
                test_id="dcm1"
                is_open=is_open
                is_deleting=Signal::from(false)
                on_confirm=Callback::new(|()| {})
                on_close=Callback::new(|()| {})
            />
        }
        .into_any()
    });
    tick().await;

    assert!(
        wrapper
            .query_selector("[data-testid=\"dcm1\"]")
            .unwrap()
            .is_some(),
        "open modal must render"
    );
    assert!(
        wrapper
            .query_selector("[data-testid=\"dcm1-cancel\"]")
            .unwrap()
            .is_some()
    );
    assert!(
        wrapper
            .query_selector("[data-testid=\"dcm1-confirm\"]")
            .unwrap()
            .is_some()
    );
}

#[wasm_bindgen_test]
async fn delete_confirm_modal_deleting_disables_buttons() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let is_open = RwSignal::new(true);
        view! {
            <DeleteConfirmModal
                test_id="dcm2"
                is_open=is_open
                is_deleting=Signal::from(true)
                on_confirm=Callback::new(|()| {})
                on_close=Callback::new(|()| {})
            />
        }
        .into_any()
    });
    tick().await;

    let confirm = wrapper
        .query_selector("[data-testid=\"dcm2-confirm\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlButtonElement>();
    assert!(
        confirm.disabled(),
        "confirm must be disabled while deleting"
    );

    let spinner = wrapper.query_selector("[data-testid=\"dcm2\"] .spinner");
    assert!(
        spinner.is_ok_and(|s| s.is_some()),
        "deleting state must show a spinner"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Dropdown
// ═══════════════════════════════════════════════════════════════════════

fn dropdown_options() -> Vec<DropdownItem> {
    vec![
        DropdownItem {
            value: "n5".into(),
            label: "N5".into(),
        },
        DropdownItem {
            value: "n4".into(),
            label: "N4".into(),
        },
        DropdownItem {
            value: "n3".into(),
            label: "N3".into(),
        },
    ]
}

#[wasm_bindgen_test]
async fn dropdown_trigger_shows_selected_label() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let selected = RwSignal::new("n4".to_string());
        let options = dropdown_options();
        view! {
            <Dropdown
                options=Signal::derive(move || options.clone())
                selected=selected
                test_id="dd1"
            />
        }
        .into_any()
    });
    tick().await;

    let trigger = wrapper
        .query_selector("[data-testid=\"dd1-trigger\"]")
        .unwrap()
        .unwrap();
    assert!(
        trigger.text_content().unwrap().contains("N4"),
        "trigger must show the selected label"
    );
}

#[wasm_bindgen_test]
async fn dropdown_unknown_value_shows_placeholder() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let selected = RwSignal::new("zzz".to_string());
        let options = dropdown_options();
        view! {
            <Dropdown
                options=Signal::derive(move || options.clone())
                selected=selected
                placeholder=Signal::derive(|| "Pick…".to_string())
                test_id="dd2"
            />
        }
        .into_any()
    });
    tick().await;

    let trigger = wrapper
        .query_selector("[data-testid=\"dd2-trigger\"]")
        .unwrap()
        .unwrap();
    assert!(
        trigger.text_content().unwrap().contains("Pick…"),
        "placeholder must show for unknown selection"
    );
}

#[wasm_bindgen_test]
async fn dropdown_select_item_updates_signal() {
    let wrapper = create_wrapper();
    let (set_selected, get_selected) = shared_cell::<RwSignal<String>>();
    mount_with_i18n(&wrapper, move || {
        let selected = RwSignal::new("n5".to_string());
        set_selected.set(Some(selected));
        let options = dropdown_options();
        view! {
            <Dropdown
                options=Signal::derive(move || options.clone())
                selected=selected
                test_id="dd3"
            />
        }
        .into_any()
    });
    let selected = get_selected.get().expect("captured");
    tick().await;

    // Act: click the N3 option
    wrapper
        .query_selector("[data-testid=\"dd3-option-n3\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    assert_eq!(selected.get(), "n3", "option click must set the signal");
}

// ═══════════════════════════════════════════════════════════════════════
// NativeLanguageToggle
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn language_toggle_selected_language_marked_current() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let lang = RwSignal::new(origa::domain::NativeLanguage::Russian);
        view! { <NativeLanguageToggle selected_language=lang test_id="lt1" /> }.into_any()
    });
    tick().await;

    let ru = wrapper
        .query_selector("[data-testid=\"lang-toggle-ru\"]")
        .unwrap()
        .unwrap();
    // Both buttons carry aria-current ("true"/"false"); the selected one
    // must read "true".
    assert_eq!(ru.get_attribute("aria-current"), Some("true".into()));
    let en = wrapper
        .query_selector("[data-testid=\"lang-toggle-en\"]")
        .unwrap()
        .unwrap();
    assert!(
        en.get_attribute("aria-current").is_none_or(|v| v != "true"),
        "unselected EN must not be marked current"
    );
}

#[wasm_bindgen_test]
async fn language_toggle_click_switches_language() {
    let wrapper = create_wrapper();
    let (set_lang, get_lang) = shared_cell::<RwSignal<origa::domain::NativeLanguage>>();
    mount_with_i18n(&wrapper, move || {
        let lang = RwSignal::new(origa::domain::NativeLanguage::Russian);
        set_lang.set(Some(lang));
        view! { <NativeLanguageToggle selected_language=lang test_id="lt2" /> }.into_any()
    });
    let lang = get_lang.get().expect("captured");
    tick().await;

    // Act
    wrapper
        .query_selector("[data-testid=\"lang-toggle-en\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    // Assert
    assert_eq!(
        lang.get(),
        origa::domain::NativeLanguage::English,
        "EN click must switch the signal"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// legal_links
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn legal_links_renders_privacy_and_terms() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        crate::ui_components::legal_links(Signal::derive(|| "ll1".to_string()))
    });
    tick().await;

    assert!(
        wrapper
            .query_selector("[data-testid=\"legal-links-privacy\"]")
            .unwrap()
            .is_some()
    );
    assert!(
        wrapper
            .query_selector("[data-testid=\"legal-links-terms\"]")
            .unwrap()
            .is_some()
    );
}

// ═══════════════════════════════════════════════════════════════════════
// SelectedCount
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn selected_count_positive_renders_number() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! { <SelectedCount count=Signal::from(7usize) /> }.into_any()
    });
    tick().await;

    let text = wrapper.text_content().unwrap_or_default();
    assert!(text.contains('7'), "count must be visible; got: {text}");
}

#[wasm_bindgen_test]
async fn selected_count_zero_renders_nothing() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! { <SelectedCount count=Signal::from(0usize) /> }.into_any()
    });
    tick().await;

    // Zero selection renders the empty () view — no <p> from Text.
    let paragraphs = wrapper.query_selector_all("p").unwrap().length();
    assert_eq!(paragraphs, 0, "zero selection must render nothing");
}

// ═══════════════════════════════════════════════════════════════════════
// UpdateDrawer
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn update_drawer_shows_both_versions_and_button() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! {
            <UpdateDrawer
                test_id="ud1"
                current_version="1.0.0".to_string()
                new_version="1.1.0".to_string()
                on_update=Callback::new(|()| {})
                download_progress=Signal::from(None::<f32>)
            />
        }
        .into_any()
    });
    tick().await;

    let text = wrapper
        .query_selector("[data-testid=\"ud1\"]")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(text.contains("1.0.0"), "got: {text}");
    assert!(text.contains("1.1.0"), "got: {text}");
    assert!(
        wrapper
            .query_selector("[data-testid=\"ud1-update\"]")
            .unwrap()
            .is_some(),
        "update button must render when not downloading"
    );
}

#[wasm_bindgen_test]
async fn update_drawer_downloading_shows_progress_instead() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! {
            <UpdateDrawer
                test_id="ud2"
                current_version="1.0.0".to_string()
                new_version="1.1.0".to_string()
                on_update=Callback::new(|()| {})
                download_progress=Signal::from(Some(0.5f32))
            />
        }
        .into_any()
    });
    tick().await;

    assert!(
        wrapper
            .query_selector("[data-testid=\"ud2-update\"]")
            .unwrap()
            .is_none(),
        "update button must hide while downloading"
    );
    assert!(
        wrapper
            .query_selector("[data-testid=\"ud2-progress\"]")
            .unwrap()
            .is_some(),
        "progress area must render while downloading"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Toast / ToastContainer
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn toast_success_type_class_and_content() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        let toast = ToastData {
            id: 1,
            toast_type: ToastType::Success,
            title: "Saved".into(),
            message: "Your card is stored".into(),
            duration_ms: None,
            closable: true,
        };
        view! { <Toast toast=toast on_close=Callback::new(|_| ()) /> }.into_any()
    });
    tick().await;

    let el = wrapper
        .query_selector("[data-testid=\"toast-1\"]")
        .unwrap()
        .unwrap();
    let class = el.get_attribute("class").unwrap_or_default();
    assert!(class.contains("toast-success"), "got: {class}");

    let text = el.text_content().unwrap();
    assert!(text.contains("Saved"), "got: {text}");
    assert!(text.contains("Your card is stored"), "got: {text}");

    assert!(
        wrapper
            .query_selector("[data-testid=\"toast-1-close\"]")
            .unwrap()
            .is_some(),
        "closable toast must render the close button"
    );
}

#[wasm_bindgen_test]
async fn toast_not_closable_has_no_close_button() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        let toast = ToastData {
            id: 2,
            toast_type: ToastType::Info,
            title: "Info".into(),
            message: String::new(),
            duration_ms: None,
            closable: false,
        };
        view! { <Toast toast=toast on_close=Callback::new(|_| ()) /> }.into_any()
    });
    tick().await;

    assert!(
        wrapper
            .query_selector("[data-testid=\"toast-2-close\"]")
            .unwrap()
            .is_none(),
        "non-closable toast must not render the close button"
    );
}

#[wasm_bindgen_test]
async fn toast_container_close_button_removes_after_animation() {
    let wrapper = create_wrapper();
    let (set_toasts, get_toasts) = shared_cell::<RwSignal<Vec<ToastData>>>();
    mount_to_wrapper(&wrapper, move || {
        let toasts = RwSignal::new(vec![ToastData {
            id: 5,
            toast_type: ToastType::Info,
            title: "Temporary".into(),
            message: String::new(),
            duration_ms: None,
            closable: true,
        }]);
        set_toasts.set(Some(toasts));
        view! { <ToastContainer toasts=toasts test_id="tc1" /> }.into_any()
    });
    let toasts = get_toasts.get().expect("captured");
    tick().await;

    assert!(
        wrapper
            .query_selector("[data-testid=\"toast-5\"]")
            .unwrap()
            .is_some()
    );

    // Act: click close, poll until the 200 ms exit animation removes it
    wrapper
        .query_selector("[data-testid=\"toast-5-close\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    // Assert
    let removed = crate::test_support::wait_until(move || toasts.get().is_empty(), 20, 25).await;
    assert!(removed, "toast must be removed after the exit animation");
}

// ═══════════════════════════════════════════════════════════════════════
// TranslatorText
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn translator_without_language_renders_plain_fallback() {
    // No native_language prop and no LessonContext → the fallback span.
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <TranslatorText text="こんにちは".to_string() test_id="tr1" /> }.into_any()
    });
    tick().await;

    let el = wrapper
        .query_selector("[data-testid=\"tr1\"]")
        .unwrap()
        .unwrap();
    let text = el.text_content().unwrap();
    assert!(
        text.contains("こんにちは"),
        "fallback must keep the original text; got: {text}"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// ConnectivityBanner
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn connectivity_banner_hidden_while_online() {
    let wrapper = create_wrapper();
    mount_with_stores(&wrapper, |_auth, _connectivity| {
        // Fresh stores default to online → the banner stays hidden.
        view! { <ConnectivityBanner test_id="cb1" /> }.into_any()
    });
    tick().await;

    assert!(
        wrapper
            .query_selector("[data-testid=\"cb1\"]")
            .unwrap()
            .is_none(),
        "online store must hide the banner"
    );
}

#[wasm_bindgen_test]
async fn connectivity_banner_visible_when_offline() {
    let wrapper = create_wrapper();
    mount_with_stores(&wrapper, |_auth, connectivity| {
        connectivity.is_online.set(false);
        view! { <ConnectivityBanner test_id="cb2" /> }.into_any()
    });
    tick().await;

    let banner = wrapper.query_selector(".connectivity-banner");
    assert!(
        banner.is_ok_and(|b| b.is_some()),
        "offline store must show the banner"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Sidebar & BottomTabBar (Router + AuthStore)
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn sidebar_authenticated_user_renders_nav_items() {
    let wrapper = create_wrapper();
    let _mount = mount_with_router_and_stores(&wrapper, Some(test_user("rin")), || {
        let user = RwSignal::new(Some(test_user("rin")));
        view! { <Sidebar current_user=user test_id="sb1" /> }.into_any()
    });
    tick().await;

    // 5 sidebar routes + profile item
    for suffix in [
        "tab-home",
        "tab-words",
        "tab-grammar",
        "tab-kanji",
        "tab-phrases",
        "tab-profile",
    ] {
        let el = wrapper.query_selector(&format!("[data-testid=\"sb1-{suffix}\"]"));
        assert!(
            el.is_ok_and(|e| e.is_some()),
            "sidebar item {suffix} must render for an authenticated user"
        );
    }
}

#[wasm_bindgen_test]
async fn sidebar_unauthenticated_renders_nothing() {
    let wrapper = create_wrapper();
    let _mount = mount_with_router_and_stores(&wrapper, None, || {
        let user = RwSignal::new(None);
        view! { <Sidebar current_user=user test_id="sb2" /> }.into_any()
    });
    tick().await;

    assert!(
        wrapper
            .query_selector("[data-testid=\"sb2\"]")
            .unwrap()
            .is_none(),
        "unauthenticated user must hide the sidebar"
    );
}

#[wasm_bindgen_test]
async fn bottom_tab_bar_authenticated_renders_six_tabs() {
    let wrapper = create_wrapper();
    let _mount = mount_with_router_and_stores(&wrapper, Some(test_user("rin")), || {
        view! { <BottomTabBar test_id="btb1" /> }.into_any()
    });
    tick().await;

    let items = wrapper
        .query_selector_all("[data-testid=\"btb1\"] .bottom-tab-item")
        .unwrap()
        .length();
    assert_eq!(items, 6, "six nav routes must render; got {items}");
}

#[wasm_bindgen_test]
async fn bottom_tab_bar_unauthenticated_renders_nothing() {
    let wrapper = create_wrapper();
    let _mount = mount_with_router_and_stores(&wrapper, None, || {
        view! { <BottomTabBar test_id="btb2" /> }.into_any()
    });
    tick().await;

    assert!(
        wrapper
            .query_selector("[data-testid=\"btb2\"]")
            .unwrap()
            .is_none(),
        "unauthenticated user must hide the bottom bar"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// OfflineBundleCard
// ═══════════════════════════════════════════════════════════════════════
// The download button depends on an async OPFS cache check; the initial
// render (checking state, no buttons) is asserted here — the full flow is
// E2E territory.

#[wasm_bindgen_test]
async fn offline_bundle_card_renders_heading_without_buttons_initially() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! { <crate::ui_components::OfflineBundleCard test_id="obc1" /> }.into_any()
    });
    tick().await;

    let el = wrapper
        .query_selector("[data-testid=\"obc1\"]")
        .unwrap()
        .unwrap();
    let text = el.text_content().unwrap();
    assert!(!text.is_empty(), "card must render its copy");

    // Before the async cache check settles neither button may be visible.
    assert!(
        el.query_selector("[data-testid=\"download-bundle-btn\"]")
            .ok()
            .flatten()
            .is_none(),
        "download button must not render in checking state"
    );
    assert!(
        el.query_selector("[data-testid=\"cancel-bundle-btn\"]")
            .ok()
            .flatten()
            .is_none(),
        "cancel button must not render in checking state"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// KanjiAnimation (SVG shell; the CDN asset itself is not loaded)
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn kanji_animation_frames_mode_renders_img_shell() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! {
            <crate::ui_components::kanji_animation::KanjiAnimation
                kanji="山".to_string()
                mode=crate::ui_components::KanjiViewMode::Frames
                fallback=None
                test_id="ka1"
            />
        }
        .into_any()
    });
    tick().await;

    // Without the CDN asset the component still mounts its shell with an
    // addressable test id; assert the container node itself exists.
    let shell = wrapper
        .query_selector("[data-testid=\"ka1\"]")
        .ok()
        .flatten()
        .expect("kanji animation shell must mount with the test id");
}

// ═══════════════════════════════════════════════════════════════════════
// LoadingStageItem (OCR loading stage row)
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn loading_stage_item_completed_state_renders() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        use crate::ui_components::ocr_loading_stage::{LoadingStageItem, StageStatus};
        view! {
            <LoadingStageItem
                status=StageStatus::Completed
                title="Dictionary".to_string()
                description="loaded".to_string()
                test_id="lsi1"
            />
        }
        .into_any()
    });
    tick().await;

    let item = wrapper
        .query_selector("[data-testid=\"lsi1\"]")
        .unwrap()
        .unwrap();
    let text = item.text_content().unwrap();
    assert!(
        text.contains("Dictionary"),
        "title must render; got: {text}"
    );
}

#[wasm_bindgen_test]
async fn loading_stage_item_error_state_shows_message() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        use crate::ui_components::ocr_loading_stage::{LoadingStageItem, StageStatus};
        view! {
            <LoadingStageItem
                status=StageStatus::Error
                title="Model".to_string()
                description="download".to_string()
                error_message=Some("network down".to_string())
                test_id="lsi2"
            />
        }
        .into_any()
    });
    tick().await;

    let item = wrapper
        .query_selector("[data-testid=\"lsi2\"]")
        .unwrap()
        .unwrap();
    let text = item.text_content().unwrap();
    assert!(
        text.contains("network down"),
        "the error message must render; got: {text}"
    );
}
