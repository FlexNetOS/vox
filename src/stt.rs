//! Speech-to-text via mlx-whisper (macOS only).
//!
//! Runs `mlx_whisper.transcribe()` in a Python subprocess, returns the text as a String.

use std::process::Command;

use anyhow::{Context, Result};

/// Use mlx-whisper (simpler, more stable than mlx_audio.stt) for transcription.
/// Outputs JSON to stdout, we extract the "text" field.
pub fn build_transcribe_command(audio_path: &str, lang: Option<&str>) -> Command {
    let lang_arg = lang.unwrap_or("en");
    let model = "mlx-community/whisper-large-v3-turbo";
    let script = format!(
        "import mlx_whisper, json; r = mlx_whisper.transcribe('{}', path_or_hf_repo='{}', language='{}'); print(json.dumps({{'text': r.get('text', '')}}))",
        audio_path.replace('\'', "\\'"),
        model,
        lang_arg,
    );
    let mut cmd = Command::new("python3");
    cmd.arg("-c").arg(script);
    cmd
}

pub fn transcribe(audio_path: &str, lang: Option<&str>) -> Result<String> {
    let output = build_transcribe_command(audio_path, lang)
        .output()
        .context(
            "Failed to run mlx-whisper STT. Is mlx-whisper installed? (pip install mlx-whisper)",
        )?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("STT failed: {stderr}");
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Parse JSON output to get text
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).context("Failed to parse STT JSON output")?;
    let text = parsed["text"].as_str().unwrap_or("").trim().to_string();
    Ok(text)
}
