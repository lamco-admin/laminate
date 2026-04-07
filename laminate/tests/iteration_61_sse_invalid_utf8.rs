//! Iteration 61 — SSE parser: invalid UTF-8 bytes via feed_bytes()
//!
//! GAP: feed_bytes() silently dropped entire chunks on any invalid UTF-8 byte.
//! Fix: use String::from_utf8_lossy() to replace invalid bytes with U+FFFD.

use laminate::streaming::sse::SseParser;

#[test]
fn sse_invalid_utf8_replaces_with_fffd() {
    let mut parser = SseParser::new();

    // "data: hel\xFFlo\n\n" — invalid byte in the middle of data
    let mut bytes: Vec<u8> = Vec::new();
    bytes.extend_from_slice(b"data: hel");
    bytes.push(0xFF);
    bytes.extend_from_slice(b"lo\n\n");

    let events = parser.feed_bytes(&bytes);
    // Previously returned 0 events (entire chunk dropped).
    // Now: event emitted with U+FFFD replacing the bad byte.
    assert_eq!(events.len(), 1);
    assert!(
        events[0].data.contains('\u{FFFD}'),
        "should contain replacement char"
    );
    assert!(events[0].data.contains("hel"), "valid prefix preserved");
    assert!(events[0].data.contains("lo"), "valid suffix preserved");
}

#[test]
fn sse_invalid_utf8_partial_event_completion() {
    let mut parser = SseParser::new();

    // Feed start of event via valid bytes
    let e1 = parser.feed_bytes(b"data: star");
    assert!(e1.is_empty());

    // Feed completion via chunk with invalid byte — previously dropped entire chunk
    let mut mixed: Vec<u8> = Vec::new();
    mixed.push(0xFF);
    mixed.extend_from_slice(b"t\n\n");
    let e2 = parser.feed_bytes(&mixed);

    // Now completes the event (with U+FFFD for the bad byte)
    assert_eq!(e2.len(), 1);
    assert!(
        e2[0].data.starts_with("star"),
        "partial data preserved from first chunk"
    );
}

#[test]
fn sse_valid_utf8_no_allocation_overhead() {
    // from_utf8_lossy returns Cow::Borrowed for valid UTF-8 — no allocation
    let mut parser = SseParser::new();
    let events = parser.feed_bytes(b"data: hello\n\n");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data, "hello");
}

#[test]
fn sse_all_invalid_bytes_corrupt_current_line_only() {
    let mut parser = SseParser::new();
    // Continuation bytes without start byte — all invalid, no newline
    let events = parser.feed_bytes(&[0x80, 0x81, 0x82]);
    assert!(events.is_empty());

    // Next "data: after" gets prefixed with buffered U+FFFD chars in the line buffer,
    // making it "���data: after" which doesn't match "data:" prefix — line is corrupted.
    // But the newlines still delimit the event boundary correctly.
    let events2 = parser.feed_bytes(b"data: after\n\n");
    // The corrupted line is discarded (unknown field per SSE spec).
    // No event emitted because the "data:" prefix was obscured.
    assert_eq!(events2.len(), 0);

    // Parser recovers on the next clean event
    let events3 = parser.feed_bytes(b"data: recovered\n\n");
    assert_eq!(events3.len(), 1);
    assert_eq!(events3[0].data, "recovered");
}

#[test]
fn sse_invalid_bytes_with_newline_isolates_damage() {
    let mut parser = SseParser::new();
    // Invalid bytes followed by newline — the corrupted line is flushed
    let mut bytes = vec![0x80, 0x81, b'\n'];
    bytes.extend_from_slice(b"data: clean\n\n");

    let events = parser.feed_bytes(&bytes);
    // The newline after invalid bytes flushes the corrupted line (unknown field),
    // then "data: clean" parses normally
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data, "clean");
}
