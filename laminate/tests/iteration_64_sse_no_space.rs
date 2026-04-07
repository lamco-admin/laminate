//! Iteration 64 — SSE parser: spec-correct single-space stripping
//!
//! GAP: trim_start() stripped ALL leading whitespace. SSE spec §9.2.6
//! says strip exactly one leading U+0020 SPACE. Fix: strip_one_leading_space().

use laminate::streaming::sse::SseParser;

#[test]
fn sse_no_space_after_colon() {
    let mut parser = SseParser::new();
    let events = parser.feed("data:hello\n\n");
    assert_eq!(events[0].data, "hello");
}

#[test]
fn sse_one_space_stripped() {
    let mut parser = SseParser::new();
    let events = parser.feed("data: hello\n\n");
    assert_eq!(events[0].data, "hello");
}

#[test]
fn sse_only_one_space_stripped_not_all() {
    let mut parser = SseParser::new();
    // Two spaces: strip one, preserve one
    let events = parser.feed("data:  hello\n\n");
    assert_eq!(events[0].data, " hello");
}

#[test]
fn sse_tab_not_stripped() {
    let mut parser = SseParser::new();
    // Tab is not U+0020 SPACE — preserved per spec
    let events = parser.feed("data:\thello\n\n");
    assert_eq!(events[0].data, "\thello");
}

#[test]
fn sse_spaces_only_data_preserves_extra() {
    let mut parser = SseParser::new();
    // Three spaces: strip one → two spaces remain
    let events = parser.feed("data:   \n\n");
    assert_eq!(events[0].data, "  ");
}

#[test]
fn sse_json_without_space_parses() {
    let mut parser = SseParser::new();
    // Common in the wild: JSON immediately after colon
    let events = parser.feed("data:{\"key\":\"value\"}\n\n");
    let parsed: serde_json::Value = serde_json::from_str(&events[0].data).unwrap();
    assert_eq!(parsed["key"], "value");
}
