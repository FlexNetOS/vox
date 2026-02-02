use std::process::Command;

use anyhow::{Context, Result};

pub fn build_transcribe_command(audio_path: &str, lang: Option<&str>) -> Command {
    let mut cmd = Command::new("python3");
    cmd.arg("-m").arg("mlx_audio.stt.generate");
    cmd.arg("--audio").arg(audio_path);
    cmd.arg("--format").arg("txt");
    if let Some(lang) = lang {
        cmd.arg("--language").arg(lang);
    }
    cmd
}

pub fn transcribe(audio_path: &str, lang: Option<&str>) -> Result<String> {
    let output = build_transcribe_command(audio_path, lang)
        .output()
        .context("Failed to run mlx_audio STT. Is mlx-audio installed? (pip install mlx-audio)")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("STT failed: {stderr}");
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(text)
}
