#![cfg(feature = "streaming")]
/// MessageSnapshot — running message state during streaming.
use laminate::streaming::{FlexStream, Provider, StreamConfig};

#[test]
fn snapshot_accumulates_text() {
    let mut stream = FlexStream::new(StreamConfig {
        provider: Provider::Anthropic,
        ..Default::default()
    });

    // Two text deltas
    let sse = "\
event: message_start\n\
data: {\"type\":\"message_start\",\"message\":{\"id\":\"m1\",\"model\":\"claude\",\"content\":[],\"stop_reason\":null,\"usage\":{\"input_tokens\":10,\"output_tokens\":0}}}\n\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello \"}}\n\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"world!\"}}\n\n";

    stream.feed_str(sse);

    let snap = stream.current_message();
    assert_eq!(snap.text, "Hello world!");
    assert!(!snap.done);
    assert!(snap.stop_reason.is_none());
}

#[test]
fn snapshot_captures_tool_calls_and_stop() {
    let mut stream = FlexStream::new(StreamConfig {
        provider: Provider::Anthropic,
        ..Default::default()
    });

    let sse = "\
event: message_start\n\
data: {\"type\":\"message_start\",\"message\":{\"id\":\"m2\",\"model\":\"claude\",\"content\":[],\"stop_reason\":null,\"usage\":{\"input_tokens\":10,\"output_tokens\":0}}}\n\n\
event: content_block_start\n\
data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"tool_use\",\"id\":\"tu_1\",\"name\":\"search\",\"input\":{}}}\n\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"q\\\": \\\"rust\\\"}\"}}\n\n\
event: content_block_stop\n\
data: {\"type\":\"content_block_stop\",\"index\":0}\n\n\
event: message_delta\n\
data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\"},\"usage\":{\"output_tokens\":20}}\n\n";

    stream.feed_str(sse);

    let snap = stream.current_message();
    assert_eq!(snap.tool_calls.len(), 1);
    assert_eq!(snap.tool_calls[0].1, "search"); // name
    let q: String = snap.tool_calls[0].2.extract("q").unwrap();
    assert_eq!(q, "rust");
    assert!(snap.done);
}
