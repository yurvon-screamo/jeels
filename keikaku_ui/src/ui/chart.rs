use dioxus::prelude::*;

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
    let color = color.unwrap_or("#22D3EE".to_string()); // cyan-400 из палитры Uwuwu
    let height = height.unwrap_or(300.0);
    let class_str = class.unwrap_or_else(|| "".to_string());

    if data.is_empty() {
        return rsx! {
            Card {
                delay: delay.clone(),
                class: Some(format!("{} {}", class_str, "")),
                div { class: "flex flex-col items-center justify-center h-48 text-center relative z-10",
                    div { class: "w-16 h-16 rounded-2xl bg-gradient-to-br from-pink-100 to-purple-100 flex items-center justify-center mb-4 shadow-md",
                        svg {
                            class: "w-8 h-8 text-accent-pink",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z",
                            }
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

    // Добавляем немного padding сверху и снизу
    let padding = (max_value - min_value) * 0.1;
    let chart_max = max_value + padding;
    let chart_min = if min_value - padding > 0.0 {
        min_value - padding
    } else {
        0.0
    };

    // Используем фиксированный viewBox для масштабирования
    let chart_width = 400.0;
    let chart_height = height - 30.0; // График занимает основную высоту

    // Создаем точки для линии
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

    let path_data = if points.len() > 1 {
        format!("M {}", points.join(" L "))
    } else if points.len() == 1 {
        format!(
            "M {} {} L {} {}",
            points[0], chart_height, points[0], points[0]
        )
    } else {
        "".to_string()
    };

    // Вычисляем точки заранее
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
            let short_label = if point.label.len() > 10 {
                format!("{}...", &point.label[..7])
            } else {
                point.label.clone()
            };
            (x, y, short_label)
        })
        .collect();

    rsx! {
        Card {
            delay: delay.clone(),
            class: Some(format!("{} {}", class_str, "")),
            div { class: "relative z-10",
                // Декоративные элементы
                div { class: "absolute top-0 right-0 w-24 h-24 bg-accent-cyan/10 rounded-full blur-xl -translate-y-1/2 translate-x-1/2" }
                div { class: "absolute bottom-0 left-0 w-20 h-20 bg-accent-purple/10 rounded-full blur-lg translate-y-1/2 -translate-x-1/2" }

                div { class: "chart-container relative z-10 w-full overflow-visible",
                    div { class: "flex items-center gap-3 mb-4",
                        div { class: "w-8 h-8 rounded-xl bg-gradient-to-br from-cyan-100 to-purple-100 flex items-center justify-center shadow-md",
                            svg {
                                class: "w-4 h-4 text-accent-cyan",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z",
                                }
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

                        // Сетка с градиентным фоном
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

                        // Линия графика с градиентом
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

                        // Точки данных с hover эффектами
                        for (x , y , _) in chart_points.iter() {
                            circle {
                                cx: "{x}",
                                cy: "{y}",
                                r: "5",
                                fill: "{color}",
                                stroke: "white",
                                stroke_width: "3",
                                class: "transition-all duration-300 hover:r-7 hover:drop-shadow-lg cursor-pointer",
                            }
                        }

                        // Ось X с метками
                        for (i , (x , _ , label)) in chart_points.iter().enumerate() {
                            if i % (data.len() / 5 + 1).max(1) == 0 || i == data.len() - 1 {
                                text {
                                    x: "{x}",
                                    y: "{height}",
                                    text_anchor: "middle",
                                    class: "text-xs fill-slate-500 font-medium",
                                    transform: "rotate(-45, {x}, {height})",
                                    {label.clone()}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
