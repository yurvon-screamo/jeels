/// Преобразует глагол в て-форму
/// Правила для групп глаголов:
/// - Группа II (годан): зависит от последнего слога
/// - Группа I (итидан): る → て
/// - Группа III (неправильные): する → して, くる/来る → きて
pub fn to_te_form(word: &str) -> String {
    // Проверяем на неправильные глаголы
    if word == "する" {
        return "して".to_string();
    }
    if word == "くる" || word == "来る" {
        return "きて".to_string();
    }

    // Определяем группу по последнему символу
    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return word.to_string();
    }

    let last_char = chars[chars.len() - 1];

    // Группа I (итидан) - оканчиваются на る, но не входят в группу II
    if last_char == 'る' {
        // Проверяем, не является ли это глаголом группы II
        // Глаголы группы II не оканчиваются наiru, ari, ori, uri
        if chars.len() >= 2 {
            let second_last = chars[chars.len() - 2];
            if !matches!(second_last, 'い' | 'え' | 'お' | 'う') {
                // Группа I - меняем る на て
                let mut result = word.to_string();
                result.pop(); // убираем る
                result.push('て');
                return result;
            }
        }
    }

    // Группа II (годан) - правила по последнему слогу
    match last_char {
        'く' => {
            let mut result = word.to_string();
            result.pop();
            result.push_str("いて");
            result
        }
        'ぐ' => {
            let mut result = word.to_string();
            result.pop();
            result.push_str("いで");
            result
        }
        'す' => {
            let mut result = word.to_string();
            result.pop();
            result.push_str("して");
            result
        }
        'つ' | 'る' | 'う' => {
            let mut result = word.to_string();
            result.pop();
            result.push_str("って");
            result
        }
        'ぬ' | 'ぶ' | 'む' => {
            let mut result = word.to_string();
            result.pop();
            result.push_str("んで");
            result
        }
        _ => word.to_string(), // на случай неизвестной формы
    }
}

/// Преобразует глагол в ない-форму
/// Для этого нужно получить основу глагола и добавить ない
pub fn to_nai_form(word: &str) -> String {
    // Проверяем на неправильные глаголы
    if word == "する" {
        return "しない".to_string();
    }
    if word == "くる" || word == "来る" {
        return "こない".to_string();
    }

    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return word.to_string();
    }

    let last_char = chars[chars.len() - 1];

    // Группа I (итидан) - оканчиваются на る
    if last_char == 'る' && chars.len() >= 2 {
        let second_last = chars[chars.len() - 2];
        if !matches!(second_last, 'い' | 'え' | 'お' | 'う') {
            // Группа I - убираем る и добавляем ない
            let mut result = word.to_string();
            result.pop();
            result.push_str("ない");
            return result;
        }
    }

    // Группа II (годан) - меняем последний слог
    match last_char {
        'う' => {
            let mut result = word.to_string();
            result.pop();
            result.push_str("わない");
            result
        }
        'く' => {
            let mut result = word.to_string();
            result.pop();
            result.push_str("かない");
            result
        }
        'ぐ' => {
            let mut result = word.to_string();
            result.pop();
            result.push_str("がない");
            result
        }
        'す' => {
            let mut result = word.to_string();
            result.pop();
            result.push_str("さない");
            result
        }
        'つ' => {
            let mut result = word.to_string();
            result.pop();
            result.push_str("たない");
            result
        }
        'ぬ' => {
            let mut result = word.to_string();
            result.pop();
            result.push_str("なない");
            result
        }
        'ぶ' => {
            let mut result = word.to_string();
            result.pop();
            result.push_str("ばない");
            result
        }
        'む' => {
            let mut result = word.to_string();
            result.pop();
            result.push_str("まない");
            result
        }
        'る' => {
            let mut result = word.to_string();
            result.pop();
            result.push_str("らない");
            result
        }
        _ => format!("{}ない", word),
    }
}

/// Преобразует глагол в た-форму (аналогично て-форме, но с た вместо て)
pub fn to_ta_form(word: &str) -> String {
    // Получаем て-форму и заменяем て на た
    let te_form = to_te_form(word);
    te_form.replace("て", "た").replace("で", "だ")
}

/// Преобразует глагол в ます-форму
/// Просто добавляем ます к основе глагола
pub fn to_masu_form(word: &str) -> String {
    format!("{}ます", word)
}

/// Преобразует глагол в ません-форму
/// Добавляем ません к основе глагола
pub fn to_masen_form(word: &str) -> String {
    format!("{}ません", word)
}

/// Преобразует глагол в ましょう-форму
/// Добавляем ましょう к основе глагола
pub fn to_mashou_form(word: &str) -> String {
    format!("{}ましょう", word)
}

/// Преобразует глагол в ます-форму без ます (основа для других конструкций)
/// Возвращает основу глагола для конструкций типа ～たい, ～すぎる и т.д.
pub fn to_masu_stem(word: &str) -> String {
    word.to_string() // для базовых глаголов это просто исходная форма
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_te_form_group2() {
        assert_eq!(to_te_form("行く"), "行って");
        assert_eq!(to_te_form("話す"), "話して");
        assert_eq!(to_te_form("読む"), "読んで");
        assert_eq!(to_te_form("書く"), "書いて");
        assert_eq!(to_te_form("泳ぐ"), "泳いで");
    }

    #[test]
    fn test_te_form_group1() {
        assert_eq!(to_te_form("食べる"), "食べて");
        assert_eq!(to_te_form("見る"), "見て");
    }

    #[test]
    fn test_te_form_irregular() {
        assert_eq!(to_te_form("する"), "して");
        assert_eq!(to_te_form("くる"), "きて");
        assert_eq!(to_te_form("来る"), "きて");
    }

    #[test]
    fn test_nai_form_group2() {
        assert_eq!(to_nai_form("行く"), "行かない");
        assert_eq!(to_nai_form("話す"), "話さない");
        assert_eq!(to_nai_form("読む"), "読まない");
    }

    #[test]
    fn test_nai_form_group1() {
        assert_eq!(to_nai_form("食べる"), "食べない");
        assert_eq!(to_nai_form("見る"), "見ない");
    }

    #[test]
    fn test_nai_form_irregular() {
        assert_eq!(to_nai_form("する"), "しない");
        assert_eq!(to_nai_form("くる"), "こない");
    }

    #[test]
    fn test_ta_form() {
        assert_eq!(to_ta_form("行く"), "行った");
        assert_eq!(to_ta_form("食べる"), "食べた");
        assert_eq!(to_ta_form("する"), "した");
    }

    #[test]
    fn test_masu_form() {
        assert_eq!(to_masu_form("行く"), "行くます");
        assert_eq!(to_masu_form("食べる"), "食べます");
    }

    #[test]
    fn test_masen_form() {
        assert_eq!(to_masen_form("行く"), "行きません");
        assert_eq!(to_masen_form("食べる"), "食べません");
    }

    #[test]
    fn test_mashou_form() {
        assert_eq!(to_mashou_form("行く"), "行きましょう");
        assert_eq!(to_mashou_form("食べる"), "食べましょう");
    }
}
