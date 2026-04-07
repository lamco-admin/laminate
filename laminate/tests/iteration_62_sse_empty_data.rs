//! Iteration 62 — SSE parser: empty data: line
//!
//! PASS: SSE parser correctly emits events with empty data per spec.
//! Handlers correctly produce ParseError for unparseable empty data.

use laminate::streaming::sse::SseParser;
use laminate::streaming::{FlexStream, Provider, StreamConfig, StreamEvent};

#[test]
fn sse_empty_data_emits_event() {
    let mut parser = SseParser::new();

    // "data:\n\n" — empty data field, spec says emit event with empty string
    let events = parser.feed("data:\n\n");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data, "");
    assert_eq!(events[0].event_type, None);
}

#[test]
fn sse_empty_data_with_space_also_empty() {
    let mut parser = SseParser::new();

    // "data: \n\n" — space after colon stripped by trim_start
    let events = parser.feed("data: \n\n");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data, "");
}

#[test]
fn sse_multiple_empty_data_lines_join() {
    let mut parser = SseParser::new();

    // Per SSE spec: multiple data lines join with "\n"
    // Three empty data lines → "" + "\n" + "" + "\n" + "" = "\n\n"
    let events = parser.feed("data:\ndata:\ndata:\n\n");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data, "\n\n");
}

#[test]
fn sse_empty_data_followed_by_normal() {
    let mut parser = SseParser::new();
    let events = parser.feed("data:\n\ndata: hello\n\n");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].data, "");
    assert_eq!(events[1].data, "hello");
}

#[test]
fn openai_handler_empty_data_produces_parse_error() {
    let mut stream = FlexStream::new(StreamConfig {
        provider: Provider::OpenAI,
        max_buffer_bytes: 1_048_576,
    });
    let events = stream.feed_str("data:\n\n");
    assert_eq!(events.len(), 1);
    assert!(
        matches!(&events[0], StreamEvent::ParseError { raw_data, .. } if raw_data.is_empty()),
        "empty data should produce ParseError, not be silently dropped"
    );
}

#[test]
fn anthropic_handler_no_event_type_ignored() {
    // Anthropic handler requires event_type — bare data events are skipped
    let mut stream = FlexStream::new(StreamConfig {
        provider: Provider::Anthropic,
        max_buffer_bytes: 1_048_576,
    });
    let events = stream.feed_str("data:\n\n");
    assert!(
        events.is_empty(),
        "Anthropic ignores events without event: type"
    );
}
