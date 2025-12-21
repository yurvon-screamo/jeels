use dioxus::prelude::*;
use dioxus_heroicons::{Icon, solid};

use super::Card;

#[derive(Clone, PartialEq)]
pub struct ChartDataPoint {
    pub label: String,
    pub value: f64,
}

#[component]
pub fn Chart(
    title: String,
    data: Vec<ChartDataPoint>,
    color: Option<String>,
    height: Option<f64>,
    delay: Option<String>,
    class: Option<String>,
) -> Element {
    let color = color.unwrap_or("#22D3EE".to_string()); // cyan-400
    let height = height.unwrap_or(300.0);
    let class_str = class.unwrap_or_else(|| "".to_string());

    // Tooltip state
    let tooltip_visible = use_signal(|| false);
    let tooltip_text = use_signal(String::new);
    let tooltip_x = use_signal(|| 0.0);
    let tooltip_y = use_signal(|| 0.0);

    let mut tooltip_visible_clone = tooltip_visible;
    let mut tooltip_text_clone = tooltip_text;
    let mut tooltip_x_clone = tooltip_x;
    let mut tooltip_y_clone = tooltip_y;

    if data.is_empty() {
        return rsx! {
            Card {
                delay: delay.clone(),
                class: Some(format!("{} {}", class_str, "")),
                div { class: "flex flex-col items-center justify-center h-48 text-center relative z-10",
                    div { class: "w-16 h-16 rounded-2xl bg-gradient-to-br from-pink-100 to-purple-100 flex items-center justify-center mb-4 shadow-md",
                        Icon {
                            icon: solid::Shape::ChartBar,
                            size: 32,
                            class: Some("w-8 h-8 text-accent-pink".to_string()),
                        }
                    }
                    div { class: "text-xs font-bold text-accent-purple uppercase tracking-widest mb-2",
                        "Нет данных"
                    }
                    p { class: "text-sm font-medium text-slate-600 leading-relaxed",
                        "Данные для графика отсутствуют"
                    }
                }
            }
        };
    }

    let max_value = data.iter().map(|p| p.value).fold(0.0, f64::max);
    let min_value = data.iter().map(|p| p.value).fold(f64::INFINITY, f64::min);

    // Add padding
    let padding = (max_value - min_value) * 0.1;
    let chart_max = max_value + padding;
    let chart_min = if min_value - padding > 0.0 {
        min_value - padding
    } else {
        0.0
    };

    let chart_width = 400.0;
    let chart_height = height - 40.0;

    let points: Vec<String> = data
        .iter()
        .enumerate()
        .map(|(i, point)| {
            let x = if data.len() > 1 {
                (i as f64 / (data.len() - 1) as f64) * chart_width
            } else {
                chart_width / 2.0
            };
            let y = if chart_max == chart_min {
                chart_height / 2.0
            } else {
                chart_height - ((point.value - chart_min) / (chart_max - chart_min)) * chart_height
            };
            format!("{},{}", x, y)
        })
        .collect();

    let path_data = if !points.is_empty() {
        format!("M {}", points.join(" L "))
    } else {
        "".to_string()
    };

    let chart_points: Vec<(f64, f64, String)> = data
        .iter()
        .enumerate()
        .map(|(i, point)| {
            let x = (i as f64 / (data.len().max(1) - 1).max(0) as f64) * chart_width;
            let y = if chart_max == chart_min {
                chart_height / 2.0
            } else {
                chart_height - ((point.value - chart_min) / (chart_max - chart_min)) * chart_height
            };
            let short_label = if point.label.len() > 12 {
                format!("{}...", &point.label[..12])
            } else {
                point.label.clone()
            };
            (x, y, short_label)
        })
        .collect();

    let y_labels: Vec<(f64, String)> = (0..5)
        .map(|i| {
            let value = chart_min + (chart_max - chart_min) * (i as f64 / 4.0);
            let y = if chart_max == chart_min {
                chart_height / 2.0
            } else {
                chart_height - ((value - chart_min) / (chart_max - chart_min)) * chart_height
            };
            (y, format!("{:.0}", value))
        })
        .collect();

    rsx! {
        Card {
            delay: delay.clone(),
            class: Some(format!("{} {}", class_str, "")),
            div { class: "relative z-10",
                div { class: "absolute top-0 right-0 w-24 h-24 bg-accent-cyan/10 rounded-full blur-xl -translate-y-1/2 translate-x-1/2" }
                div { class: "absolute bottom-0 left-0 w-20 h-20 bg-accent-purple/10 rounded-full blur-lg translate-y-1/2 -translate-x-1/2" }

                div { class: "chart-container relative z-10 w-full overflow-visible",
                    div { class: "flex items-center gap-3 mb-4",
                        div { class: "w-8 h-8 rounded-xl bg-gradient-to-br from-cyan-100 to-purple-100 flex items-center justify-center shadow-md",
                            Icon {
                                icon: solid::Shape::ChartBar,
                                size: 16,
                                class: Some("w-4 h-4 text-accent-cyan".to_string()),
                            }
                        }
                        h3 { class: "text-sm font-bold text-slate-700", {title} }
                    }
                    svg {
                        width: "100%",
                        height: "{height + 40.0}",
                        view_box: "0 0 {chart_width} {height + 40.0}",
                        preserve_aspect_ratio: "xMidYMid meet",
                        class: "chart-svg w-full",
                        defs {
                            linearGradient {
                                id: "chart-bg",
                                x1: "0%",
                                y1: "0%",
                                x2: "100%",
                                y2: "100%",
                                stop { offset: "0%", stop_color: "#fef7ff" }
                                stop { offset: "50%", stop_color: "#f0f9ff" }
                                stop { offset: "100%", stop_color: "#fef7ff" }
                            }
                            pattern {
                                id: "grid",
                                width: "20",
                                height: "20",
                                pattern_units: "userSpaceOnUse",
                                rect {
                                    width: "100%",
                                    height: "100%",
                                    fill: "url(#chart-bg)",
                                }
                                path {
                                    d: "M 20 0 L 0 0 0 20",
                                    fill: "none",
                                    stroke: "#e2e8f0",
                                    stroke_width: "0.5",
                                }
                            }
                        }
                        rect {
                            width: "100%",
                            height: "{chart_height}",
                            fill: "url(#grid)",
                            rx: "8",
                        }

                        if !path_data.is_empty() {
                            defs {
                                linearGradient {
                                    id: "line-gradient",
                                    x1: "0%",
                                    y1: "0%",
                                    x2: "100%",
                                    y2: "0%",
                                    stop {
                                        offset: "0%",
                                        stop_color: "{color}",
                                        stop_opacity: "0.8",
                                    }
                                    stop {
                                        offset: "50%",
                                        stop_color: "{color}",
                                        stop_opacity: "1",
                                    }
                                    stop {
                                        offset: "100%",
                                        stop_color: "{color}",
                                        stop_opacity: "0.8",
                                    }
                                }
                            }
                            path {
                                d: "{path_data}",
                                fill: "none",
                                stroke: "url(#line-gradient)",
                                stroke_width: "3",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                class: "drop-shadow-sm",
                            }
                        }

                        for (y_pos , _) in y_labels.iter() {
                            line {
                                x1: "0",
                                y1: "{y_pos}",
                                x2: "{chart_width}",
                                y2: "{y_pos}",
                                stroke: "#e2e8f0",
                                stroke_width: "0.5",
                                stroke_dasharray: "2,2",
                            }
                        }

                        for (y_pos , label) in y_labels.iter() {
                            text {
                                x: "-10",
                                y: "{y_pos}",
                                text_anchor: "end",
                                class: "text-xs fill-slate-500 font-medium",
                                {label.clone()}
                            }
                        }

                        {
                            chart_points
                                .iter()
                                .enumerate()
                                .map(|(i, (x, y, label))| {
                                    let value = data[i].value;
                                    let label_clone = label.clone();
                                    let x_clone = *x;
                                    let y_clone = *y;
                                    rsx! {
                                        circle {
                                            cx: "{x}",
                                            cy: "{y}",
                                            r: "5",
                                            fill: "{color}",
                                            stroke: "white",
                                            stroke_width: "3",
                                            class: "transition-all duration-300 hover:r-7 hover:drop-shadow-lg cursor-pointer",
                                            onmouseover: move |_| {
                                                tooltip_visible_clone.set(true);
                                                tooltip_text_clone.set(format!("{}: {:.1}", label_clone, value));
                                                tooltip_x_clone.set(x_clone);
                                                tooltip_y_clone.set(y_clone);
                                            },
                                            onmouseout: move |_| {
                                                tooltip_visible_clone.set(false);
                                            },
                                        }
                                    }
                                })
                        }

                        for (i , (x , _ , label)) in chart_points.iter().enumerate() {
                            if i % (data.len() / 5 + 1).max(1) == 0 || i == data.len() - 1 {
                                text {
                                    x: "{x}",
                                    y: "{height}",
                                    text_anchor: "middle",
                                    class: "text-xs fill-slate-500 font-medium",
                                    transform: "rotate(-25, {x}, {height})",
                                    {label.clone()}
                                }
                            }
                        }

                        if *tooltip_visible.read() {
                            g {
                                rect {
                                    x: "{tooltip_x() - 45.0}",
                                    y: "{tooltip_y() - 25.0}",
                                    width: "140",
                                    height: "20",
                                    fill: "#1f2937",
                                    rx: "4",
                                    opacity: "0.9",
                                }
                                text {
                                    x: "{tooltip_x() + 25.0}",
                                    y: "{tooltip_y() - 10.0}",
                                    text_anchor: "middle",
                                    class: "text-xs fill-white font-medium",
                                    {tooltip_text()}
                                }
                            }
                        }

                        if let Some((x, y, _)) = chart_points.last() {
                            g {
                                rect {
                                    x: "{x + 10.0}",
                                    y: "{y - 10.0}",
                                    width: "45",
                                    height: "20",
                                    fill: "{color}",
                                    rx: "4",
                                    opacity: "0.9",
                                }
                                text {
                                    x: "{x + 32.5}",
                                    y: "{y + 5.0}",
                                    text_anchor: "middle",
                                    class: "text-xs fill-white font-bold",
                                    {format!("{:.1}", data.last().unwrap().value)}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
