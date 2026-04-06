//! Piper TTS backend — fast local neural TTS via ONNX.
//!
//! 15-80MB models, <1s inference on CPU, 30+ languages.
//! Requires: `pip install piper-tts` and ONNX model files.

use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};

use super::{SpeakOptions, TtsBackend};
use crate::config;

pub struct PiperBackend;

/// Find the piper binary — check PATH first, then common venv locations.
pub fn find_piper() -> Option<PathBuf> {
    if let Ok(status) = Command::new("piper")
        .arg("--help")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
    {
        if status.success() {
            return Some(PathBuf::from("piper"));
        }
    }

    let candidates = [
        dirs::home_dir().map(|h| h.join(".local/venvs/piper/bin/piper")),
        dirs::home_dir().map(|h| h.join(".venvs/piper/bin/piper")),
    ];

    candidates
        .into_iter()
        .flatten()
        .find(|candidate| candidate.exists())
}

fn models_dir() -> PathBuf {
    config::config_dir().join("piper")
}

/// Map lang code to piper model name and download URL.
fn model_for_lang(lang: &str) -> (&'static str, &'static str) {
    match lang {
        "fr" => (
            "fr_FR-siwis-medium",
            "https://huggingface.co/rhasspy/piper-voices/resolve/main/fr/fr_FR/siwis/medium",
        ),
        "es" => (
            "es_ES-davefx-medium",
            "https://huggingface.co/rhasspy/piper-voices/resolve/main/es/es_ES/davefx/medium",
        ),
        "de" => (
            "de_DE-thorsten-medium",
            "https://huggingface.co/rhasspy/piper-voices/resolve/main/de/de_DE/thorsten/medium",
        ),
        "it" => (
            "it_IT-riccardo-x_low",
            "https://huggingface.co/rhasspy/piper-voices/resolve/main/it/it_IT/riccardo/x_low",
        ),
        "pt" => (
            "pt_BR-faber-medium",
            "https://huggingface.co/rhasspy/piper-voices/resolve/main/pt/pt_BR/faber/medium",
        ),
        "zh" => (
            "zh_CN-huayan-medium",
            "https://huggingface.co/rhasspy/piper-voices/resolve/main/zh/zh_CN/huayan/medium",
        ),
        "ja" => (
            "ja_JP-kokoro-medium",
            "https://huggingface.co/rhasspy/piper-voices/resolve/main/ja/ja_JP/kokoro/medium",
        ),
        "ko" => (
            "ko_KR-kss-medium",
            "https://huggingface.co/rhasspy/piper-voices/resolve/main/ko/ko_KR/kss/medium",
        ),
        "ru" => (
            "ru_RU-irina-medium",
            "https://huggingface.co/rhasspy/piper-voices/resolve/main/ru/ru_RU/irina/medium",
        ),
        _ => (
            "en_US-lessac-medium",
            "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/medium",
        ),
    }
}

/// Ensure model files exist, downloading if needed.
fn ensure_model(lang: &str) -> Result<PathBuf> {
    let (model_name, base_url) = model_for_lang(lang);
    let dir = models_dir();
    std::fs::create_dir_all(&dir).ok();

    let onnx_path = dir.join(format!("{model_name}.onnx"));
    let json_path = dir.join(format!("{model_name}.onnx.json"));

    if !onnx_path.exists() {
        eprintln!("Downloading piper model '{model_name}'...");
        let url = format!("{base_url}/{model_name}.onnx?download=true");
        let bytes = reqwest::blocking::get(&url)
            .and_then(|r| r.bytes())
            .with_context(|| format!("failed to download {model_name}.onnx"))?;
        std::fs::write(&onnx_path, &bytes)?;
    }

    if !json_path.exists() {
        let url = format!("{base_url}/{model_name}.onnx.json?download=true");
        let bytes = reqwest::blocking::get(&url)
            .and_then(|r| r.bytes())
            .with_context(|| format!("failed to download {model_name}.onnx.json"))?;
        std::fs::write(&json_path, &bytes)?;
    }

    Ok(onnx_path)
}

impl TtsBackend for PiperBackend {
    fn name(&self) -> &str {
        "piper"
    }

    fn speak(&self, text: &str, opts: &SpeakOptions) -> Result<()> {
        let bin = find_piper().context(
            "piper not found. Install it:\n\
             uv venv ~/.local/venvs/piper --python 3.11\n\
             uv pip install --python ~/.local/venvs/piper/bin/python piper-tts",
        )?;

        let lang = opts.lang.as_deref().unwrap_or("en");
        let model_path = ensure_model(lang)?;

        let tmp = tempfile::NamedTempFile::new().context("failed to create temp file")?;
        let wav_path = tmp.path().with_extension("wav");

        let mut cmd = Command::new(&bin);
        cmd.arg("--model").arg(&model_path);
        cmd.arg("-f").arg(&wav_path);

        if let Some(ref voice) = opts.voice {
            // Piper voices are speaker IDs (integers) for multi-speaker models
            if let Ok(speaker_id) = voice.parse::<u32>() {
                cmd.arg("-s").arg(speaker_id.to_string());
            }
        }

        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("failed to run piper")?;

        if let Some(ref mut stdin) = child.stdin {
            use std::io::Write;
            stdin.write_all(text.as_bytes()).ok();
        }
        drop(child.stdin.take());

        let status = child.wait().context("piper process failed")?;
        if !status.success() {
            anyhow::bail!("piper exited with status {status}");
        }

        // Play audio
        #[cfg(target_os = "macos")]
        {
            Command::new("afplay")
                .arg(&wav_path)
                .status()
                .context("failed to play audio")?;
        }
        #[cfg(not(target_os = "macos"))]
        {
            crate::audio::play_wav_blocking(&wav_path)?;
        }

        let _ = std::fs::remove_file(&wav_path);
        Ok(())
    }

    fn list_voices(&self) -> Result<Vec<String>> {
        Ok(vec![
            "en_US-lessac-medium".into(),
            "fr_FR-siwis-medium".into(),
            "es_ES-davefx-medium".into(),
            "de_DE-thorsten-medium".into(),
            "it_IT-riccardo-x_low".into(),
            "pt_BR-faber-medium".into(),
            "zh_CN-huayan-medium".into(),
            "ja_JP-kokoro-medium".into(),
            "ko_KR-kss-medium".into(),
            "ru_RU-irina-medium".into(),
        ])
    }

    fn is_available(&self) -> bool {
        find_piper().is_some()
    }
}
