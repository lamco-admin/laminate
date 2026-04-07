//! Iteration 65 — Stream handlers: malformed JSON in data field
//!
//! PASS: Both handlers produce clean ParseError with preserved raw data.
//! Stream recovers immediately for subsequent valid events.

use laminate::streaming::{FlexStream, Provider, StreamConfig, StreamEvent};

#[test]
fn anthropic_malformed_json_produces_parse_error() {
    let mut stream = FlexStream::new(StreamConfig {
        provider: Provider::Anthropic,
        max_buffer_bytes: 1_048_576,
    });
    let events = stream.feed_str("event: message_start\ndata: {broken\n\n");
    assert_eq!(events.len(), 1);
    match &events[0] {
        StreamEvent::ParseError {
            event_type,
            raw_data,
            error,
        } => {
            assert_eq!(event_type, &Some("message_start".into()));
            assert_eq!(raw_data, "{broken");
            assert!(error.contains("key must be a string"));
        }
        other => panic!("expected ParseError, got {:?}", other),
    }
}

#[test]
fn anthropic_recovers_after_malformed() {
    let mut stream = FlexStream::new(StreamConfig {
        provider: Provider::Anthropic,
        max_buffer_bytes: 1_048_576,
    });
    // Broken event followed by valid event
    let events = stream.feed_str(
        "event: message_start\ndata: {broken\n\n\
         event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}\n\n",
    );
    assert_eq!(events.len(), 2);
    assert!(matches!(&events[0], StreamEvent::ParseError { .. }));
    assert!(matches!(&events[1], StreamEvent::TextDelta(t) if t == "Hello"));
}

#[test]
fn openai_malformed_then_valid_then_done() {
    let mut stream = FlexStream::new(StreamConfig {
        provider: Provider::OpenAI,
        max_buffer_bytes: 1_048_576,
    });
    let events = stream.feed_str(
        "data: {nope\n\n\
         data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"world\"}}]}\n\n\
         data: [DONE]\n\n",
    );
    assert_eq!(events.len(), 3);
    assert!(matches!(&events[0], StreamEvent::ParseError { .. }));
    assert!(matches!(&events[1], StreamEvent::TextDelta(t) if t == "world"));
    assert!(matches!(&events[2], StreamEvent::Stop(_)));
}
