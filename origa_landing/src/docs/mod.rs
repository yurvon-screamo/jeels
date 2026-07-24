//! Markdown source loader for `/docs` pages.
//!
//! Doc pages live as `content/docs/<locale>/<slug>.md` and are embedded into
//! the binary at compile time via `include_str!`. The frontmatter parser and
//! markdown renderer are reused from [`crate::blog`] — the frontmatter shape
//! is identical across blog and docs, so no duplication is needed. Only the
//! registry (the list of which files ship) is docs-specific.

pub mod registry;

pub use registry::{
    DocPage, SIDEBAR_SLUGS, all, find, index_page, list_by_locale, locales_for_slug,
    sidebar_entries,
};
