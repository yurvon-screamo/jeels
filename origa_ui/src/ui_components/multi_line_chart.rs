use leptos::prelude::*;

const PADDING_LEFT: u32 = 64;
const PADDING_RIGHT: u32 = 40;
const PADDING_BOTTOM: u32 = 48;
const PADDING_TOP: u32 = 24;

#[derive(Clone)]
pub struct ChartLine {
    pub data: Vec<(String, f64)>,
    pub color: String,
    pub label: String,
}

struct PolylineData {
    color: String,
    key: String,
    points_str: String,
}

struct CircleData {
    color: String,
    x: f64,
    y: f64,
    key: (i64, i64),
}

fn compute_global_bounds(lines: &[ChartLine]) -> (f64, f64) {
    let mut min_val = f64::INFINITY;
    let mut max_val = f64::NEG_INFINITY;

    for line in lines {
        for (_, v) in &line.data {
            min_val = min_val.min(*v);
            max_val = max_val.max(*v);
        }
    }

    if min_val == f64::INFINITY {
        min_val = 0.0;
        max_val = 1.0;
    }

    (min_val, max_val)
}

fn format_axis_value(v: f64) -> String {
    if v >= 1000.0 {
        format!("{:.1}k", v / 1000.0)
    } else {
        format!("{:.0}", v)
    }
}

#[component]
pub fn MultiLineChart(
    #[prop(into)] lines: Signal<Vec<ChartLine>>,
    #[prop(default = 400)] width: u32,
    #[prop(default = 200)] height: u32,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
    #[prop(optional, into)] empty_text: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let chart_width = width - PADDING_LEFT - PADDING_RIGHT;
    let chart_height = height - PADDING_TOP - PADDING_BOTTOM;

    let y_ticks = move || {
        let (min_val, max_val) = compute_global_bounds(&lines.get());

        if (max_val - min_val).abs() < 0.001 {
            let y = PADDING_TOP as f64 + chart_height as f64 / 2.0;
            return vec![(y, min_val)];
        }

        let tick_count = if chart_height < 100 {
            2
        } else if chart_height < 160 {
            3
        } else {
            4
        };

        (0..=tick_count)
            .map(|i| {
                let ratio = i as f64 / tick_count as f64;
                let y = PADDING_TOP as f64 + chart_height as f64 * (1.0 - ratio);
                let value = min_val + (max_val - min_val) * ratio;
                (y, value)
            })
            .collect::<Vec<_>>()
    };

    let x_label_positions = move || {
        let all_lines = lines.get();
        let first_line = match all_lines.first() {
            Some(l) => l,
            None => return Vec::new(),
        };

        let count = first_line.data.len();
        if count == 0 {
            return Vec::new();
        }

        if count == 1 {
            return vec![(
                PADDING_LEFT as f64 + chart_width as f64 / 2.0,
                first_line.data[0].0.clone(),
            )];
        }

        let step_x = chart_width as f64 / (count - 1) as f64;
        // Limit the number of X-axis labels to prevent overlap on narrow
        // viewports. On mobile (chart_width ~296px) even 6 short labels can
        // collide, so scale down the cap for smaller chart widths.
        let max_labels = if chart_width < 320 {
            4
        } else if count <= 10 {
            count
        } else {
            6
        };
        let label_step = (count as f64 / max_labels as f64).ceil() as usize;

        first_line
            .data
            .iter()
            .enumerate()
            .filter(|(i, _)| *i == 0 || *i == count - 1 || i % label_step == 0)
            .map(|(i, (label, _))| {
                let x = PADDING_LEFT as f64 + i as f64 * step_x;
                (x, label.clone())
            })
            .collect::<Vec<_>>()
    };

    let polyline_data = move || {
        let all_lines = lines.get();
        let (min_val, max_val) = compute_global_bounds(&all_lines);
        let range = (max_val - min_val).max(1.0);

        all_lines
            .into_iter()
            .enumerate()
            .filter_map(|(line_idx, line)| {
                let count = line.data.len();
                if count == 0 {
                    return None;
                }

                let step_x = if count == 1 {
                    0.0
                } else {
                    chart_width as f64 / (count - 1) as f64
                };

                let points: Vec<(f64, f64)> = line
                    .data
                    .iter()
                    .enumerate()
                    .map(|(i, (_, value))| {
                        let x = if count == 1 {
                            PADDING_LEFT as f64 + chart_width as f64 / 2.0
                        } else {
                            PADDING_LEFT as f64 + i as f64 * step_x
                        };
                        let y = PADDING_TOP as f64 + chart_height as f64
                            - ((value - min_val) / range) * chart_height as f64;
                        (x, y)
                    })
                    .collect();

                let points_str = points
                    .iter()
                    .map(|(x, y)| format!("{:.1},{:.1}", x, y))
                    .collect::<Vec<_>>()
                    .join(" ");

                Some(PolylineData {
                    color: line.color,
                    key: format!("line-{}", line_idx),
                    points_str,
                })
            })
            .collect::<Vec<_>>()
    };

    let circle_data = move || {
        let all_lines = lines.get();
        let (min_val, max_val) = compute_global_bounds(&all_lines);
        let range = (max_val - min_val).max(1.0);

        let mut result = Vec::new();

        for (line_idx, line) in all_lines.into_iter().enumerate() {
            if line.data.len() > 15 || line.data.is_empty() {
                continue;
            }

            let count = line.data.len();
            let step_x = if count == 1 {
                0.0
            } else {
                chart_width as f64 / (count - 1) as f64
            };

            for (i, (_, value)) in line.data.iter().enumerate() {
                let x = if count == 1 {
                    PADDING_LEFT as f64 + chart_width as f64 / 2.0
                } else {
                    PADDING_LEFT as f64 + i as f64 * step_x
                };
                let y = PADDING_TOP as f64 + chart_height as f64
                    - ((value - min_val) / range) * chart_height as f64;

                result.push(CircleData {
                    color: line.color.clone(),
                    x,
                    y,
                    key: (
                        ((line_idx as f64 * 10000.0) + i as f64 * 100.0 + x) as i64,
                        (y * 1000.0) as i64,
                    ),
                });
            }
        }

        result
    };

    let legend_items = move || {
        lines
            .get()
            .into_iter()
            .enumerate()
            .map(|(i, line)| (format!("legend-{}", i), line.color, line.label))
            .collect::<Vec<_>>()
    };

    let class_str = move || format!("{} chart-container w-full", class.get());
    let has_data = move || lines.get().iter().any(|l| !l.data.is_empty());

    view! {
        <div data-testid=test_id_val>
            <Show when=move || !has_data()>
                <div class="flex items-center justify-center" style="width:100%">
                    <svg viewBox=format!("0 0 {} {}", width, height) class=class_str preserveAspectRatio="xMidYMid meet" style="width:100%;height:auto;">
                        <text
                            x=format!("{}", width / 2)
                            y=format!("{}", height / 2)
                            text_anchor="middle"
                            dominant_baseline="central"
                            fill="var(--fg-muted)"
                            font_size="16"
                            font_family="IBM Plex Mono"
                        >
                            {move || empty_text.get()}
                        </text>
                    </svg>
                </div>
            </Show>

            <Show when=move || has_data()>
                <svg viewBox=format!("0 0 {} {}", width, height) class=class_str preserveAspectRatio="xMidYMid meet" style="width:100%;height:auto;">
                    // Y-axis
                    <line
                        x1=PADDING_LEFT
                        y1=PADDING_TOP
                        x2=PADDING_LEFT
                        y2=height - PADDING_BOTTOM
                        stroke="var(--border-light)"
                        stroke-width="1"
                    />
                    // X-axis
                    <line
                        x1=PADDING_LEFT
                        y1=height - PADDING_BOTTOM
                        x2=width - PADDING_RIGHT
                        y2=height - PADDING_BOTTOM
                        stroke="var(--border-light)"
                        stroke-width="1"
                    />

                    // Y-axis labels
                    <For
                        each=move || y_ticks()
                        key=|(_, v)| v.to_bits()
                        children=move |(y, value)| {
                            view! {
                                <text
                                    x=PADDING_LEFT - 44
                                    y=y
                                    text_anchor="end"
                                    dominant_baseline="central"
                                    fill="var(--fg-muted)"
                                    font_size="10"
                                    font_family="IBM Plex Mono"
                                >
                                    {format_axis_value(value)}
                                </text>
                            }
                        }
                    />

                    // X-axis labels
                    <For
                        each=move || x_label_positions()
                        key=|(x, label)| format!("{}-{}", x, label)
                        children=move |(x, label)| {
                            view! {
                                <text
                                    x=x
                                    y=height - PADDING_BOTTOM + 20
                                    text_anchor="middle"
                                    fill="var(--fg-muted)"
                                    font_size="10"
                                    font_family="IBM Plex Mono"
                                >
                                    {label}
                                </text>
                            }
                        }
                    />

                    // Polylines
                    <For
                        each=move || polyline_data()
                        key=|p| p.key.clone()
                        children=move |data| {
                            view! {
                                <polyline
                                    points=data.points_str
                                    fill="none"
                                    stroke=data.color
                                    stroke-width="2"
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                />
                            }
                        }
                    />

                    // Data point circles (only when <=15 points)
                    <For
                        each=move || circle_data()
                        key=|c| c.key
                        children=move |data| {
                            view! {
                                <circle
                                    cx=data.x
                                    cy=data.y
                                    r=4
                                    fill=data.color
                                    stroke="var(--bg-paper)"
                                    stroke-width="2"
                                />
                            }
                        }
                    />
                </svg>
            </Show>

            // Legend below SVG
            <div class="flex justify-center gap-2 sm:gap-5 mt-4 flex-wrap">
                <For
                    each=move || legend_items()
                    key=|(key, _, _)| key.clone()
                    children=move |(_, color, label)| {
                        view! {
                            <div class="flex items-center gap-1.5">
                                <span
                                    class="inline-block"
                                    style=format!("width:12px;height:2px;background-color:{}", color)
                                ></span>
                                <span class="font-mono text-[11px] sm:text-[12px] uppercase tracking-[0.08em] sm:tracking-[0.1em]" style="color:var(--fg-muted)">
                                    {label}
                                </span>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}
