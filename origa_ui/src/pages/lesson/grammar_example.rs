//! Фронт тренировки грамматики: японская строка примера без перевода.
//!
//! Формат `examples` (cdn/grammar): последовательные code-fence по две
//! строки — японское предложение, затем перевод. Фронт показывает только
//! японскую строку первого примера: перевод раскрывает смысл, а смысл и
//! есть ответ (спека §Тренировка: «пример с конструкцией → её смысл»).

/// Первая непустая строка первого code-fence: японское предложение без
/// перевода. `None`, если examples пуст или fence не найден — фронт
/// вырождается в заголовок конструкции.
pub fn grammar_example_front(examples_markdown: &str) -> Option<String> {
    let fence_start = examples_markdown.find("```")?;
    let after_fence = &examples_markdown[fence_start + 3..];
    // Пропускаем строку языка fence (в контенте грамматики она пустая).
    let body_start = after_fence.find('\n')? + 1;
    let body = &after_fence[body_start..];
    // Незакрытый fence — битые данные: фронтом становится заголовок.
    let body_end = body.find("```")?;
    body[..body_end]
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod grammar_example_front_tests {
    use super::*;

    #[test]
    fn returns_japanese_line_without_translation() {
        // Arrange: реальный формат контента — JP-строка, затем перевод
        let examples = "```\n私は学生です。\nI am a student.\n```";

        // Act
        let front = grammar_example_front(examples);

        // Assert
        assert_eq!(front.as_deref(), Some("私は学生です。"));
    }

    #[test]
    fn takes_only_the_first_fence_of_several() {
        // Arrange
        let examples = "```\n今日は晴れです。\nIt is sunny today.\n```\n\n```\n猫がいます。\nThere is a cat.\n```";

        // Act
        let front = grammar_example_front(examples);

        // Assert
        assert_eq!(front.as_deref(), Some("今日は晴れです。"));
    }

    #[test]
    fn skips_blank_lines_inside_the_fence() {
        // Arrange
        let examples = "```\n\n明日行きます。\nI will go tomorrow.\n```";

        // Act / Assert
        assert_eq!(
            grammar_example_front(examples).as_deref(),
            Some("明日行きます。")
        );
    }

    #[rstest::rstest]
    #[case::empty(String::new())]
    #[case::no_fence("просто текст без разметки")]
    #[case::unclosed_fence("```\n訳があります")]
    fn returns_none_when_no_complete_example(#[case] examples: String) {
        assert_eq!(grammar_example_front(&examples), None);
    }
}
