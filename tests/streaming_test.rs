#![cfg(target_os = "macos")]

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use vox::chat::claude_api::StreamEvent;
use vox::chat::sentence::SentenceAccumulator;

/// Simulate Claude SSE streaming: sends text deltas with realistic timing,
/// then verifies the sentence accumulator + channel pipeline works correctly.
#[test]
fn test_streaming_pipeline_mock() {
    // Simulate Claude streaming text token by token
    let (claude_tx, claude_rx) = mpsc::channel::<StreamEvent>();
    let (tts_tx, tts_rx) = mpsc::channel::<String>();

    // Simulated Claude response (2 sentences, realistic length)
    let response = "Bonjour ! Je suis un assistant vocal et je suis là pour t'aider. \
                    N'hésite pas à me poser toutes tes questions, je ferai de mon mieux \
                    pour y répondre de manière claire et concise.";

    // Thread 1: Simulate Claude streaming (token by token, ~20ms per token)
    let response_owned = response.to_string();
    let claude_thread = thread::spawn(move || {
        // Send tokens in small chunks (simulating SSE deltas)
        for chunk in response_owned.split_inclusive(' ') {
            claude_tx
                .send(StreamEvent::TextDelta(chunk.to_string()))
                .unwrap();
            thread::sleep(Duration::from_millis(15));
        }
        claude_tx.send(StreamEvent::Done).unwrap();
    });

    // Thread 2: Accumulate sentences (like the main thread in streaming.rs)
    let accumulator_thread = thread::spawn(move || {
        let mut accumulator = SentenceAccumulator::new();
        let mut full_reply = String::new();
        let mut sentence_times: Vec<(String, Duration)> = Vec::new();
        let start = Instant::now();

        for event in claude_rx {
            match event {
                StreamEvent::TextDelta(text) => {
                    full_reply.push_str(&text);
                    for sentence in accumulator.push(&text) {
                        sentence_times.push((sentence.clone(), start.elapsed()));
                        tts_tx.send(sentence).unwrap();
                    }
                }
                StreamEvent::Done => {
                    if let Some(remaining) = accumulator.flush() {
                        sentence_times.push((remaining.clone(), start.elapsed()));
                        tts_tx.send(remaining).unwrap();
                    }
                    break;
                }
            }
        }
        drop(tts_tx);
        (full_reply, sentence_times)
    });

    // Collect TTS commands (simulating the TTS thread)
    let tts_sentences: Vec<String> = tts_rx.iter().collect();

    claude_thread.join().unwrap();
    let (full_reply, sentence_times) = accumulator_thread.join().unwrap();

    // Verify: full reply is reconstructed correctly
    assert_eq!(full_reply, response);

    // Verify: we got sentences (at least 1 emitted before Done)
    assert!(
        !tts_sentences.is_empty(),
        "Should have emitted at least one sentence"
    );

    // Verify: all sentences concatenated equal the original text
    let reconstructed: String = tts_sentences.join(" ");
    // Normalize whitespace for comparison
    let normalized_original: String = response.split_whitespace().collect::<Vec<_>>().join(" ");
    let normalized_reconstructed: String = reconstructed
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert_eq!(normalized_reconstructed, normalized_original);

    // Print timing info for manual inspection
    for (sentence, elapsed) in &sentence_times {
        let preview: String = sentence.chars().take(60).collect();
        eprintln!("  [{:>4}ms] {preview}...", elapsed.as_millis());
    }
    eprintln!("  Total sentences emitted: {}", tts_sentences.len());
}

/// Test that short responses (single sentence) are flushed correctly.
#[test]
fn test_streaming_pipeline_short_response() {
    let (claude_tx, claude_rx) = mpsc::channel::<StreamEvent>();
    let (tts_tx, tts_rx) = mpsc::channel::<String>();

    // Short response that won't hit the 120-char threshold
    let response = "Bonjour !";

    let response_owned = response.to_string();
    let claude_thread = thread::spawn(move || {
        claude_tx
            .send(StreamEvent::TextDelta(response_owned))
            .unwrap();
        claude_tx.send(StreamEvent::Done).unwrap();
    });

    let accumulator_thread = thread::spawn(move || {
        let mut accumulator = SentenceAccumulator::new();
        for event in claude_rx {
            match event {
                StreamEvent::TextDelta(text) => {
                    for sentence in accumulator.push(&text) {
                        tts_tx.send(sentence).unwrap();
                    }
                }
                StreamEvent::Done => {
                    if let Some(remaining) = accumulator.flush() {
                        tts_tx.send(remaining).unwrap();
                    }
                    break;
                }
            }
        }
        drop(tts_tx);
    });

    let tts_sentences: Vec<String> = tts_rx.iter().collect();
    claude_thread.join().unwrap();
    accumulator_thread.join().unwrap();

    // Short response should be flushed as a single sentence
    assert_eq!(tts_sentences.len(), 1);
    assert_eq!(tts_sentences[0], "Bonjour !");
}

/// Test that the pipeline handles empty response gracefully.
#[test]
fn test_streaming_pipeline_empty_response() {
    let (claude_tx, claude_rx) = mpsc::channel::<StreamEvent>();
    let (tts_tx, tts_rx) = mpsc::channel::<String>();

    let claude_thread = thread::spawn(move || {
        claude_tx.send(StreamEvent::Done).unwrap();
    });

    let accumulator_thread = thread::spawn(move || {
        let mut accumulator = SentenceAccumulator::new();
        for event in claude_rx {
            match event {
                StreamEvent::TextDelta(text) => {
                    for sentence in accumulator.push(&text) {
                        tts_tx.send(sentence).unwrap();
                    }
                }
                StreamEvent::Done => {
                    if let Some(remaining) = accumulator.flush() {
                        tts_tx.send(remaining).unwrap();
                    }
                    break;
                }
            }
        }
        drop(tts_tx);
    });

    let tts_sentences: Vec<String> = tts_rx.iter().collect();
    claude_thread.join().unwrap();
    accumulator_thread.join().unwrap();

    assert!(tts_sentences.is_empty());
}
