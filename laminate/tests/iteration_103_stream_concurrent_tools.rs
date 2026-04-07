#![cfg(feature = "streaming")]
/// Iteration 103: FlexStream — two concurrent tool calls with interleaved fragments
///
/// Anthropic can stream multiple tool_use blocks simultaneously, with
/// fragments arriving interleaved across different block indices.
/// Does FlexStream correctly accumulate and assemble fragments per-index?
use laminate::streaming::{FlexStream, Provider, StreamConfig, StreamEvent};

#[test]
fn two_concurrent_tool_calls_interleaved() {
    let mut stream = FlexStream::new(StreamConfig {
        provider: Provider::Anthropic,
        ..Default::default()
    });

    // Stream: text → tool_use_0 start → tool_use_1 start → fragments interleaved → stops
    let sse_data = "\
event: message_start\n\
data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_x\",\"model\":\"claude-opus-4-6-20260301\",\"content\":[],\"stop_reason\":null,\"usage\":{\"input_tokens\":10,\"output_tokens\":0}}}\n\n\
event: content_block_start\n\
data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"tool_use\",\"id\":\"tu_A\",\"name\":\"search\",\"input\":{}}}\n\n\
event: content_block_start\n\
data: {\"type\":\"content_block_start\",\"index\":1,\"content_block\":{\"type\":\"tool_use\",\"id\":\"tu_B\",\"name\":\"lookup\",\"input\":{}}}\n\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"q\\\": \\\"\"}}\n\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"key\\\": \\\"\"}}\n\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"rust\\\"}\"}}\n\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"docs\\\"}\"}}\n\n\
event: content_block_stop\n\
data: {\"type\":\"content_block_stop\",\"index\":0}\n\n\
event: content_block_stop\n\
data: {\"type\":\"content_block_stop\",\"index\":1}\n\n\
event: message_delta\n\
data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\"},\"usage\":{\"output_tokens\":20}}\n\n";

    let events = stream.feed_str(sse_data);

    // Collect BlockComplete events
    let completes: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            StreamEvent::BlockComplete {
                index,
                id,
                name,
                content,
                ..
            } => Some((index, id.as_str(), name.as_deref(), content)),
            _ => None,
        })
        .collect();

    assert_eq!(completes.len(), 2, "should have 2 completed tool calls");

    // Tool A (index 0): search with q="rust"
    let (idx_a, id_a, name_a, content_a) = &completes[0];
    assert_eq!(**idx_a, 0);
    assert_eq!(*id_a, "tu_A");
    assert_eq!(*name_a, Some("search"));
    let q: String = content_a.extract("q").unwrap();
    assert_eq!(q, "rust", "tool A fragments should assemble correctly");

    // Tool B (index 1): lookup with key="docs"
    let (idx_b, id_b, name_b, content_b) = &completes[1];
    assert_eq!(**idx_b, 1);
    assert_eq!(*id_b, "tu_B");
    assert_eq!(*name_b, Some("lookup"));
    let key: String = content_b.extract("key").unwrap();
    assert_eq!(
        key, "docs",
        "tool B fragments should assemble independently"
    );
}
