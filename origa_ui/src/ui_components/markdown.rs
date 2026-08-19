use std::collections::HashSet;

use ammonia::clean;
use ego_tree::NodeRef;
use leptos::prelude::*;
use origa::domain::furiganize_text;
use pulldown_cmark::{Options, Parser, html};
use scraper::{Html, Node};

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum MarkdownVariant {
    #[default]
    Default,
    Compact,
    Large,
}

/// Terminal punctuation that must never start a visual line. Chromium's
/// emergency word-break ignores kinsoku rules (UAX #14 LB13), so a wrapped
/// line can start with "," / ";" / "!" / "、" — gluing is required.
const TRAILING_PUNCT: &str = "、。，．，．；：！？;:,.!?…»」』）)]";

/// Wraps every [word char + trailing punctuation run] cluster in a
/// `<span class="markdown-no-break">` so the browser treats it as an
/// unbreakable unit. The class resets the inherited eager breakers
/// (`word-break`/`overflow-wrap`) inside the span — without that reset the
/// parent `.markdown-text` styles re-enable breaking inside the span and
/// the glue is ineffective.
///
/// Operates on plain text only: tag interiors and HTML entities are copied
/// verbatim. Clusters split by a tag boundary (e.g. `<em>bold</em>,`) are
/// not glued — quiz option strings are plain text, verified against
/// cdn/dictionary data.
fn glue_trailing_punctuation(html: &str) -> String {
    let is_anchor = |c: char| c.is_alphanumeric();
    let is_trailing = |c: char| TRAILING_PUNCT.contains(c);

    let mut out = String::with_capacity(html.len());
    let mut chars = html.char_indices().peekable();
    let mut in_tag = false;

    while let Some((idx, c)) = chars.next() {
        if c == '<' {
            // Idempotency: our own glue spans are copied verbatim so a
            // second pass does not nest spans.
            if html[idx..].starts_with("<span class=\"markdown-no-break\">") {
                if let Some(end) = html[idx..].find("</span>") {
                    let end = idx + end + "</span>".len();
                    out.push_str(&html[idx..end]);
                    // Re-sync the iterator past the copied block.
                    while let Some(&(pos, _)) = chars.peek() {
                        if pos >= end {
                            break;
                        }
                        chars.next();
                    }
                    continue;
                }
            }
            in_tag = true;
            out.push(c);
            continue;
        }
        if in_tag {
            if c == '>' {
                in_tag = false;
            }
            out.push(c);
            continue;
        }
        if c == '&' {
            // Copy HTML entities (&amp; &lt; &#39; …) verbatim: their ';'
            // is markup, not glueable punctuation.
            out.push(c);
            while let Some(&(_, next)) = chars.peek() {
                if next.is_ascii_alphanumeric() || next == '#' {
                    out.push(next);
                    chars.next();
                } else {
                    break;
                }
            }
            if let Some(&(_, ';')) = chars.peek() {
                out.push(';');
                chars.next();
            }
            continue;
        }
        if is_anchor(c) {
            let mut punct_run = String::new();
            while let Some(&(_, next)) = chars.peek() {
                if is_trailing(next) {
                    punct_run.push(next);
                    chars.next();
                } else {
                    break;
                }
            }
            if punct_run.is_empty() {
                out.push(c);
            } else {
                let anchor = &html[idx..idx + c.len_utf8()];
                out.push_str("<span class=\"markdown-no-break\">");
                out.push_str(anchor);
                out.push_str(&punct_run);
                out.push_str("</span>");
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn render_markdown(content: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(content, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    clean(&html_output)
}

// Furigana is intentionally skipped inside existing ruby markup to avoid
// nested <ruby> elements. Code-fence (<pre><code>) content is NOT skipped:
// every grammar-rule "examples" block is rendered as a code fence (see
// cdn/grammar/rules/*.json) and contains Japanese-language content that
// needs furigana — verified by audit (3253/3253 code blocks contain kana/kanji).
const SKIP_TAGS: &[&str] = &["ruby", "rt", "rp"];

fn add_furigana_to_html(html: &str, known_kanji: &HashSet<char>) -> String {
    let document = Html::parse_document(html);
    let mut result = String::new();

    fn process_node(
        node_ref: NodeRef<'_, Node>,
        output: &mut String,
        in_skip: bool,
        known_kanji: &HashSet<char>,
    ) {
        match node_ref.value() {
            Node::Text(text) => {
                let text_str: &str = text;
                if in_skip {
                    output.push_str(text_str);
                } else {
                    match furiganize_text(text_str, known_kanji) {
                        Ok(furigana) => output.push_str(&furigana),
                        Err(_) => output.push_str(text_str),
                    }
                }
            },
            Node::Element(elem) => {
                let tag = elem.name();
                let should_skip = in_skip || SKIP_TAGS.contains(&tag);

                output.push_str(&format!("<{}", tag));
                for (name, value) in elem.attrs() {
                    output.push_str(&format!(" {}=\"{}\"", name, value));
                }
                output.push('>');

                for child in node_ref.children() {
                    process_node(child, output, should_skip, known_kanji);
                }

                output.push_str(&format!("</{}>", tag));
            },
            _ => {
                for child in node_ref.children() {
                    process_node(child, output, in_skip, known_kanji);
                }
            },
        }
    }

    for node_ref in document.tree.root().children() {
        process_node(node_ref, &mut result, false, known_kanji);
    }

    result
}

#[component]
pub fn MarkdownText(
    #[prop(into)] content: Signal<String>,
    known_kanji: HashSet<char>,
    #[prop(optional, into)] variant: Signal<MarkdownVariant>,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, default = true)] furigana: bool,
    #[prop(optional, default = false)] glue_punctuation: bool,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let html_content = Memo::new(move |_| {
        let rendered = render_markdown(&content.get());
        let rendered = if furigana {
            add_furigana_to_html(&rendered, &known_kanji)
        } else {
            rendered
        };
        if glue_punctuation {
            glue_trailing_punctuation(&rendered)
        } else {
            rendered
        }
    });

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <div
            class=move || {
                let variant_class = match variant.get() {
                    MarkdownVariant::Default => "prose prose-sm",
                    MarkdownVariant::Compact => "prose prose-xs",
                    MarkdownVariant::Large => "prose prose-lg",
                };
                format!("markdown-text {} {}", variant_class, class.get())
            }
            data-testid=test_id_val
        >
            <div inner_html=move || html_content.get() />
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_basic_markdown() {
        let input = "**bold** and *italic*";
        let output = render_markdown(input);
        assert!(output.contains("<strong>bold</strong>"));
        assert!(output.contains("<em>italic</em>"));
    }

    #[test]
    fn test_render_heading() {
        let input = "# Heading 1\n## Heading 2";
        let output = render_markdown(input);
        assert!(output.contains("<h1>Heading 1</h1>"));
        assert!(output.contains("<h2>Heading 2</h2>"));
    }

    #[test]
    fn test_render_list() {
        let input = "- item 1\n- item 2";
        let output = render_markdown(input);
        assert!(output.contains("<ul>"));
        assert!(output.contains("<li>item 1</li>"));
        assert!(output.contains("<li>item 2</li>"));
    }

    #[test]
    fn test_render_code() {
        let input = "`inline code`";
        let output = render_markdown(input);
        assert!(output.contains("<code>inline code</code>"));
    }

    #[test]
    fn test_render_link() {
        let input = "[text](https://example.com)";
        let output = render_markdown(input);
        assert!(output.contains("<a"));
        assert!(output.contains("href=\"https://example.com\""));
        assert!(output.contains(">text</a>"));
    }

    #[test]
    fn test_sanitize_script_tag() {
        let input = "<script>alert('xss')</script>";
        let output = render_markdown(input);
        assert!(!output.contains("<script>"));
        assert!(!output.contains("alert"));
    }

    #[test]
    fn test_sanitize_javascript_link() {
        let input = "[click](javascript:alert('xss'))";
        let output = render_markdown(input);
        assert!(!output.contains("javascript:"));
    }

    #[test]
    fn test_sanitize_event_handler() {
        let input = "<img src=\"x\" onerror=\"alert('xss')\">";
        let output = render_markdown(input);
        assert!(!output.contains("onerror"));
    }

    #[test]
    fn test_empty_input() {
        let output = render_markdown("");
        assert!(output.is_empty());
    }

    #[test]
    fn glue_wraps_letter_with_trailing_punctuation() {
        let out = glue_trailing_punctuation("стремительный, проворный");
        assert_eq!(
            out,
            "стремительны<span class=\"markdown-no-break\">й,</span> проворный"
        );
    }

    #[test]
    fn glue_wraps_punctuation_runs() {
        let out = glue_trailing_punctuation("проворный; молниеносный");
        assert!(out.contains("<span class=\"markdown-no-break\">й;</span>"));
    }

    #[test]
    fn glue_handles_japanese_punctuation() {
        let out = glue_trailing_punctuation("はやい、すばやい");
        assert!(out.contains("<span class=\"markdown-no-break\">い、</span>"));
    }

    #[test]
    fn glue_no_change_without_punctuation() {
        assert_eq!(glue_trailing_punctuation("просто текст"), "просто текст");
    }

    #[test]
    fn glue_preserves_tag_interiors() {
        let out = glue_trailing_punctuation("<p class=\"x\">слово, второе</p>");
        assert!(out.starts_with("<p class=\"x\">"));
        assert!(out.ends_with("</p>"));
        assert!(out.contains("<span class=\"markdown-no-break\">о,</span>"));
    }

    #[test]
    fn glue_preserves_html_entities() {
        // &amp; must not be treated as an anchor+punct cluster
        let out = glue_trailing_punctuation("rock &amp; roll, друг");
        assert!(out.contains("&amp;"));
        assert!(out.contains("<span class=\"markdown-no-break\">l,</span>"));
    }

    #[test]
    fn glue_span_is_idempotent() {
        let once = glue_trailing_punctuation("быстрый, скорый");
        let twice = glue_trailing_punctuation(&once);
        assert_eq!(once, twice);
    }

    #[test]
    fn glue_does_not_touch_punctuation_alone() {
        // A punctuation run not preceded by a word char (e.g. after a space
        // or at string start) is not wrapped: there is nothing to glue to.
        assert_eq!(glue_trailing_punctuation("..."), "...");
    }

    #[test]
    fn test_add_furigana_preserves_html_structure() {
        let html = "<p>Hello world</p>";
        let known_kanji = HashSet::new();
        let output = add_furigana_to_html(html, &known_kanji);
        assert!(output.contains("<p>"));
        assert!(output.contains("</p>"));
        assert!(output.contains("Hello"));
        assert!(output.contains("world"));
    }

    #[test]
    fn test_add_furigana_skips_code_tag() {
        let html = "<code>test</code>";
        let known_kanji = HashSet::new();
        let output = add_furigana_to_html(html, &known_kanji);
        assert!(output.contains("<code>test</code>"));
    }

    #[test]
    fn test_pre_tag_not_skipped_for_furigana() {
        // Regression for issue #178 W-12: grammar-rule example blocks are
        // rendered as <pre><code>...</code></pre> by pulldown-cmark when the
        // source uses ``` ``` ``` fences. All 3253 example blocks across 332
        // grammar rule files contain Japanese language content (audit-verified)
        // and need furigana. Keeping <pre> out of SKIP_TAGS lets the text-node
        // walker apply ruby markup to these example blocks.
        //
        // Note on test scope: a full behavioral test that asserts ruby markup
        // appears inside <pre> requires the lindera dictionary to be loaded,
        // which is not available in `origa_ui` unit tests. The behavioral
        // verification lives in `origa/src/domain/furigana.rs` integration
        // tests (e.g. `furigana_text_unknown_kanji_gets_reading`) which run
        // with the real CDN dictionary. This const-membership test guards
        // the regression path that would re-introduce `<pre>` to SKIP_TAGS.
        assert!(
            !SKIP_TAGS.contains(&"pre"),
            "pre must not be skipped: furigana is needed inside grammar example code-fences"
        );
        assert!(SKIP_TAGS.contains(&"ruby"));
        assert!(SKIP_TAGS.contains(&"rt"));
        assert!(SKIP_TAGS.contains(&"rp"));
    }

    #[test]
    fn test_add_furigana_skips_ruby_tag() {
        let html = "<ruby>食<rt>しょく</rt></ruby>";
        let known_kanji = HashSet::new();
        let output = add_furigana_to_html(html, &known_kanji);
        assert!(output.contains("<ruby>"));
        assert!(output.contains("<rt>"));
    }

    #[test]
    fn test_add_furigana_preserves_links() {
        let html = "<a href=\"https://example.com\">link</a>";
        let known_kanji = HashSet::new();
        let output = add_furigana_to_html(html, &known_kanji);
        assert!(output.contains("href=\"https://example.com\""));
        assert!(output.contains(">link</a>"));
    }

    #[test]
    fn test_add_furigana_nested_elements() {
        let html = "<div><p>text</p></div>";
        let known_kanji = HashSet::new();
        let output = add_furigana_to_html(html, &known_kanji);
        assert!(output.contains("<div>"));
        assert!(output.contains("<p>"));
        assert!(output.contains("text"));
        assert!(output.contains("</p>"));
        assert!(output.contains("</div>"));
    }

    #[test]
    fn test_add_furigana_code_inside_p() {
        let html = "<p>text <code>code</code> more</p>";
        let known_kanji = HashSet::new();
        let output = add_furigana_to_html(html, &known_kanji);
        assert!(output.contains("text"));
        assert!(output.contains("<code>code</code>"));
        assert!(output.contains("more"));
    }

    #[test]
    fn test_render_hard_break() {
        let input = "line1  \nline2";
        let output = render_markdown(input);
        assert!(
            output.contains("<br"),
            "Expected hard break, got: {}",
            output
        );
    }
}
