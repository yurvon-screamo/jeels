//! Body scroll lock for overlay components (`Drawer`, `UpdateDrawer`).
//!
//! Locking uses the `position: fixed` technique instead of plain
//! `overflow: hidden`: on iOS/WKWebView `overflow: hidden` on `<body>` does
//! not stop page scrolling, while a fixed body does. The scroll offset is
//! captured on the first lock and restored on the last unlock, and a
//! scrollbar-width compensation (measured BEFORE the body is fixed — a fixed
//! body stops overflowing and the measurement would read 0) prevents the
//! layout shift when the desktop scrollbar disappears.
//!
//! The refcount lets nested overlays share one lock. Unlocking is
//! saturating: an unbalanced unlock never underflows the counter into a
//! state where a future lock silently no-ops.

use std::sync::atomic::{AtomicUsize, Ordering};

static LOCK_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Increments the lock refcount; applies the body lock on the 0→1
/// transition only.
pub fn lock_scroll() {
    let prev = LOCK_COUNT.fetch_add(1, Ordering::SeqCst);
    if prev == 0 {
        apply_body_lock();
    }
}

/// Decrements the lock refcount saturating; removes the body lock on the
/// 1→0 transition only. Unlocking at zero is a no-op (unbalanced unlock).
pub fn unlock_scroll() {
    let current = LOCK_COUNT.load(Ordering::SeqCst);
    if current == 0 {
        return;
    }
    if current == 1 {
        let released = LOCK_COUNT
            .compare_exchange(1, 0, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok();
        if released {
            remove_body_lock();
        }
        return;
    }
    LOCK_COUNT.fetch_sub(1, Ordering::SeqCst);
}

#[cfg(target_arch = "wasm32")]
struct SavedScrollState {
    x: f64,
    y: f64,
    padding_right: String,
}

#[cfg(target_arch = "wasm32")]
static SAVED_STATE: std::sync::Mutex<Option<SavedScrollState>> = std::sync::Mutex::new(None);

#[cfg(target_arch = "wasm32")]
fn apply_body_lock() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let (Some(body), Some(html)) = (document.body(), document.document_element()) else {
        return;
    };

    // Scrollbar compensation must be measured before `position: fixed`:
    // once the body is fixed the document no longer overflows and the
    // measurement would always read 0.
    let scrollbar_width = window
        .inner_width()
        .ok()
        .and_then(|w| w.as_f64())
        .unwrap_or(0.0)
        - html.client_width() as f64;

    let saved = SavedScrollState {
        x: window.scroll_x().unwrap_or(0.0),
        y: window.scroll_y().unwrap_or(0.0),
        padding_right: body
            .style()
            .get_property_value("padding-right")
            .unwrap_or_default(),
    };
    let saved_y = saved.y;
    if let Ok(mut guard) = SAVED_STATE.lock() {
        *guard = Some(saved);
    }

    let style = body.style();
    let _ = style.set_property("position", "fixed");
    let _ = style.set_property("top", &format!("-{saved_y}px"));
    let _ = style.set_property("width", "100%");
    if scrollbar_width > 0.0 {
        let _ = style.set_property("padding-right", &format!("{scrollbar_width}px"));
    }
}

#[cfg(target_arch = "wasm32")]
fn remove_body_lock() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Some(body) = document.body() else {
        return;
    };

    let saved = SAVED_STATE.lock().ok().and_then(|mut guard| guard.take());

    let style = body.style();
    let _ = style.set_property("position", "");
    let _ = style.set_property("top", "");
    let _ = style.set_property("width", "");
    if let Some(saved) = &saved {
        let _ = style.set_property("padding-right", &saved.padding_right);
    }

    if let Some(saved) = saved {
        window.scroll_to_with_x_and_y(saved.x, saved.y);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn apply_body_lock() {}

#[cfg(not(target_arch = "wasm32"))]
fn remove_body_lock() {}

/// Test-only: resets the refcount and clears any body lock styles, so
/// assertions about the body are independent of leaked mounts from earlier
/// tests (`mount_to_wrapper` leaks its owner, keeping Effects alive).
#[cfg(test)]
pub fn reset_for_tests() {
    LOCK_COUNT.store(0, Ordering::SeqCst);
    #[cfg(target_arch = "wasm32")]
    {
        if let Ok(mut guard) = SAVED_STATE.lock() {
            *guard = None;
        }
        if let Some(window) = web_sys::window()
            && let Some(document) = window.document()
            && let Some(body) = document.body()
        {
            let style = body.style();
            let _ = style.set_property("position", "");
            let _ = style.set_property("top", "");
            let _ = style.set_property("width", "");
            let _ = style.set_property("padding-right", "");
        }
    }
}

#[cfg(test)]
pub fn lock_count_for_tests() -> usize {
    LOCK_COUNT.load(Ordering::SeqCst)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn lock_twice_unlock_once_stays_locked() {
        reset_for_tests();
        lock_scroll();
        lock_scroll();
        unlock_scroll();
        assert_eq!(lock_count_for_tests(), 1, "one lock must remain held");
        unlock_scroll();
        assert_eq!(lock_count_for_tests(), 0);
        reset_for_tests();
    }

    #[test]
    fn unlock_without_lock_is_ignored_without_underflow() {
        reset_for_tests();
        unlock_scroll();
        assert_eq!(
            lock_count_for_tests(),
            0,
            "unbalanced unlock must saturate at 0"
        );
        lock_scroll();
        assert_eq!(
            lock_count_for_tests(),
            1,
            "a lock after a saturated unlock must still apply"
        );
        unlock_scroll();
        assert_eq!(lock_count_for_tests(), 0);
        reset_for_tests();
    }
}
