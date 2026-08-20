use origa::domain::JapaneseLevel;

#[derive(Clone)]
pub enum AppType {
    DuolingoRu,
    DuolingoEn,
    Migii,
    MinnaNoNihongo,
    Irodori,
}

pub fn parse_app_type(app_id: &str) -> Option<AppType> {
    match app_id {
        "DuolingoRu" => Some(AppType::DuolingoRu),
        "DuolingoEn" => Some(AppType::DuolingoEn),
        "Migii" => Some(AppType::Migii),
        "MinnaNoNihongo" => Some(AppType::MinnaNoNihongo),
        "Irodori" => Some(AppType::Irodori),
        _ => None,
    }
}

pub fn level_to_str(level: JapaneseLevel) -> &'static str {
    match level {
        JapaneseLevel::N5 => "N5",
        JapaneseLevel::N4 => "N4",
        JapaneseLevel::N3 => "N3",
        JapaneseLevel::N2 => "N2",
        JapaneseLevel::N1 => "N1",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_app_type_maps_every_known_app_id() {
        assert!(parse_app_type("DuolingoRu").is_some());
        assert!(parse_app_type("DuolingoEn").is_some());
        assert!(parse_app_type("Migii").is_some());
        assert!(parse_app_type("MinnaNoNihongo").is_some());
        assert!(parse_app_type("Irodori").is_some());
    }

    #[test]
    fn parse_app_type_unknown_id_is_none() {
        assert!(parse_app_type("Anki").is_none());
        assert!(parse_app_type("").is_none());
    }

    #[test]
    fn level_to_str_matches_jlpt_codes() {
        assert_eq!(level_to_str(JapaneseLevel::N5), "N5");
        assert_eq!(level_to_str(JapaneseLevel::N3), "N3");
        assert_eq!(level_to_str(JapaneseLevel::N1), "N1");
    }
}
