use crate::application::FuriganaService;
use crate::domain::error::JeersError;
use autoruby::{annotate, format, select};

pub struct AutorubyFuriganaGenerator {
    annotator: annotate::Annotator<'static>,
}

impl AutorubyFuriganaGenerator {
    pub fn new() -> Result<Self, JeersError> {
        Ok(Self {
            annotator: annotate::Annotator::new_with_integrated_dictionary(),
        })
    }
}

impl FuriganaService for AutorubyFuriganaGenerator {
    fn get_furigana(&self, text: &str) -> String {
        self.annotator
            .annotate(text)
            .render(&select::heuristic::All, &format::Markdown)
    }
}
