//! Markdown source loader and HTML renderer for `/blog/<slug>` pages.
//!
//! Articles live as `content/blog/<locale>/<slug>.md` and are embedded into the
//! binary at compile time via `include_str!`. The first call to [`registry`]
//! parses every article's frontmatter and body once, stores them in a
//! `OnceLock<Vec<BlogPost>>` (chosen over `LazyLock` for MSRV 1.75 —
//! `LazyLock` requires 1.80+), and subsequent lookups are zero-cost. A draft
//! article (`status: draft` in frontmatter) is a programmer error and panics
//! on first access — drafts must not ship.

pub mod faq;
pub mod frontmatter;
pub mod registry;
pub mod render;

pub use registry::{BlogPost, all, find, list_by_locale, locales_for_slug};
