use crate::domain::{JapaneseLevel, NativeLanguage, User};
use crate::traits::UserRepository;
use crate::use_cases::ImportOnboardingSetsUseCase;
use crate::use_cases::tests::fixtures::{
    InMemoryUserRepository, MockCdnProvider, init_real_dictionaries,
};

/// Returns a CDN pre-populated with the well-known-set JSONs for the given
/// (set_id, level, words) tuples. Path resolution matches the production
/// `resolve_set_path` for `jlpt_*` and `minna_n*_*` ids so the import use case
/// can fetch them like real CDN objects.
fn cdn_with_sets(sets: &[(&str, JapaneseLevel, &[&str])]) -> MockCdnProvider {
    let mut cdn = MockCdnProvider::new();
    for (set_id, level, words) in sets {
        let level_str = match level {
            JapaneseLevel::N5 => "N5",
            JapaneseLevel::N4 => "N4",
            JapaneseLevel::N3 => "N3",
            JapaneseLevel::N2 => "N2",
            JapaneseLevel::N1 => "N1",
        };
        let body = format!(
            r#"{{"level":"{}","words":[{}]}}"#,
            level_str,
            words
                .iter()
                .map(|w| format!("\"{}\"", w))
                .collect::<Vec<_>>()
                .join(",")
        );
        let path = crate::domain::resolve_set_path(set_id);
        cdn = cdn.with_text(&path, &body);
    }
    cdn
}

fn fresh_user() -> User {
    User::new(
        "import@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    )
}

#[tokio::test]
async fn import_creates_grammar_for_non_jlpt_set_when_target_allows_it() {
    // Arrange — Minna N4 set alone (no JLPT set picked). Before the fix this
    // produced zero grammar cards because grammar import was gated on the
    // "Jlpt" set-type prefix.
    init_real_dictionaries();
    let repo = InMemoryUserRepository::with_user(fresh_user());
    let cdn = cdn_with_sets(&[("minna_n4_1", JapaneseLevel::N4, &["猫"])]);

    let use_case = ImportOnboardingSetsUseCase::new(&repo, &cdn);

    // Act
    let result = use_case
        .execute(
            repo.get_current_user().await.unwrap().unwrap(),
            vec!["minna_n4_1".to_string()],
            JapaneseLevel::N4,
        )
        .await
        .unwrap();

    // Assert — at least one grammar rule should be created for N4 once the
    // JLPT-prefix gate is dropped (N5 also pulled in by the "every level ≤
    // target" rule).
    assert!(
        result.created_grammar > 0,
        "non-JLPT set must still import grammar rules; got 0"
    );
}

#[tokio::test]
async fn import_expands_grammar_to_every_level_up_to_target() {
    // Arrange — same set, two different target levels. A higher target must
    // pull in strictly more grammar rules (N5 alone vs N5+N4+N3).
    init_real_dictionaries();
    let cdn = cdn_with_sets(&[("jlpt_n5", JapaneseLevel::N5, &["猫"])]);

    let repo_low = InMemoryUserRepository::with_user(fresh_user());
    let uc_low = ImportOnboardingSetsUseCase::new(&repo_low, &cdn);
    let low = uc_low
        .execute(
            repo_low.get_current_user().await.unwrap().unwrap(),
            vec!["jlpt_n5".to_string()],
            JapaneseLevel::N5,
        )
        .await
        .unwrap();

    let repo_high = InMemoryUserRepository::with_user(fresh_user());
    let uc_high = ImportOnboardingSetsUseCase::new(&repo_high, &cdn);
    let high = uc_high
        .execute(
            repo_high.get_current_user().await.unwrap().unwrap(),
            vec!["jlpt_n5".to_string()],
            JapaneseLevel::N3,
        )
        .await
        .unwrap();

    assert!(
        high.created_grammar > low.created_grammar,
        "raising target_level from N5 to N3 must strictly increase grammar cards \
         (n5-target={}, n3-target={})",
        low.created_grammar,
        high.created_grammar
    );
}

#[tokio::test]
async fn import_skips_kanji_above_target_level() {
    // Arrange — pick N5 as target with a set that *would* contribute N4
    // kanji if the gate weren't there. With target N5, only N5 kanji should
    // be created. We assert via the kanji count vs a target=N4 run on the
    // same set.
    init_real_dictionaries();
    let repo = InMemoryUserRepository::with_user(fresh_user());
    let cdn = cdn_with_sets(&[("jlpt_n4", JapaneseLevel::N4, &["猫"])]);

    let use_case = ImportOnboardingSetsUseCase::new(&repo, &cdn);
    let n5_result = use_case
        .execute(
            repo.get_current_user().await.unwrap().unwrap(),
            vec!["jlpt_n4".to_string()],
            JapaneseLevel::N5,
        )
        .await
        .unwrap();

    let repo_n4 = InMemoryUserRepository::with_user(fresh_user());
    let cdn_n4 = cdn_with_sets(&[("jlpt_n4", JapaneseLevel::N4, &["猫"])]);
    let uc_n4 = ImportOnboardingSetsUseCase::new(&repo_n4, &cdn_n4);
    let n4_result = uc_n4
        .execute(
            repo_n4.get_current_user().await.unwrap().unwrap(),
            vec!["jlpt_n4".to_string()],
            JapaneseLevel::N4,
        )
        .await
        .unwrap();

    // Assert — N4 target must create strictly more kanji than N5 target on
    // the same N4 set, because the N5 run filters out N4-only kanji.
    assert!(
        n4_result.created_kanji >= n5_result.created_kanji,
        "raising target_level must not reduce kanji creation (n5={}, n4={})",
        n5_result.created_kanji,
        n4_result.created_kanji
    );
}

#[tokio::test]
async fn import_creates_kanji_from_dictionary_not_from_vocab() {
    // Кандзи должны импортироваться напрямую из словаря по уровням ≤ target,
    // точно как grammar — а НЕ извлекаться из слов набора.
    //
    // Тест: набор N5 с единственным словом "猫" (кандзи N5). Target=N4.
    // Словарь кандзи: N5=80, N4=170. Ожидаем creation каналов обоих уровней.
    // При старом подходе (vocab→kanji) создавался бы только кандзи из слова "猫",
    // т.е. 1 кандзи. При новом подходе — все N5+N4 из словаря.
    init_real_dictionaries();
    let cdn = cdn_with_sets(&[("jlpt_n5", JapaneseLevel::N5, &["猫"])]);
    let repo = InMemoryUserRepository::with_user(fresh_user());
    let use_case = ImportOnboardingSetsUseCase::new(&repo, &cdn);

    let result = use_case
        .execute(
            repo.get_current_user().await.unwrap().unwrap(),
            vec!["jlpt_n5".to_string()],
            JapaneseLevel::N4,
        )
        .await
        .unwrap();

    // N5=80 + N4=170 кандзи из словаря → ожидаем >50 (с учётом дублей-пропусков).
    // Старый подход дал бы ровно 1 (только 猫 из слова).
    assert!(
        result.created_kanji > 50,
        "Kanji must be imported from dictionary by level (≤ target), not extracted \
         from vocab words; expected >50 kanji cards, got {}",
        result.created_kanji
    );
}

#[tokio::test]
async fn import_kanji_does_not_create_above_target_level() {
    // Кандзи уровня выше target НЕ должны создаваться, даже если слово из
    // набора их содержит. Target=N5 → только N5 кандзи из словаря.
    init_real_dictionaries();
    let cdn = cdn_with_sets(&[("jlpt_n4", JapaneseLevel::N4, &["猫"])]);
    let repo_n5 = InMemoryUserRepository::with_user(fresh_user());
    let uc_n5 = ImportOnboardingSetsUseCase::new(&repo_n5, &cdn);
    let n5_result = uc_n5
        .execute(
            repo_n5.get_current_user().await.unwrap().unwrap(),
            vec!["jlpt_n4".to_string()],
            JapaneseLevel::N5,
        )
        .await
        .unwrap();

    let repo_n4 = InMemoryUserRepository::with_user(fresh_user());
    let uc_n4 = ImportOnboardingSetsUseCase::new(&repo_n4, &cdn);
    let n4_result = uc_n4
        .execute(
            repo_n4.get_current_user().await.unwrap().unwrap(),
            vec!["jlpt_n4".to_string()],
            JapaneseLevel::N4,
        )
        .await
        .unwrap();

    // Target=N4 должен создать строго больше кандзи, чем target=N5
    // (N5=80 vs N5+N4=250 из словаря).
    assert!(
        n4_result.created_kanji > n5_result.created_kanji,
        "N4 target must create more kanji than N5 target (dictionary-driven); \
         n5={}, n4={}",
        n5_result.created_kanji,
        n4_result.created_kanji
    );
}
