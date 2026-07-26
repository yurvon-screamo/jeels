use crate::domain::OrigaError;
use std::path::Path;

#[derive(Clone)]
pub struct Vocabulary {
    chars: Vec<char>,
}

impl Vocabulary {
    pub fn from_file(path: &Path) -> Result<Self, OrigaError> {
        let content = std::fs::read_to_string(path).map_err(|e| OrigaError::OcrError {
            reason: format!("Failed to read vocabulary file {:?}: {:?}", path, e),
        })?;

        let chars: Vec<char> = content
            .lines()
            .filter(|line| !line.is_empty())
            .flat_map(|line| line.chars().next())
            .collect();

        Ok(Self { chars })
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, OrigaError> {
        let content = std::str::from_utf8(bytes).map_err(|e| OrigaError::OcrError {
            reason: format!("Failed to parse vocabulary as UTF-8: {:?}", e),
        })?;

        let chars: Vec<char> = content
            .lines()
            .filter(|line| !line.is_empty())
            .flat_map(|line| line.chars().next())
            .collect();

        Ok(Self { chars })
    }

    pub fn decode(&self, indices: &[i64]) -> String {
        indices
            .iter()
            .filter(|&&idx| idx > 0)
            .filter_map(|&idx| {
                let char_idx = (idx - 1) as usize;
                self.chars.get(char_idx).copied()
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.chars.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // from_bytes tests
    #[test]
    fn from_bytes_empty() {
        let vocab = Vocabulary::from_bytes(b"").unwrap();
        assert!(vocab.is_empty());
    }

    #[test]
    fn from_bytes_single_token() {
        let vocab = Vocabulary::from_bytes(b"hello").unwrap();
        assert_eq!(vocab.len(), 1);
        assert!(!vocab.is_empty());
    }

    #[test]
    fn from_bytes_multiple_tokens() {
        let vocab = Vocabulary::from_bytes(b"a\nb\nc").unwrap();
        assert_eq!(vocab.len(), 3);
    }

    #[test]
    fn from_bytes_filters_empty_lines() {
        // Empty lines should be filtered out
        let vocab = Vocabulary::from_bytes(b"a\n\n\nb").unwrap();
        assert_eq!(vocab.len(), 2); // Only "a" and "b"
    }

    // decode tests
    #[test]
    fn decode_valid_indices() {
        let vocab = Vocabulary::from_bytes(b"a\nb\nc").unwrap();
        assert_eq!(vocab.decode(&[1, 2, 3]), "abc");
    }

    #[test]
    fn decode_filters_zero() {
        let vocab = Vocabulary::from_bytes(b"a\nb\nc").unwrap();
        assert_eq!(vocab.decode(&[0, 1, 0, 2, 0, 3, 0]), "abc");
    }

    #[test]
    fn decode_filters_negative() {
        let vocab = Vocabulary::from_bytes(b"a\nb\nc").unwrap();
        assert_eq!(vocab.decode(&[-1, 1, -5, 2, 3]), "abc");
    }

    #[test]
    fn decode_filters_out_of_bounds() {
        let vocab = Vocabulary::from_bytes(b"a\nb").unwrap();
        // Index 99 is out of bounds, should be filtered
        assert_eq!(vocab.decode(&[1, 99, 2]), "ab");
    }

    #[test]
    fn decode_empty_indices() {
        let vocab = Vocabulary::from_bytes(b"a\nb\nc").unwrap();
        assert_eq!(vocab.decode(&[]), "");
    }

    #[test]
    fn decode_all_invalid_indices() {
        let vocab = Vocabulary::from_bytes(b"a\nb\nc").unwrap();
        assert_eq!(vocab.decode(&[0, -1, 99, 0]), "");
    }

    // from_file tests
    #[test]
    fn from_file_creates_vocabulary() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "token1").unwrap();
        writeln!(temp_file, "token2").unwrap();

        let vocab = Vocabulary::from_file(temp_file.path()).unwrap();
        assert_eq!(vocab.len(), 2);
        assert_eq!(vocab.decode(&[1, 2]), "tt");
    }

    #[test]
    fn from_file_empty_file() {
        let temp_file = NamedTempFile::new().unwrap();
        // Write nothing

        let vocab = Vocabulary::from_file(temp_file.path()).unwrap();
        assert!(vocab.is_empty());
    }

    #[test]
    fn from_file_not_found() {
        let result = Vocabulary::from_file(std::path::Path::new("/nonexistent/file.txt"));
        assert!(result.is_err());
    }

    // is_empty tests
    #[test]
    fn is_empty_true() {
        let vocab = Vocabulary::from_bytes(b"").unwrap();
        assert!(vocab.is_empty());
    }

    #[test]
    fn is_empty_false() {
        let vocab = Vocabulary::from_bytes(b"a").unwrap();
        assert!(!vocab.is_empty());
    }
}
