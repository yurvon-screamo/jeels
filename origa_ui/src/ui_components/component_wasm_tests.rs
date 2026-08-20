//! Component-level WASM tests for pure UI components.
//!
//! These tests mount real Leptos components into a headless browser DOM
//! and assert on the rendered HTML — catching bugs that `cargo test`
//! (no rendering) and BDD (testid-only assertions) miss.
//!
//! All components in this file have **zero external dependencies**
//! (no i18n, no context, no effects) — they are pure presentational.
//!
//! Run locally:
//! ```bash
//! wasm-pack test --headless --chrome origa_ui --features csr -- component_wasm_tests
//! ```

#![cfg(all(target_arch = "wasm32", test))]

use leptos::prelude::*;
use leptos::task::tick;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

use crate::ui_components::{
    Avatar, AvatarSize, Button, ButtonSize, ButtonVariant, Card, Checkbox, DeleteButton, Divider,
    DividerVariant, FavoriteButton, FilterTag, Logo, LogoSize, Tag, TagVariant,
};

use crate::test_support::{create_wrapper, mount_to_wrapper};

// ═══════════════════════════════════════════════════════════════════════
// Tag
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn tag_default_variant_no_special_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Tag variant=TagVariant::Default>"N5"</Tag> }.into_any()
    });
    tick().await;

    let el = wrapper.query_selector(".tag").unwrap().unwrap();
    let class = el.get_attribute("class").unwrap_or_default();
    assert!(
        !class.contains("tag-filled") && !class.contains("tag-olive"),
        "default variant must not add special class; got: {class}"
    );
}

#[wasm_bindgen_test]
async fn tag_olive_variant_adds_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Tag variant=TagVariant::Olive>"Kanji"</Tag> }.into_any()
    });
    tick().await;

    let el = wrapper.query_selector(".tag").unwrap().unwrap();
    let class = el.get_attribute("class").unwrap_or_default();
    assert!(class.contains("tag-olive"), "got: {class}");
}

#[wasm_bindgen_test]
async fn tag_terracotta_variant_adds_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Tag variant=TagVariant::Terracotta>"Grammar"</Tag> }.into_any()
    });
    tick().await;

    let el = wrapper.query_selector(".tag").unwrap().unwrap();
    let class = el.get_attribute("class").unwrap_or_default();
    assert!(class.contains("tag-terracotta"), "got: {class}");
}

#[wasm_bindgen_test]
async fn tag_sage_variant_adds_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Tag variant=TagVariant::Sage>"Phrase"</Tag> }.into_any()
    });
    tick().await;

    let el = wrapper.query_selector(".tag").unwrap().unwrap();
    let class = el.get_attribute("class").unwrap_or_default();
    assert!(class.contains("tag-sage"), "got: {class}");
}

#[wasm_bindgen_test]
async fn tag_filled_variant_adds_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Tag variant=TagVariant::Filled>"Vocab"</Tag> }.into_any()
    });
    tick().await;

    let el = wrapper.query_selector(".tag").unwrap().unwrap();
    let class = el.get_attribute("class").unwrap_or_default();
    assert!(class.contains("tag-filled"), "got: {class}");
}

#[wasm_bindgen_test]
async fn tag_renders_children_text() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Tag variant=TagVariant::Olive test_id="my-tag">"漢字"</Tag> }.into_any()
    });
    tick().await;

    let el = wrapper
        .query_selector("[data-testid=\"my-tag\"]")
        .unwrap()
        .unwrap();
    let text = el.text_content().unwrap();
    assert!(text.contains("漢字"), "got: {text}");
}

#[wasm_bindgen_test]
async fn tag_test_id_attribute_set() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Tag variant=TagVariant::Default test_id="level-tag">"N4"</Tag> }.into_any()
    });
    tick().await;

    let el = wrapper.query_selector("[data-testid=\"level-tag\"]");
    assert!(
        el.is_ok_and(|e| e.is_some()),
        "tag must be findable by test_id"
    );
}

#[wasm_bindgen_test]
async fn tag_empty_test_id_no_attribute() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Tag variant=TagVariant::Default>"N3"</Tag> }.into_any()
    });
    tick().await;

    let el = wrapper.query_selector(".tag").unwrap().unwrap();
    assert!(
        !el.has_attribute("data-testid"),
        "tag without explicit test_id must not have data-testid"
    );
}

#[wasm_bindgen_test]
async fn tag_with_on_click_renders_button() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Tag variant=TagVariant::Default
                on_click=Callback::new(|_ev: leptos::ev::MouseEvent| {})
                test_id="clickable-tag"
            >"Click"</Tag>
        }
        .into_any()
    });
    tick().await;

    let el = wrapper.query_selector("button.tag").unwrap().unwrap();
    let class = el.get_attribute("class").unwrap_or_default();
    assert!(class.contains("tag-clickable"), "got: {class}");
}

#[wasm_bindgen_test]
async fn tag_without_on_click_renders_span() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Tag variant=TagVariant::Default>"Plain"</Tag> }.into_any()
    });
    tick().await;

    let el = wrapper.query_selector("span.tag").unwrap().unwrap();
    let class = el.get_attribute("class").unwrap_or_default();
    assert!(!class.contains("tag-clickable"), "got: {class}");
}

// ═══════════════════════════════════════════════════════════════════════
// Button
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn button_default_variant_no_special_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Button variant=ButtonVariant::Default>"OK"</Button> }.into_any()
    });
    tick().await;

    let el = wrapper.query_selector("button.btn").unwrap().unwrap();
    let class = el.get_attribute("class").unwrap_or_default();
    assert!(
        !class.contains("btn-filled")
            && !class.contains("btn-olive")
            && !class.contains("btn-ghost"),
        "default variant must not add special class; got: {class}"
    );
}

#[wasm_bindgen_test]
async fn button_filled_variant_adds_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Button variant=ButtonVariant::Filled>"Submit"</Button> }.into_any()
    });
    tick().await;

    let el = wrapper.query_selector("button.btn").unwrap().unwrap();
    let class = el.get_attribute("class").unwrap_or_default();
    assert!(class.contains("btn-filled"), "got: {class}");
}

#[wasm_bindgen_test]
async fn button_olive_variant_adds_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Button variant=ButtonVariant::Olive>"Next"</Button> }.into_any()
    });
    tick().await;

    let el = wrapper.query_selector("button.btn").unwrap().unwrap();
    let class = el.get_attribute("class").unwrap_or_default();
    assert!(class.contains("btn-olive"), "got: {class}");
}

#[wasm_bindgen_test]
async fn button_ghost_variant_adds_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Button variant=ButtonVariant::Ghost>"Skip"</Button> }.into_any()
    });
    tick().await;

    let el = wrapper.query_selector("button.btn").unwrap().unwrap();
    let class = el.get_attribute("class").unwrap_or_default();
    assert!(class.contains("btn-ghost"), "got: {class}");
}

#[wasm_bindgen_test]
async fn button_small_size_adds_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Button variant=ButtonVariant::Default size=ButtonSize::Small>"S"</Button> }
            .into_any()
    });
    tick().await;

    let el = wrapper.query_selector("button.btn").unwrap().unwrap();
    let class = el.get_attribute("class").unwrap_or_default();
    assert!(class.contains("btn-sm"), "got: {class}");
}

#[wasm_bindgen_test]
async fn button_disabled_attribute() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Button variant=ButtonVariant::Default disabled=true>"Disabled"</Button> }
            .into_any()
    });
    tick().await;

    let el = wrapper
        .query_selector("button.btn")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlButtonElement>();
    assert!(el.disabled(), "button must be disabled");
}

#[wasm_bindgen_test]
async fn button_loading_shows_spinner_and_disables() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Button variant=ButtonVariant::Default loading=true>"Loading"</Button> }.into_any()
    });
    tick().await;

    let spinner = wrapper.query_selector(".btn-spinner");
    assert!(
        spinner.is_ok_and(|e| e.is_some()),
        "spinner must be present when loading"
    );

    let el = wrapper
        .query_selector("button.btn")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlButtonElement>();
    assert!(el.disabled(), "loading button must be disabled");
}

#[wasm_bindgen_test]
async fn button_renders_children() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Button variant=ButtonVariant::Default>"Submit Form"</Button> }.into_any()
    });
    tick().await;

    let text = wrapper
        .query_selector("button.btn")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(text.contains("Submit Form"), "got: {text}");
}

#[wasm_bindgen_test]
async fn button_test_id_set() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Button variant=ButtonVariant::Default test_id="submit-btn">"OK"</Button> }
            .into_any()
    });
    tick().await;

    let el = wrapper.query_selector("[data-testid=\"submit-btn\"]");
    assert!(
        el.is_ok_and(|e| e.is_some()),
        "button must be findable by test_id"
    );
}

#[wasm_bindgen_test]
async fn button_default_type_is_button() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Button variant=ButtonVariant::Default>"OK"</Button> }.into_any()
    });
    tick().await;

    let el = wrapper
        .query_selector("button.btn")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlButtonElement>();
    assert_eq!(el.type_(), "button", "default button type must be 'button'");
}

// ═══════════════════════════════════════════════════════════════════════
// Checkbox
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn checkbox_checked_state() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Checkbox checked=true test_id="cb1" /> }.into_any()
    });
    tick().await;

    let el = wrapper
        .query_selector("[data-testid=\"cb1\"] input[type=\"checkbox\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlInputElement>();
    assert!(el.checked(), "checkbox must be checked");
}

#[wasm_bindgen_test]
async fn checkbox_unchecked_state() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Checkbox checked=false test_id="cb2" /> }.into_any()
    });
    tick().await;

    let el = wrapper
        .query_selector("[data-testid=\"cb2\"] input[type=\"checkbox\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlInputElement>();
    assert!(!el.checked(), "checkbox must be unchecked");
}

#[wasm_bindgen_test]
async fn checkbox_disabled_state() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Checkbox checked=false disabled=true test_id="cb3" /> }.into_any()
    });
    tick().await;

    let el = wrapper
        .query_selector("[data-testid=\"cb3\"] input[type=\"checkbox\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlInputElement>();
    assert!(el.disabled(), "checkbox must be disabled");
}

#[wasm_bindgen_test]
async fn checkbox_renders_sr_only_label() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Checkbox checked=false label="Accept terms" test_id="cb4" /> }.into_any()
    });
    tick().await;

    let el = wrapper
        .query_selector("[data-testid=\"cb4\"] .sr-only")
        .unwrap()
        .unwrap();
    let text = el.text_content().unwrap();
    assert!(
        text.contains("Accept terms"),
        "sr-only label must contain text; got: {text}"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Card
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn card_renders_children() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Card test_id="card1">
                <p>"Content inside card"</p>
            </Card>
        }
        .into_any()
    });
    tick().await;

    let text = wrapper
        .query_selector("[data-testid=\"card1\"]")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(text.contains("Content inside card"), "got: {text}");
}

#[wasm_bindgen_test]
async fn card_shadow_adds_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Card shadow=true test_id="card2">"Shadow"</Card> }.into_any()
    });
    tick().await;

    let el = wrapper
        .query_selector("[data-testid=\"card2\"]")
        .unwrap()
        .unwrap();
    let class = el.get_attribute("class").unwrap_or_default();
    assert!(class.contains("card-shadow"), "got: {class}");
}

#[wasm_bindgen_test]
async fn card_borderless_adds_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Card borderless=true test_id="card3">"No border"</Card> }.into_any()
    });
    tick().await;

    let el = wrapper
        .query_selector("[data-testid=\"card3\"]")
        .unwrap()
        .unwrap();
    let class = el.get_attribute("class").unwrap_or_default();
    assert!(class.contains("border-none"), "got: {class}");
}

// ═══════════════════════════════════════════════════════════════════════
// Divider
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn divider_single_variant_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Divider variant=DividerVariant::Single test_id="d1" /> }.into_any()
    });
    tick().await;

    let el = wrapper
        .query_selector("[data-testid=\"d1\"]")
        .unwrap()
        .unwrap();
    let class = el.get_attribute("class").unwrap_or_default();
    assert!(class.contains("divider"), "got: {class}");
}

#[wasm_bindgen_test]
async fn divider_double_variant_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Divider variant=DividerVariant::Double test_id="d2" /> }.into_any()
    });
    tick().await;

    let el = wrapper
        .query_selector("[data-testid=\"d2\"]")
        .unwrap()
        .unwrap();
    let class = el.get_attribute("class").unwrap_or_default();
    assert!(class.contains("divider-double"), "got: {class}");
}

// ═══════════════════════════════════════════════════════════════════════
// Avatar
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn avatar_default_size_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Avatar size=AvatarSize::Default initials="JT" test_id="av1" /> }.into_any()
    });
    tick().await;

    let el = wrapper
        .query_selector("[data-testid=\"av1\"]")
        .unwrap()
        .unwrap();
    let class = el.get_attribute("class").unwrap_or_default();
    assert!(class.contains("avatar"), "got: {class}");
}

#[wasm_bindgen_test]
async fn avatar_small_size_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Avatar size=AvatarSize::Small initials="JT" test_id="av2" /> }.into_any()
    });
    tick().await;

    let el = wrapper
        .query_selector("[data-testid=\"av2\"]")
        .unwrap()
        .unwrap();
    let class = el.get_attribute("class").unwrap_or_default();
    assert!(class.contains("avatar-sm"), "got: {class}");
}

#[wasm_bindgen_test]
async fn avatar_renders_initials() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Avatar size=AvatarSize::Default initials="JT" test_id="av3" /> }.into_any()
    });
    tick().await;

    let text = wrapper
        .query_selector("[data-testid=\"av3\"]")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(text.contains("JT"), "got: {text}");
}

// ═══════════════════════════════════════════════════════════════════════
// FilterTag
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn filter_tag_active_is_filled() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <FilterTag
                label="N5".to_string()
                is_active=Signal::from(true)
                on_click=Callback::new(|_ev: leptos::ev::MouseEvent| {})
                test_id="ft1"
            />
        }
        .into_any()
    });
    tick().await;

    let el = wrapper
        .query_selector("[data-testid=\"ft1\"]")
        .unwrap()
        .unwrap();
    let class = el.get_attribute("class").unwrap_or_default();
    assert!(
        class.contains("tag-filled"),
        "active FilterTag must be filled; got: {class}"
    );
}

#[wasm_bindgen_test]
async fn filter_tag_inactive_is_default() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <FilterTag
                label="N4".to_string()
                is_active=Signal::from(false)
                on_click=Callback::new(|_ev: leptos::ev::MouseEvent| {})
                test_id="ft2"
            />
        }
        .into_any()
    });
    tick().await;

    let el = wrapper
        .query_selector("[data-testid=\"ft2\"]")
        .unwrap()
        .unwrap();
    let class = el.get_attribute("class").unwrap_or_default();
    assert!(
        !class.contains("tag-filled"),
        "inactive FilterTag must not be filled; got: {class}"
    );
}

#[wasm_bindgen_test]
async fn filter_tag_renders_as_button() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <FilterTag
                label="N3".to_string()
                is_active=Signal::from(false)
                on_click=Callback::new(|_ev: leptos::ev::MouseEvent| {})
                test_id="ft3"
            />
        }
        .into_any()
    });
    tick().await;

    let button = wrapper.query_selector("button[data-testid=\"ft3\"]");
    assert!(
        button.is_ok_and(|e| e.is_some()),
        "FilterTag must render as <button> because it has on_click"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// DeleteButton
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn delete_button_renders_svg() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <DeleteButton on_click=Callback::new(|_| {}) test_id="del1" /> }.into_any()
    });
    tick().await;

    let svg = wrapper.query_selector("[data-testid=\"del1\"] svg");
    assert!(
        svg.is_ok_and(|e| e.is_some()),
        "DeleteButton must render an SVG icon"
    );
}

#[wasm_bindgen_test]
async fn delete_button_has_danger_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <DeleteButton on_click=Callback::new(|_| {}) test_id="del2" /> }.into_any()
    });
    tick().await;

    let el = wrapper
        .query_selector("[data-testid=\"del2\"]")
        .unwrap()
        .unwrap();
    let class = el.get_attribute("class").unwrap_or_default();
    assert!(
        class.contains("icon-btn-danger"),
        "DeleteButton must have danger class; got: {class}"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// FavoriteButton
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn favorite_not_favorite_shows_outline_path() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <FavoriteButton
                is_favorite=Signal::from(false)
                on_click=Callback::new(|_| {})
                test_id="fav1"
            />
        }
        .into_any()
    });
    tick().await;

    let html = wrapper
        .query_selector("[data-testid=\"fav1\"]")
        .unwrap()
        .unwrap()
        .inner_html();
    assert!(
        html.contains("fill=\"none\""),
        "not-favorite must have outline path (fill=none); got: {html}"
    );
}

#[wasm_bindgen_test]
async fn favorite_is_favorite_shows_filled_path() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <FavoriteButton
                is_favorite=Signal::from(true)
                on_click=Callback::new(|_| {})
                test_id="fav2"
            />
        }
        .into_any()
    });
    tick().await;

    let html = wrapper
        .query_selector("[data-testid=\"fav2\"]")
        .unwrap()
        .unwrap()
        .inner_html();
    assert!(
        html.contains("fill=\"currentColor\""),
        "favorite must have filled path (fill=currentColor); got: {html}"
    );
}

#[wasm_bindgen_test]
async fn favorite_pending_shows_spinner_and_disables() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <FavoriteButton
                is_favorite=Signal::from(false)
                on_click=Callback::new(|_| {})
                pending=true
                test_id="fav3"
            />
        }
        .into_any()
    });
    tick().await;

    let spinner = wrapper.query_selector("[data-testid=\"fav3\"] .spinner");
    assert!(
        spinner.is_ok_and(|e| e.is_some()),
        "pending FavoriteButton must show spinner"
    );

    let el = wrapper
        .query_selector("[data-testid=\"fav3\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlButtonElement>();
    assert!(el.disabled(), "pending FavoriteButton must be disabled");
}

// ═══════════════════════════════════════════════════════════════════════
// Logo
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn logo_renders_img_element() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Logo size=LogoSize::Md test_id="logo1" /> }.into_any()
    });
    tick().await;

    let img = wrapper
        .query_selector("[data-testid=\"logo1\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlImageElement>();
    assert!(
        img.get_attribute("src")
            .unwrap_or_default()
            .contains("logo"),
        "Logo src must contain 'logo'; got: {:?}",
        img.get_attribute("src")
    );
}

#[wasm_bindgen_test]
async fn logo_sm_dimensions() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Logo size=LogoSize::Sm test_id="logo2" /> }.into_any()
    });
    tick().await;

    let img = wrapper
        .query_selector("[data-testid=\"logo2\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlImageElement>();
    assert_eq!(img.width(), 32, "Sm logo width must be 32");
    assert_eq!(img.height(), 32, "Sm logo height must be 32");
}
