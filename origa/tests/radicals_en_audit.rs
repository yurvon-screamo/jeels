//! Radicals dictionary EN-fields audit.
//!
//! `cdn/dictionary/radicals.json` ships `name`/`description` (Russian) and
//! `name_en`/`description_en` (English). The EN fields are optional in the
//! wire format so old CDN artifacts keep parsing, but the shipped file must
//! carry them for every radical — an English user otherwise sees empty
//! radical copy in kanji lessons.
//!
//! The `cdn/` directory is gitignored; on a fresh clone the test gracefully
//! skips (same policy as `well_known_sets_audit.rs`).
//!
//! Run: `cargo test -p origa --test radicals_en_audit`.

use std::fs;
use std::path::PathBuf;

use origa::dictionary::radical::RadicalDatabase;
use origa::domain::NativeLanguage;

fn radicals_path() -> PathBuf {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set by cargo");
    PathBuf::from(manifest_dir)
        .parent()
        .expect("workspace root is parent of CARGO_MANIFEST_DIR")
        .join("cdn")
        .join("dictionary")
        .join("radicals.json")
}

fn load_database() -> Option<RadicalDatabase> {
    let path = radicals_path();
    if !path.exists() {
        eprintln!("radicals.json not found (cdn/ is gitignored), skipping audit");
        return None;
    }
    let json = fs::read_to_string(&path).expect("radicals.json must be readable");
    Some(
        RadicalDatabase::from_json(&json)
            .expect("shipped radicals.json must parse into RadicalDatabase"),
    )
}

#[test]
fn shipped_radicals_file_provides_english_for_every_radical() {
    let Some(db) = load_database() else {
        return;
    };

    let radicals = db.radical_list();
    assert!(!radicals.is_empty(), "the shipped file must not be empty");

    for info in &radicals {
        assert_ne!(
            info.name(&NativeLanguage::English),
            "",
            "radical {} must carry an English name",
            info.radical()
        );
        assert_ne!(
            info.description(&NativeLanguage::English),
            "",
            "radical {} must carry an English description",
            info.radical()
        );
    }
}

#[test]
fn shipped_radicals_file_keeps_russian_for_every_radical() {
    let Some(db) = load_database() else {
        return;
    };

    for info in db.radical_list() {
        assert_ne!(
            info.name(&NativeLanguage::Russian),
            "",
            "radical {} must keep its Russian name",
            info.radical()
        );
        assert_ne!(
            info.description(&NativeLanguage::Russian),
            "",
            "radical {} must keep its Russian description",
            info.radical()
        );
    }
}

/// Radicals that shipped with non-canonical Russian labels (e.g. 虍 as
/// «Кошка», 巛/阡 as «Гора», 无 as «Бесконечность» — the opposite of its
/// meaning). Guards a regeneration of radicals.json from a stale source
/// against silently reintroducing the mislabels. Spot list, not exhaustive.
#[test]
fn previously_mislabelled_radicals_carry_canonical_names() {
    let Some(db) = load_database() else {
        return;
    };

    let expected: &[(char, &str, &str)] = &[
        ('虍', "Тигр", "Tiger"),
        ('巛', "Извилистая река", "Winding river"),
        ('酉', "Сосуд для сакэ", "Sake vessel"),
        ('廾', "Две руки", "Two hands"),
        ('凵', "Сосуд", "Receptacle"),
        ('艮', "Остановка", "Stopping"),
        ('辰', "Знак дракона", "Zodiac dragon"),
        ('无', "Ничто", "Nothing"),
        ('飛', "Полёт", "Fly"),
        ('韋', "Выделанная кожа", "Tanned leather"),
        ('厂', "Утёс", "Cliff"),
        ('歹', "Смерть", "Death"),
        ('夂', "Медленный шаг", "Slow step"),
        ('彳', "Идущий", "Going man"),
        ('攵', "Удар", "Tap"),
        ('囗', "Ограда", "Enclosure"),
        ('黹', "Вышивка", "Embroidery"),
        ('止', "Стоп", "Stop"),
        ('亡', "Гибель", "Perish"),
    ];

    let mut seen: Vec<(char, String, String)> = Vec::new();
    for &(radical, ru_name, en_name) in expected {
        let info = db
            .radical_list()
            .into_iter()
            .find(|info| info.radical() == radical)
            .unwrap_or_else(|| panic!("radical {radical} must exist in the shipped file"));
        assert_eq!(
            info.name(&NativeLanguage::Russian),
            ru_name,
            "radical {radical} must carry its canonical Russian name"
        );
        assert_eq!(
            info.name(&NativeLanguage::English),
            en_name,
            "radical {radical} must carry its canonical English name"
        );
        // Distinct components must keep distinct names: the pre-fix data had
        // 儿/夂/彳 all labelled «Ноги» and 攵 colliding with 又/手 as «Рука».
        seen.push((radical, ru_name.to_string(), en_name.to_string()));
    }
    for (index, (_, ru_name, en_name)) in seen.iter().enumerate() {
        for (_, other_ru, other_en) in seen.iter().skip(index + 1) {
            assert_ne!(
                ru_name, other_ru,
                "distinct corrected radicals must not share a Russian name"
            );
            assert_ne!(
                en_name, other_en,
                "distinct corrected radicals must not share an English name"
            );
        }
    }

    // Description-level spot checks for the mislabelled semantics: the
    // canonical meaning must be present in the body text, not just the name.
    let desc_expectations: &[(char, &str, &str)] = &[
        ('虍', "тигра", "tiger"),
        ('酉', "сакэ", "sake"),
        ('无', "ничто", "nothing"),
        ('艮', "гора", "mountain"),
    ];
    for &(radical, ru_keyword, en_keyword) in desc_expectations {
        let info = db
            .radical_list()
            .into_iter()
            .find(|info| info.radical() == radical)
            .unwrap_or_else(|| panic!("radical {radical} must exist in the shipped file"));
        assert!(
            info.description(&NativeLanguage::Russian)
                .contains(ru_keyword),
            "radical {radical} description must mention its canonical meaning ({ru_keyword})"
        );
        assert!(
            info.description(&NativeLanguage::English)
                .to_lowercase()
                .contains(en_keyword),
            "radical {radical} English description must mention its canonical meaning ({en_keyword})"
        );
    }
}
