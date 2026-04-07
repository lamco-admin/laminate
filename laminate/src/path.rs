//! Path parser for dot/bracket navigation syntax.
//!
//! Parses path strings like `"users[0].address.city"` into a sequence
//! of segments for FlexValue navigation. Supports dot keys, bracket
//! indices, and quoted keys for special characters.

use crate::error::{FlexError, Result};

/// A single segment in a navigation path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    /// An object key (e.g., "foo" in "foo.bar").
    Key(String),
    /// An array index (e.g., `0` in `items[0]`).
    Index(usize),
}

/// Parse a dot/bracket path string into a sequence of segments.
///
/// Supported syntax:
/// - `"foo"` — single key
/// - `"foo.bar"` — nested keys
/// - `"foo[0]"` — array index
/// - `"foo[0].bar.baz[2]"` — mixed
/// - `"meta[\"content-type\"]"` — quoted key for special characters
pub fn parse_path(path: &str) -> Result<Vec<Segment>> {
    if path.is_empty() {
        return Ok(vec![]);
    }

    let mut segments = Vec::new();
    let chars: Vec<char> = path.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '[' {
            i += 1; // skip '['

            if i >= len {
                return Err(FlexError::InvalidPath {
                    detail: "unexpected end after '['".into(),
                });
            }

            if chars[i] == '"' {
                // Quoted key: ["content-type"]
                i += 1; // skip opening quote
                let start = i;
                while i < len && chars[i] != '"' {
                    i += 1;
                }
                if i >= len {
                    return Err(FlexError::InvalidPath {
                        detail: "unterminated quoted key".into(),
                    });
                }
                let key: String = chars[start..i].iter().collect();
                i += 1; // skip closing quote

                if i >= len || chars[i] != ']' {
                    return Err(FlexError::InvalidPath {
                        detail: "expected ']' after quoted key".into(),
                    });
                }
                i += 1; // skip ']'

                segments.push(Segment::Key(key));
            } else if chars[i].is_ascii_digit() {
                // Numeric index: [0]
                let start = i;
                while i < len && chars[i].is_ascii_digit() {
                    i += 1;
                }
                if i >= len || chars[i] != ']' {
                    return Err(FlexError::InvalidPath {
                        detail: "expected ']' after index".into(),
                    });
                }
                let idx_str: String = chars[start..i].iter().collect();
                let idx: usize = idx_str.parse().map_err(|_| FlexError::InvalidPath {
                    detail: format!("invalid index: {idx_str}"),
                })?;
                i += 1; // skip ']'
                segments.push(Segment::Index(idx));
            } else {
                return Err(FlexError::InvalidPath {
                    detail: format!("unexpected character after '[': '{}'", chars[i]),
                });
            }

            // Skip a dot separator after bracket if present
            if i < len && chars[i] == '.' {
                i += 1;
            }
        } else {
            // Regular key: read until '.', '[', or end
            let start = i;
            while i < len && chars[i] != '.' && chars[i] != '[' {
                i += 1;
            }
            if i == start {
                return Err(FlexError::InvalidPath {
                    detail: "empty key segment".into(),
                });
            }
            let key: String = chars[start..i].iter().collect();
            segments.push(Segment::Key(key));

            // Skip dot separator
            if i < len && chars[i] == '.' {
                i += 1;
                // Trailing dot with nothing after it
                if i >= len {
                    return Err(FlexError::InvalidPath {
                        detail: "trailing dot in path".into(),
                    });
                }
            }
        }
    }

    Ok(segments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_key() {
        assert_eq!(parse_path("foo").unwrap(), vec![Segment::Key("foo".into())]);
    }

    #[test]
    fn nested_keys() {
        assert_eq!(
            parse_path("foo.bar.baz").unwrap(),
            vec![
                Segment::Key("foo".into()),
                Segment::Key("bar".into()),
                Segment::Key("baz".into()),
            ]
        );
    }

    #[test]
    fn array_index() {
        assert_eq!(
            parse_path("items[0]").unwrap(),
            vec![Segment::Key("items".into()), Segment::Index(0)]
        );
    }

    #[test]
    fn mixed_path() {
        assert_eq!(
            parse_path("choices[0].message.tool_calls[2].function.name").unwrap(),
            vec![
                Segment::Key("choices".into()),
                Segment::Index(0),
                Segment::Key("message".into()),
                Segment::Key("tool_calls".into()),
                Segment::Index(2),
                Segment::Key("function".into()),
                Segment::Key("name".into()),
            ]
        );
    }

    #[test]
    fn quoted_key() {
        assert_eq!(
            parse_path("meta[\"content-type\"]").unwrap(),
            vec![
                Segment::Key("meta".into()),
                Segment::Key("content-type".into()),
            ]
        );
    }

    #[test]
    fn empty_path() {
        assert_eq!(parse_path("").unwrap(), vec![]);
    }

    #[test]
    fn index_only() {
        assert_eq!(parse_path("[0]").unwrap(), vec![Segment::Index(0)]);
    }

    #[test]
    fn consecutive_indices() {
        assert_eq!(
            parse_path("matrix[0][1]").unwrap(),
            vec![
                Segment::Key("matrix".into()),
                Segment::Index(0),
                Segment::Index(1),
            ]
        );
    }

    #[test]
    fn invalid_unterminated_bracket() {
        assert!(parse_path("foo[0").is_err());
    }

    #[test]
    fn invalid_unterminated_quote() {
        assert!(parse_path("foo[\"bar").is_err());
    }

    #[test]
    fn invalid_empty_segment() {
        assert!(parse_path("foo..bar").is_err());
    }
}
