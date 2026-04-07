//! Iteration 63 — SSE parser: data split across tiny chunks
//!
//! PASS: char-by-char line_buffer architecture handles arbitrary chunk splits.

use laminate::streaming::sse::SseParser;

#[test]
fn sse_char_by_char_reassembly() {
    let full = "data: hello\n\n";
    let mut parser = SseParser::new();
    for ch in full.chars() {
        let _ = parser.feed(&ch.to_string());
    }
    // Last char was second \n, which should have emitted the event
    // But feed returns events per call — collect from the final char
    let mut parser2 = SseParser::new();
    let mut all_events = Vec::new();
    for ch in full.chars() {
        all_events.extend(parser2.feed(&ch.to_string()));
    }
    assert_eq!(all_events.len(), 1);
    assert_eq!(all_events[0].data, "hello");
}

#[test]
fn sse_split_double_newline_across_chunks() {
    let mut parser = SseParser::new();
    let _ = parser.feed("data: split");
    let e1 = parser.feed("\n");
    assert!(e1.is_empty(), "first newline should not emit");
    let e2 = parser.feed("\n");
    assert_eq!(e2.len(), 1, "second newline completes the event");
    assert_eq!(e2[0].data, "split");
}

#[test]
fn sse_split_mid_json_reassembles() {
    let mut parser = SseParser::new();
    let _ = parser.feed("data: {\"ke");
    let _ = parser.feed("y\":\"val");
    let events = parser.feed("ue\"}\n\n");
    assert_eq!(events.len(), 1);
    let parsed: serde_json::Value = serde_json::from_str(&events[0].data).unwrap();
    assert_eq!(parsed["key"], "value");
}

#[test]
fn sse_split_field_name_across_chunks() {
    let mut parser = SseParser::new();
    let _ = parser.feed("ev");
    let _ = parser.feed("ent: msg\n");
    let _ = parser.feed("da");
    let _ = parser.feed("ta: payload");
    let events = parser.feed("\n\n");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, Some("msg".into()));
    assert_eq!(events[0].data, "payload");
}
