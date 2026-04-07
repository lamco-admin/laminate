#![cfg(feature = "streaming")]
/// Iteration 79: Full FlexStream — Anthropic tool_use with 3 argument fragments
///
/// Simulates a realistic Anthropic streaming sequence where tool_use arguments
/// arrive as 3 separate `input_json_delta` chunks that must be assembled into
/// valid JSON by FlexStream.
///
/// Stream sequence:
///   message_start → content_block_start(tool_use) → 3x content_block_delta(input_json_delta)
///   → content_block_stop → message_delta(stop_reason:tool_use) → message_stop
use laminate::streaming::{FlexStream, Provider, StreamConfig, StreamEvent};

#[test]
fn anthropic_tool_use_fragment_assembly() {
    let mut stream = FlexStream::new(StreamConfig {
        provider: Provider::Anthropic,
        ..Default::default()
    });

    // Full SSE stream for a tool_use response with fragmented arguments
    let sse_data = "\
event: message_start\n\
data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_abc\",\"type\":\"message\",\"role\":\"assistant\",\"model\":\"claude-opus-4-6-20260301\",\"content\":[],\"stop_reason\":null,\"usage\":{\"input_tokens\":50,\"output_tokens\":0}}}\n\n\
event: content_block_start\n\
data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_01XYZ\",\"name\":\"get_weather\",\"input\":{}}}\n\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"city\\\": \\\"Lon\"}}\n\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"don\\\", \\\"uni\"}}\n\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"ts\\\": \\\"celsius\\\"}\"}}\n\n\
event: content_block_stop\n\
data: {\"type\":\"content_block_stop\",\"index\":0}\n\n\
event: message_delta\n\
data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\"},\"usage\":{\"output_tokens\":42}}\n\n\
event: message_stop\n\
data: {\"type\":\"message_stop\"}\n\n";

    let events = stream.feed_str(sse_data);

    // Collect events by type for inspection
    let block_starts: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, StreamEvent::BlockStart { .. }))
        .collect();
    let block_deltas: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, StreamEvent::BlockDelta { .. }))
        .collect();
    let block_completes: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, StreamEvent::BlockComplete { .. }))
        .collect();

    // Should have one tool_use block start
    assert_eq!(block_starts.len(), 1, "expected 1 BlockStart");
    match &block_starts[0] {
        StreamEvent::BlockStart {
            block_type,
            name,
            id,
            ..
        } => {
            assert_eq!(block_type, "tool_use");
            assert_eq!(name, &Some("get_weather".to_string()));
            assert_eq!(id, "toolu_01XYZ");
        }
        _ => unreachable!(),
    }

    // Should have 3 fragment deltas
    assert_eq!(block_deltas.len(), 3, "expected 3 BlockDelta fragments");

    // Should have one BlockComplete — THIS IS THE KEY ASSERTION:
    // The assembled content should be valid JSON with city=London, units=celsius
    assert_eq!(block_completes.len(), 1, "expected 1 BlockComplete");
    match &block_completes[0] {
        StreamEvent::BlockComplete {
            index,
            id,
            block_type,
            name,
            content,
        } => {
            assert_eq!(*index, 0);
            // The BlockComplete should carry the assembled tool info
            assert_eq!(block_type, "tool_use", "block_type should be tool_use");
            assert_eq!(id, "toolu_01XYZ", "id should be preserved from BlockStart");
            assert_eq!(
                name,
                &Some("get_weather".to_string()),
                "name should be preserved from BlockStart"
            );

            // The content should be the assembled JSON from the 3 fragments
            let city: Option<String> = content.extract::<String>("city").ok();
            let units: Option<String> = content.extract::<String>("units").ok();
            assert_eq!(
                city.as_deref(),
                Some("London"),
                "assembled JSON should have city=London, got content: {:?}",
                content
            );
            assert_eq!(
                units.as_deref(),
                Some("celsius"),
                "assembled JSON should have units=celsius, got content: {:?}",
                content
            );
        }
        _ => unreachable!(),
    }
}
