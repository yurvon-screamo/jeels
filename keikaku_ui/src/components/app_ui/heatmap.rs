use chrono::{Datelike, Duration, NaiveDate, Utc};
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct HeatmapDataPoint {
    pub date: NaiveDate,
    pub minutes: u32,
}

impl HeatmapDataPoint {
    pub fn new(date: NaiveDate, minutes: u32) -> Self {
        Self { date, minutes }
    }
}

#[component]
pub fn Heatmap(data: Vec<HeatmapDataPoint>, class: Option<String>) -> Element {
    let class_str = class.unwrap_or_default();

    // Создаем карту дата -> минуты для быстрого доступа
    let mut data_map = std::collections::HashMap::new();
    for point in &data {
        data_map.insert(point.date, point.minutes);
    }

    // Получаем текущую дату
    let today = Utc::now().date_naive();

    let months = generate_months(today);

    rsx! {
        // Heatmap has fixed-width months; on narrow screens it must scroll horizontally.
        div { class: "max-w-full overflow-x-auto {class_str}",
            div { class: "flex flex-row gap-2 min-w-max",
                for month in months {
                    div { class: "flex flex-col gap-0 flex-shrink-0",
                        // Название месяца (очень маленький)
                        h3 { class: "text-xs font-medium text-text-secondary text-center leading-none mb-1",
                            {format_month_name(month.year, month.month).chars().take(3).collect::<String>()}
                        }
                        // Сетка дней в одну строку с фиксированной шириной
                        div { class: "grid grid-cols-7 gap-0 w-[112px]",
                            // Ячейки дней (включая пустые для начала месяца)
                            for _ in 0..month.first_weekday_offset {
                                div { class: "w-4 h-4" }
                            }
                            for day in 1..=month.days_in_month {
                                {render_day_cell_compact(month.year, month.month, day, &data_map)}
                            }
                        }
                    }
                }
            }
        }
    }
}

struct MonthInfo {
    year: i32,
    month: u32,
    days_in_month: u32,
    first_weekday_offset: u32,
}

fn generate_months(today: NaiveDate) -> Vec<MonthInfo> {
    let mut months = Vec::new();

    for month_offset in (0..5).rev() {
        let target_date = today - Duration::days(30 * month_offset as i64);
        let year = target_date.year();
        let month = target_date.month();

        let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let days_in_month = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
        }
        .signed_duration_since(first_day)
        .num_days() as u32;

        // Вычисляем смещение первого дня недели (0 = понедельник)
        let first_weekday = first_day.weekday().num_days_from_monday();

        months.push(MonthInfo {
            year,
            month,
            days_in_month,
            first_weekday_offset: first_weekday,
        });
    }

    months
}

fn format_month_name(year: i32, month: u32) -> String {
    let month_names = [
        "Январь",
        "Февраль",
        "Март",
        "Апрель",
        "Май",
        "Июнь",
        "Июль",
        "Август",
        "Сентябрь",
        "Октябрь",
        "Ноябрь",
        "Декабрь",
    ];

    format!("{} {}", month_names[month as usize - 1], year)
}

fn render_day_cell_compact(
    year: i32,
    month: u32,
    day: u32,
    data_map: &std::collections::HashMap<NaiveDate, u32>,
) -> Element {
    let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
    let today = Utc::now().date_naive();
    let minutes = data_map.get(&date).copied().unwrap_or(0);

    let (bg_class, bg_style) = if date > today {
        // Будущие дни - светло-серый цвет
        ("", "background-color: #f3f4f6;".to_string())
    } else {
        let minutes = data_map.get(&date).copied().unwrap_or(0);
        calculate_background_style(minutes)
    };

    rsx! {
        div {
            class: "w-4 h-4 rounded-full {bg_class} cursor-pointer hover:ring-1 hover:ring-text-main/30 transition-all",
            style: bg_style,
            title: format!("{}: {} мин", date.format("%d.%m.%Y"), minutes),
        }
    }
}

fn calculate_background_style(minutes: u32) -> (&'static str, String) {
    // Константы времени
    const MIN_MINUTES: f32 = 10.0; // менее 10 минут - красный
    const MAX_MINUTES: f32 = 180.0; // 3 часа - идеально

    let ratio = if minutes <= MIN_MINUTES as u32 {
        0.0
    } else if minutes >= MAX_MINUTES as u32 {
        1.0
    } else {
        // Используем квадратный корень для более быстрого начального перехода
        let linear_ratio = (minutes as f32 - MIN_MINUTES) / (MAX_MINUTES - MIN_MINUTES);
        linear_ratio.sqrt().min(1.0)
    };

    if ratio <= 0.0 {
        // Красный для значений <= 10 минут
        ("", "background-color: #f0888f;".to_string())
    } else if ratio >= 1.0 {
        // Золото-бензиновый градиент для значений >= 3 часов
        (
            "",
            "background-image: linear-gradient(135deg, #FDE68A 0%, #F472B6 50%, #22D3EE 100%);"
                .to_string(),
        )
    } else {
        // Промежуточные значения - линейная интерполяция
        let color = interpolate_color(ratio);
        ("", format!("background-color: {};", color))
    }
}

fn interpolate_color(ratio: f32) -> String {
    // Цвета для интерполяции
    // Красный: #f0888f -> RGB(240, 136, 143)
    let red_rgb = (240, 136, 143);
    // Зеленый:rgb(52, 255, 150) -> RGB(209, 250, 229) 10b981
    let green_rgb = (52, 255, 150);
    // Золото: #FDE68A -> RGB(253, 230, 138)
    let gold_rgb = (253, 230, 138);

    let (start_rgb, end_rgb, adjusted_ratio) = if ratio <= 0.5 {
        // От красного к зеленому (0.0 - 0.5)
        (red_rgb, green_rgb, ratio * 2.0)
    } else {
        // От зеленого к золотому (0.5 - 1.0)
        (green_rgb, gold_rgb, (ratio - 0.5) * 2.0)
    };

    let r = start_rgb.0 as f32 + (end_rgb.0 as f32 - start_rgb.0 as f32) * adjusted_ratio;
    let g = start_rgb.1 as f32 + (end_rgb.1 as f32 - start_rgb.1 as f32) * adjusted_ratio;
    let b = start_rgb.2 as f32 + (end_rgb.2 as f32 - start_rgb.2 as f32) * adjusted_ratio;

    format!("#{:02x}{:02x}{:02x}", r as u8, g as u8, b as u8)
}
