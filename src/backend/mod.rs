//! TTS backend abstraction layer.
//!
//! Each backend implements `TtsBackend` and is selected at runtime via `get_backend()`.
//! Platform-gated: `say` and `qwen` are macOS-only; `kokoro`, `qwen-native`, and `voxtream` are cross-platform.

pub mod kokoro;
pub mod piper;
#[cfg(target_os = "macos")]
pub mod qwen;
pub mod qwen_native;
#[cfg(target_os = "macos")]
pub mod say;
pub mod voxtream;

use anyhow::Result;

#[derive(Debug, Clone, Default)]
pub struct SpeakOptions {
    pub voice: Option<String>,
    pub lang: Option<String>,
    pub rate: Option<u32>,
    pub gender: Option<String>,
    pub style: Option<String>,
    pub ref_audio: Option<String>,
    pub ref_text: Option<String>,
    pub model: Option<String>,
}

pub trait TtsBackend {
    fn name(&self) -> &str;
    fn speak(&self, text: &str, opts: &SpeakOptions) -> Result<()>;
    fn list_voices(&self) -> Result<Vec<String>>;
    fn is_available(&self) -> bool;
}

pub fn get_backend(name: &str) -> Result<Box<dyn TtsBackend>> {
    match name {
        "kokoro" => Ok(Box::new(kokoro::KokoroBackend)),
        #[cfg(target_os = "macos")]
        "say" => Ok(Box::new(say::SayBackend)),
        #[cfg(target_os = "macos")]
        "qwen" => Ok(Box::new(qwen::QwenBackend)),
        "qwen-native" => Ok(Box::new(qwen_native::QwenNativeBackend)),
        "voxtream" => Ok(Box::new(voxtream::VoxtreamBackend)),
        "piper" => Ok(Box::new(piper::PiperBackend)),
        #[cfg(not(target_os = "macos"))]
        "say" | "qwen" => {
            anyhow::bail!("Backend '{name}' is only available on macOS. Use 'qwen-native' instead.")
        }
        _ => anyhow::bail!("Unknown backend: {name}"),
    }
}
