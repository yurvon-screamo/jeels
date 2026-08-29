mod canvas_renderer;
mod event_handlers;
mod stroke_comparison;
mod svg_parser;

use crate::i18n::{t, use_i18n};
use crate::repository::cdn_provider;
use canvas_renderer::redraw_canvas;
use leptos::ev::PointerEvent;
use leptos::html::Canvas;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use origa::traits::CdnProvider;
use std::sync::{Arc, Mutex};
use svg_parser::parse_stroke_paths;
use web_sys::CanvasRenderingContext2d;

pub(super) const CANVAS_SIZE: u32 = 320;
pub(super) const SVG_VIEWBOX_SIZE: f64 = 109.0;
pub(super) const SVG_SCALE: f64 = CANVAS_SIZE as f64 / SVG_VIEWBOX_SIZE;
pub(super) const CANVAS_TOLERANCE: f64 = 25.0;
pub(super) const SUCCESS_THRESHOLD: f64 = 0.80;
pub(super) const SAMPLE_COUNT: usize = 21;

pub(super) const MIN_STROKE_POINTS: usize = 8;
pub(super) const STROKE_LINE_WIDTH: f64 = 12.0;
pub(super) const HINT_LINE_WIDTH: f64 = 4.0;
pub(super) const USER_LINE_WIDTH: f64 = 8.0;

#[derive(Clone, Default)]
pub(super) struct StrokeData {
    pub d: String,
}

#[derive(Clone, Default)]
struct DrawingState {
    points: Vec<(f64, f64)>,
    is_drawing: bool,
}

type CanvasContext = Arc<Mutex<Option<CanvasRenderingContext2d>>>;
type DrawingStateRef = Arc<Mutex<DrawingState>>;

#[component]
pub fn KanjiDrawingPractice(
    kanji: String,
    #[prop(optional)] on_complete: Option<Callback<()>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let svg_content = LocalResource::new(move || {
        let encoded = urlencoding::encode(&kanji);
        let path = format!("kanji_animations/{}.svg", encoded);
        async move {
            let cdn = cdn_provider();
            cdn.fetch_text(&path).await.ok()
        }
    });

    let strokes = RwSignal::new(Vec::<StrokeData>::new());
    let current_stroke_index = RwSignal::new(0usize);
    let is_completed = RwSignal::new(false);
    let load_error = RwSignal::new(false);
    let canvas_ref: NodeRef<Canvas> = NodeRef::new();
    let drawing_state: DrawingStateRef = Arc::new(Mutex::new(DrawingState::default()));
    let ctx_storage: CanvasContext = Arc::new(Mutex::new(None));
    let ctx_storage_clone = ctx_storage.clone();
    Effect::new(move |_| match svg_content.get() {
        Some(Some(svg)) => {
            let parsed = parse_stroke_paths(&svg);
            if parsed.is_empty() {
                load_error.set(true);
            } else {
                strokes.set(parsed);
                current_stroke_index.set(0);
                is_completed.set(false);
                load_error.set(false);
            }
        },
        Some(None) => {
            load_error.set(true);
        },
        None => {},
    });
    Effect::new(move |_| {
        let canvas = canvas_ref.get()?;
        let canvas: web_sys::HtmlCanvasElement = canvas.unchecked_into();
        let ctx = canvas
            .get_context("2d")
            .ok()?
            .and_then(|v| v.dyn_into::<CanvasRenderingContext2d>().ok())?;
        redraw_canvas(
            &ctx,
            &strokes.get(),
            current_stroke_index.get(),
            CANVAS_SIZE,
        );
        ctx_storage_clone.lock().ok()?.replace(ctx);
        Some(())
    });
    let handle_pointer_down = {
        let state = drawing_state.clone();
        let ctx_store = ctx_storage.clone();
        move |ev: PointerEvent| {
            event_handlers::on_pointer_down(ev, &canvas_ref, &ctx_store, &state);
        }
    };
    let handle_pointer_move = {
        let state = drawing_state.clone();
        let ctx_store = ctx_storage.clone();
        move |ev: PointerEvent| {
            event_handlers::on_pointer_move(ev, &canvas_ref, &ctx_store, &state);
        }
    };
    let handle_pointer_up = {
        let state = drawing_state.clone();
        let ctx_store = ctx_storage.clone();
        let completion = event_handlers::StrokeCompletionState {
            strokes,
            current_stroke_index,
            is_completed,
            on_complete,
        };
        move |_| {
            event_handlers::on_pointer_up(&state, &ctx_store, &completion);
        }
    };
    let handle_pointer_leave = handle_pointer_up.clone();

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let test_id_canvas = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(format!("{}-canvas", val))
        }
    };

    let test_id_progress = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(format!("{}-progress", val))
        }
    };

    view! {
        <div class="kanji-drawing-container" data-testid=test_id_val>
            <div class="kanji-drawing-info" data-testid=test_id_progress>
                {move || {
                    if load_error.get() {
                        view! {
                            <div class="kanji-drawing-error">
                                {t!(i18n, kanji_page.animation_unavailable)}
                            </div>
                        }
                        .into_any()
                    } else if is_completed.get() {
                        view! {
                            <div class="kanji-drawing-progress">
                                {t!(i18n, kanji_page.done)}
                            </div>
                        }
                        .into_any()
                    } else {
                        let total = strokes.get().len();
                        let current = current_stroke_index.get() + 1;
                        view! {
                            <div class="kanji-drawing-progress">
                                {i18n.get_keys().kanji_page().stroke_progress().inner().to_string()
                                    .replacen("{}", &current.to_string(), 1)
                                    .replacen("{}", &total.to_string(), 1)}
                            </div>
                        }
                        .into_any()
                    }
                }}
            </div>
            // The canvas is a direct child of the container: the old
            // wrapper div existed only as the positioning context for a
            // decorative ::after shadow that never rendered (see
            // .kanji-drawing-canvas in input.css).
            <canvas
                node_ref={canvas_ref}
                width={CANVAS_SIZE}
                height={CANVAS_SIZE}
                data-testid=test_id_canvas
                class=move || {
                    if is_completed.get() {
                        "kanji-drawing-canvas pointer-events-none"
                    } else {
                        "kanji-drawing-canvas"
                    }
                }
                on:pointerdown={handle_pointer_down}
                on:pointermove={handle_pointer_move}
                on:pointerup={handle_pointer_up}
                on:pointerleave={handle_pointer_leave}
            />
        </div>
    }
}
