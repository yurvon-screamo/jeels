//! Time-budgeted slicing for the import-preview classification loop.
//!
//! Classifying a word costs a tokenizer pass in WASM on the main thread;
//! a fixed chunk size cannot guarantee a frame budget. Each slice runs
//! until `SLICE_BUDGET_MS` of (injectable) clock time has elapsed, always
//! processing at least one word so a slow word cannot livelock the loop.

/// Milliseconds of classification work per slice before yielding to the
/// browser. Kept well under a 60fps frame (16.6 ms).
pub(crate) const SLICE_BUDGET_MS: f64 = 8.0;

/// Returns the exclusive end index of the slice starting at `start`:
/// at least `start + 1` (capped at `len`) and then extended while the
/// elapsed `now() - slice_start` time stays under [`SLICE_BUDGET_MS`].
/// The first `now()` call captures the slice start, so a fake clock must
/// return the start time first and later readings after it.
pub(crate) fn slice_end(len: usize, start: usize, mut now: impl FnMut() -> f64) -> usize {
    let slice_start = now();
    let mut end = (start + 1).min(len);
    while end < len && now() - slice_start < SLICE_BUDGET_MS {
        end += 1;
    }
    end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice_end_frozen_clock_consumes_everything() {
        // now() always returns the slice-start time → elapsed is 0 forever.
        let end = slice_end(500, 0, || 42.0);
        assert_eq!(end, 500);
    }

    #[test]
    fn slice_end_clock_over_budget_processes_exactly_one_word() {
        let mut calls = 0;
        let end = slice_end(10, 3, move || {
            calls += 1;
            if calls == 1 { 0.0 } else { 100.0 }
        });
        assert_eq!(end, 4, "budget exceeded after the first word");
    }

    #[test]
    fn slice_end_growing_clock_stops_when_budget_is_reached() {
        // 0, 3, 6, 9, …: elapsed hits 9 ≥ 8.0 after two extensions.
        let mut tick = -3.0;
        let end = slice_end(10, 2, move || {
            tick += 3.0;
            tick
        });
        assert_eq!(end, 5, "3ms/word budget of 8ms → start + 3 words");
    }

    #[test]
    fn slice_end_starting_at_length_returns_length() {
        let end = slice_end(5, 5, || 0.0);
        assert_eq!(end, 5);
    }
}
