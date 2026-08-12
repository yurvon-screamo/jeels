use leptos::html::Div;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

const VIEWPORT_MARGIN: f64 = 8.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TooltipPlacement {
    #[default]
    Top,
    Bottom,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TooltipPlacementMode {
    /// Automatically choose Top/Bottom based on available viewport space.
    #[default]
    Auto,
    /// Force Bottom (e.g. when the trigger is at the top of a list and Top
    /// would overlap the element above).
    ForceBottom,
}

/// Pure placement decision. Prefers Top when there is enough room above
/// the trigger (`trigger_top >= tooltip_height + margin`); otherwise
/// flips to Bottom when the viewport offers more room below the trigger
/// than above. Boundary is non-strict (>=) so the tooltip does not
/// thrash between sides at the exact threshold.
pub(crate) fn decide_placement(
    trigger_top: f64,
    viewport_height: f64,
    tooltip_height: f64,
    margin: f64,
) -> TooltipPlacement {
    let needed = tooltip_height + margin;
    let room_above = trigger_top;
    let room_below = (viewport_height - trigger_top).max(0.0);

    if room_above >= needed {
        TooltipPlacement::Top
    } else if room_below > room_above {
        TooltipPlacement::Bottom
    } else {
        TooltipPlacement::Top
    }
}

/// Calculate the horizontal shift (in px) needed so the tooltip body stays
/// within the viewport. Returns a positive value (shift right) when the
/// tooltip overflows the left edge, a negative value (shift left) when it
/// overflows the right edge, or 0 when it fits.
///
/// `trigger_left` / `trigger_right` — trigger element viewport coordinates.
/// `tooltip_width` — measured tooltip width.
/// `viewport_width` — inner width of the window.
/// `margin` — minimum pixel gap to keep between tooltip and viewport edge.
pub(crate) fn decide_shift_x(
    trigger_left: f64,
    trigger_right: f64,
    tooltip_width: f64,
    viewport_width: f64,
    margin: f64,
) -> f64 {
    // The tooltip is centered above/below the trigger by default
    // (left: 50%; transform: translateX(-50%)).
    let trigger_center = (trigger_left + trigger_right) / 2.0;
    let tooltip_half = tooltip_width / 2.0;

    let tooltip_left = trigger_center - tooltip_half;
    let tooltip_right = trigger_center + tooltip_half;

    if tooltip_left < margin {
        // Overflows left edge → shift right
        margin - tooltip_left
    } else if tooltip_right > viewport_width - margin {
        // Overflows right edge → shift left
        (viewport_width - margin) - tooltip_right
    } else {
        0.0
    }
}

#[component]
pub fn Tooltip(
    #[prop(optional, into)] text: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
    #[prop(optional)] placement_mode: TooltipPlacementMode,
    children: Children,
) -> impl IntoView {
    let placement = RwSignal::new(TooltipPlacement::default());
    let container_ref: NodeRef<Div> = NodeRef::new();
    let tooltip_ref: NodeRef<Div> = NodeRef::new();

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let placement_class = move || match placement.get() {
        TooltipPlacement::Top => "tooltip tooltip--top",
        TooltipPlacement::Bottom => "tooltip tooltip--bottom",
    };

    let on_enter = move |_: leptos::ev::PointerEvent| {
        let Some(container) = container_ref.get() else {
            return;
        };
        let trigger_rect = container.get_bounding_client_rect();
        let tooltip_el = tooltip_ref.get();

        let tooltip_height = tooltip_el
            .as_ref()
            .map(|el| el.get_bounding_client_rect().height())
            .unwrap_or(40.0);
        let tooltip_width = tooltip_el
            .as_ref()
            .map(|el| el.get_bounding_client_rect().width())
            .unwrap_or(120.0);

        let viewport_height = window()
            .inner_height()
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(800.0);
        let viewport_width = window()
            .inner_width()
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(400.0);

        // Y-axis: Top vs Bottom
        let resolved_placement = match placement_mode {
            TooltipPlacementMode::Auto => decide_placement(
                trigger_rect.top(),
                viewport_height,
                tooltip_height,
                VIEWPORT_MARGIN,
            ),
            TooltipPlacementMode::ForceBottom => TooltipPlacement::Bottom,
        };
        placement.set(resolved_placement);

        // X-axis: keep tooltip within viewport horizontally
        let shift_x = decide_shift_x(
            trigger_rect.left(),
            trigger_rect.right(),
            tooltip_width,
            viewport_width,
            VIEWPORT_MARGIN,
        );

        // Apply shift via CSS variable consumed by .tooltip / .tooltip::after
        if let Some(el) = &tooltip_el {
            let html_el: &web_sys::HtmlElement = el.unchecked_ref();
            let _ = html_el
                .style()
                .set_property("--tooltip-shift-x", &format!("{shift_x}px"));
        }
    };

    view! {
        <div
            class="tooltip-container"
            data-testid=test_id_val
            node_ref=container_ref
            on:pointerenter=on_enter
        >
            {children()}
            <div class=placement_class node_ref=tooltip_ref>
                {move || text.get()}
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decide_placement_near_top_returns_bottom() {
        assert_eq!(
            decide_placement(10.0, 800.0, 100.0, 8.0),
            TooltipPlacement::Bottom,
        );
    }

    #[test]
    fn decide_placement_mid_viewport_returns_top() {
        assert_eq!(
            decide_placement(400.0, 800.0, 100.0, 8.0),
            TooltipPlacement::Top,
        );
    }

    #[test]
    fn decide_placement_at_boundary_returns_top() {
        assert_eq!(
            decide_placement(108.0, 800.0, 100.0, 8.0),
            TooltipPlacement::Top,
        );
    }

    #[test]
    fn decide_placement_just_below_boundary_returns_bottom() {
        assert_eq!(
            decide_placement(107.0, 800.0, 100.0, 8.0),
            TooltipPlacement::Bottom,
        );
    }

    #[test]
    fn decide_placement_tight_viewport_picks_wider_side() {
        // Neither side fits the tooltip (room_above=10, room_below=50,
        // needed=108), so the function picks the side with more room — Bottom.
        assert_eq!(
            decide_placement(10.0, 60.0, 100.0, 8.0),
            TooltipPlacement::Bottom,
        );
    }

    // --- decide_shift_x tests ---

    #[test]
    fn shift_x_centered_trigger_returns_zero() {
        // Trigger centered in 400px viewport, tooltip 120px → fits easily
        assert_eq!(decide_shift_x(170.0, 230.0, 120.0, 400.0, 8.0), 0.0);
    }

    #[test]
    fn shift_x_trigger_near_left_edge_shifts_right() {
        // Trigger at left edge (left=0, right=40, center=20).
        // Tooltip half=60, tooltip_left = 20-60 = -40 → overflow.
        // shift = margin - tooltip_left = 8 - (-40) = 48
        assert_eq!(decide_shift_x(0.0, 40.0, 120.0, 400.0, 8.0), 48.0);
    }

    #[test]
    fn shift_x_trigger_near_right_edge_shifts_left() {
        // Trigger at right edge (left=360, right=400, center=380).
        // Tooltip half=60, tooltip_right = 380+60 = 440 → overflow.
        // shift = (400-8) - 440 = -48
        assert_eq!(decide_shift_x(360.0, 400.0, 120.0, 400.0, 8.0), -48.0);
    }

    #[test]
    fn shift_x_trigger_in_corner_shifts_correctly() {
        // Trigger in top-left corner: left=2, right=30, center=16
        // Tooltip 200px wide: tooltip_left = 16-100 = -84
        // shift = 8 - (-84) = 92
        assert_eq!(decide_shift_x(2.0, 30.0, 200.0, 400.0, 8.0), 92.0);
    }
}
