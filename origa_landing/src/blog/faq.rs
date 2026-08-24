//! FAQ extraction from rendered article HTML.
//!
//! Articles end with an `## FAQ` section whose questions are `###` headings
//! followed by a single answer paragraph. After the markdown → HTML pipeline
//! that shape is a `<h3>question</h3>` tag immediately followed by a
//! `<p>answer</p>` paragraph, with no other `<h2>` sections after it. The
//! extractor walks those pairs so the same content drives both the visible
//! Q&A block and the `FAQPage` JSON-LD — one source of truth, no drift.

/// One question-answer pair extracted from the FAQ section.
#[derive(Debug, Clone, PartialEq)]
pub struct FaqEntry {
    pub question: String,
    pub answer: String,
}

/// Extract `(question, answer)` pairs from the trailing FAQ section of a
/// rendered article body.
///
/// The section is identified as everything after the **last** `<h2>` opening
/// tag (the FAQ is authored as the final section of every article). Within
/// it, each `<h3>…</h3>` opens an entry and the following `<p>…</p>` supplies
/// the answer. Answer inner HTML is kept verbatim (already ammonia-sanitized
/// upstream), so inline links survive into both the visible block and the
/// JSON-LD. Malformed tails — a dangling `<h3>` with no following `<p>` — are
/// skipped rather than panicking: a broken authoring format degrades to "no
/// FAQPage schema", never to a broken page.
pub fn extract_faq(html: &str) -> Vec<FaqEntry> {
    let Some(faq_start) = html.rfind("<h2>") else {
        return Vec::new();
    };
    let mut entries = Vec::new();
    let mut rest = &html[faq_start..];
    while let Some(q_open) = rest.find("<h3>") {
        let after_q_open = &rest[q_open + "<h3>".len()..];
        let Some(question_close) = after_q_open.find("</h3>") else {
            return entries;
        };
        let question = after_q_open[..question_close].to_string();
        rest = &after_q_open[question_close + "</h3>".len()..];

        let p_open = match rest.find("<p>") {
            Some(i) => i,
            None => return entries,
        };
        let after_p_open = &rest[p_open + "<p>".len()..];
        let Some(answer_close) = after_p_open.find("</p>") else {
            return entries;
        };
        let answer = after_p_open[..answer_close].to_string();
        rest = &after_p_open[answer_close + "</p>".len()..];

        if !question.is_empty() && !answer.is_empty() {
            entries.push(FaqEntry { question, answer });
        }
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_question_answer_pairs() {
        let html = "<h2>Реалистичный итог</h2>\
<p>Итоговый текст.</p>\
<h2>FAQ</h2>\
<h3>Вопрос первый?</h3>\
<p>Ответ первый.</p>\
<h3>Вопрос второй?</h3>\
<p>Ответ <a href=\"/ru/compare\">со ссылкой</a>.</p>";
        let faq = extract_faq(html);
        assert_eq!(faq.len(), 2);
        assert_eq!(faq[0].question, "Вопрос первый?");
        assert_eq!(faq[0].answer, "Ответ первый.");
        assert_eq!(faq[1].question, "Вопрос второй?");
        assert_eq!(
            faq[1].answer,
            "Ответ <a href=\"/ru/compare\">со ссылкой</a>."
        );
    }

    #[test]
    fn ignores_h3_outside_trailing_faq_section() {
        // Only pairs after the LAST <h2> count — earlier h3s belong to
        // regular article subsections and must not leak into the schema.
        let html = "<h2>Раздел</h2>\
<h3>Обычный подраздел</h3>\
<p>Не вопрос.</p>\
<h2>FAQ</h2>\
<h3>Настоящий вопрос?</h3>\
<p>Настоящий ответ.</p>";
        let faq = extract_faq(html);
        assert_eq!(faq.len(), 1);
        assert_eq!(faq[0].question, "Настоящий вопрос?");
    }

    #[test]
    fn returns_empty_without_faq_section() {
        assert!(extract_faq("<p>Короткая заметка без FAQ.</p>").is_empty());
    }

    #[test]
    fn dangling_h3_degrades_to_no_entries() {
        // A malformed tail must not panic and must not emit a half pair —
        // the page renders fine, just without FAQPage JSON-LD.
        let html = "<h2>FAQ</h2><h3>Вопрос без ответа?</h3>";
        assert!(extract_faq(html).is_empty());
    }

    #[test]
    fn empty_question_is_skipped() {
        let html = "<h2>FAQ</h2><h3></h3><p>Ответ.</p><h3>Ок?</h3><p>Да.</p>";
        let faq = extract_faq(html);
        assert_eq!(faq.len(), 1);
        assert_eq!(faq[0].question, "Ок?");
    }
}
