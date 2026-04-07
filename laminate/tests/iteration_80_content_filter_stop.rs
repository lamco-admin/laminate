#![cfg(feature = "streaming")]
use laminate::diagnostic::StopReason;
/// Iteration 80: Full FlexStream — OpenAI stream where finish_reason is "content_filter"
///
/// Simulates a realistic OpenAI stream that delivers partial text and then
/// terminates abruptly with finish_reason "content_filter" (not a standard
/// StopReason variant). Tests that:
/// 1. Text deltas received before the filter are preserved
/// 2. The unknown finish_reason is properly captured
/// 3. The [DONE] sentinel still works after the filter event
use laminate::streaming::{FlexStream, Provider, StreamConfig, StreamEvent};

#[test]
fn openai_content_filter_finish_reason() {
    let mut stream = FlexStream::new(StreamConfig {
        provider: Provider::OpenAI,
        ..Default::default()
    });

    // OpenAI stream: 2 text deltas, then content_filter finish, then [DONE]
    let sse_data = "\
data: {\"id\":\"chatcmpl-abc\",\"object\":\"chat.completion.chunk\",\"model\":\"gpt-4o\",\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":\"\"},\"finish_reason\":null}]}\n\n\
data: {\"id\":\"chatcmpl-abc\",\"object\":\"chat.completion.chunk\",\"model\":\"gpt-4o\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Here is how to \"},\"finish_reason\":null}]}\n\n\
data: {\"id\":\"chatcmpl-abc\",\"object\":\"chat.completion.chunk\",\"model\":\"gpt-4o\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"make a \"},\"finish_reason\":null}]}\n\n\
data: {\"id\":\"chatcmpl-abc\",\"object\":\"chat.completion.chunk\",\"model\":\"gpt-4o\",\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"content_filter\"}]}\n\n\
data: [DONE]\n\n";

    let events = stream.feed_str(sse_data);

    // Collect text deltas
    let text_deltas: Vec<&str> = events
        .iter()
        .filter_map(|e| match e {
            StreamEvent::TextDelta(t) => Some(t.as_str()),
            _ => None,
        })
        .collect();

    // Collect stop events
    let stop_events: Vec<&StopReason> = events
        .iter()
        .filter_map(|e| match e {
            StreamEvent::Stop(reason) => Some(reason),
            _ => None,
        })
        .collect();

    // Text deltas before the filter should be preserved
    assert_eq!(
        text_deltas,
        vec!["Here is how to ", "make a "],
        "text deltas before content_filter should be preserved"
    );

    // Should have exactly 2 stops: one for content_filter, one for [DONE]
    assert_eq!(
        stop_events.len(),
        2,
        "expected 2 Stop events (content_filter + DONE), got {:?}",
        stop_events
    );

    // First stop should be Unknown("content_filter")
    match stop_events[0] {
        StopReason::Unknown(reason) => {
            assert_eq!(
                reason, "content_filter",
                "first stop should capture content_filter as Unknown"
            );
        }
        other => panic!("expected Unknown(\"content_filter\"), got {:?}", other),
    }

    // Second stop (from [DONE]) should be EndTurn
    assert!(
        matches!(stop_events[1], StopReason::EndTurn),
        "DONE sentinel should produce EndTurn, got {:?}",
        stop_events[1]
    );
}
