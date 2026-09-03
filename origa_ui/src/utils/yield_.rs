#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

/// Yields execution to the browser, allowing it to process UI events
/// and render pending changes. This prevents UI freezing during CPU-bound operations.
///
/// In WASM, async functions run on a single thread (cooperative multitasking).
/// Without explicit yields, CPU-bound operations block the main thread,
/// preventing the browser from rendering UI updates.
///
/// Native builds (loader tests) resolve immediately: there is no event loop
/// to yield to outside the browser.
#[cfg(target_arch = "wasm32")]
pub async fn yield_to_browser() {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        if let Some(window) = web_sys::window() {
            // requestAnimationFrame schedules callback before next repaint
            let _ = window.request_animation_frame(&resolve);
        } else {
            // Fallback: resolve immediately if window unavailable
            resolve.call(&JsValue::NULL, ()).ok();
        }
    });

    JsFuture::from(promise).await.ok();
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn yield_to_browser() {}
