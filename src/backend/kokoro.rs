//! Kokoro TTS backend — pure Rust ONNX inference via Python kokoro-onnx bridge.
//!
//! Cross-platform, no GPU required. Model files (~80MB) downloaded separately.

use std::process::Command;

use anyhow::{Context, Result};

use super::{SpeakOptions, TtsBackend};
use crate::audio;
use crate::config;

const MODEL_FILE: &str = "kokoro-v1.0.onnx";
const VOICES_FILE: &str = "voices-v1.0.bin";

pub struct KokoroBackend;

fn kokoro_dir() -> std::path::PathBuf {
    config::config_dir().join("kokoro")
}

fn model_path() -> std::path::PathBuf {
    kokoro_dir().join(MODEL_FILE)
}

fn voices_path() -> std::path::PathBuf {
    kokoro_dir().join(VOICES_FILE)
}

/// Map our short lang codes to kokoro-onnx lang codes.
fn map_lang(code: &str) -> &'static str {
    match code {
        "en" => "en-us",
        "fr" => "fr-fr",
        "ja" => "ja",
        "zh" => "zh-cn",
        "ko" => "ko",
        "hi" => "hi",
        "it" => "it",
        "pt" => "pt-br",
        "de" => "de",
        "es" => "es",
        _ => "en-us",
    }
}

/// Check if a voice name is a valid Kokoro voice (prefix pattern: af_, am_, bf_, etc.)
fn is_kokoro_voice(name: &str) -> bool {
    name.len() >= 3 && name.as_bytes()[2] == b'_'
}

/// Pick default voice based on language.
fn default_voice_for_lang(lang: &str) -> &'static str {
    match lang {
        "fr" => "ff_siwis",
        "ja" => "jf_alpha",
        "zh" => "zf_xiaoxiao",
        "hi" => "hf_alpha",
        "it" => "if_sara",
        "pt" => "pf_dora",
        _ => "af_heart",
    }
}

impl TtsBackend for KokoroBackend {
    fn name(&self) -> &str {
        "kokoro"
    }

    fn speak(&self, text: &str, opts: &SpeakOptions) -> Result<()> {
        let mp = model_path();
        let vp = voices_path();
        if !mp.exists() || !vp.exists() {
            anyhow::bail!(
                "Kokoro model not found. Download model files to {}:\n\
                 curl -L -o {} https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.0/kokoro-v1.0.onnx\n\
                 curl -L -o {} https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.0/voices-v1.0.bin",
                kokoro_dir().display(),
                mp.display(),
                vp.display(),
            );
        }

        let lang = opts.lang.as_deref().unwrap_or("en");
        let voice = opts
            .voice
            .as_deref()
            .filter(|v| is_kokoro_voice(v))
            .unwrap_or_else(|| default_voice_for_lang(lang));
        let kokoro_lang = map_lang(lang);

        let tmp = tempfile::NamedTempFile::new().context("failed to create temp file")?;
        let wav_path = tmp.path().with_extension("wav");
        let wav_str = wav_path.to_string_lossy().to_string();

        let script = format!(
            r#"
from kokoro_onnx import Kokoro
import soundfile as sf
kokoro = Kokoro("{model}", "{voices}")
samples, sr = kokoro.create("{text}", "{voice}", speed=1.0, lang="{lang}")
sf.write("{out}", samples, sr)
"#,
            model = mp.to_string_lossy().replace('"', r#"\""#),
            voices = vp.to_string_lossy().replace('"', r#"\""#),
            text = text.replace('"', r#"\""#).replace('\n', " "),
            voice = voice,
            lang = kokoro_lang,
            out = wav_str.replace('"', r#"\""#),
        );

        let output = Command::new("python3")
            .arg("-c")
            .arg(&script)
            .output()
            .context(
                "Failed to run kokoro-onnx. Is it installed? (pip install kokoro-onnx soundfile)",
            )?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Kokoro TTS failed: {stderr}");
        }

        audio::play_wav_blocking(&wav_path)?;
        let _ = std::fs::remove_file(&wav_path);
        Ok(())
    }

    fn list_voices(&self) -> Result<Vec<String>> {
        Ok(vec![
            // English
            "af_heart".into(),
            "af_bella".into(),
            "af_nova".into(),
            "af_sky".into(),
            "am_adam".into(),
            "am_michael".into(),
            "bf_emma".into(),
            "bm_george".into(),
            // French
            "ff_siwis".into(),
            // Japanese
            "jf_alpha".into(),
            "jm_kumo".into(),
            // Chinese
            "zf_xiaoxiao".into(),
            // Hindi
            "hf_alpha".into(),
            // Italian
            "if_sara".into(),
            // Portuguese
            "pf_dora".into(),
        ])
    }

    fn is_available(&self) -> bool {
        model_path().exists() && voices_path().exists()
    }
}
