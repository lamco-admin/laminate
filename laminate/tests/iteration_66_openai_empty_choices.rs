//! Iteration 66 — OpenAI stream handler: empty choices array
//!
//! PASS: Handler gracefully handles empty/null/missing choices.
//! Usage metadata captured correctly when choices is empty.

use laminate::streaming::{FlexStream, Provider, StreamConfig, StreamEvent};

fn openai_stream() -> FlexStream {
    FlexStream::new(StreamConfig {
        provider: Provider::OpenAI,
        max_buffer_bytes: 1_048_576,
    })
}

#[test]
fn openai_empty_choices_produces_nothing() {
    let mut stream = openai_stream();
    let events = stream.feed_str("data: {\"id\":\"chatcmpl-1\",\"choices\":[]}\n\n");
    assert!(events.is_empty());
}

#[test]
fn openai_empty_choices_with_usage_produces_metadata() {
    let mut stream = openai_stream();
    let events = stream.feed_str(
        "data: {\"id\":\"chatcmpl-1\",\"choices\":[],\"usage\":{\"prompt_tokens\":10,\"completion_tokens\":5}}\n\n",
    );
    assert_eq!(events.len(), 1);
    assert!(matches!(&events[0], StreamEvent::Metadata(v) if v.has("usage")));
}

#[test]
fn openai_null_choices_graceful() {
    let mut stream = openai_stream();
    let events = stream.feed_str("data: {\"choices\":null}\n\n");
    assert!(events.is_empty());
}

#[test]
fn openai_empty_choices_then_normal_continues() {
    let mut stream = openai_stream();
    let events = stream.feed_str(
        "data: {\"choices\":[]}\n\n\
         data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hi\"}}]}\n\n",
    );
    assert_eq!(events.len(), 1);
    assert!(matches!(&events[0], StreamEvent::TextDelta(t) if t == "Hi"));
}
