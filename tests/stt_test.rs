#![cfg(target_os = "macos")]

use std::ffi::OsStr;

use vox::stt;

#[test]
fn build_transcribe_command_basic() {
    let cmd = stt::build_transcribe_command("/tmp/audio.wav", None);
    assert_eq!(cmd.get_program(), "python3");
    let args: Vec<&OsStr> = cmd.get_args().collect();
    assert_eq!(args.len(), 2);
    assert_eq!(args[0], "-c");
    let script = args[1].to_string_lossy();
    assert!(script.contains("mlx_whisper"));
    assert!(script.contains("/tmp/audio.wav"));
    assert!(script.contains("language='en'")); // default language
}

#[test]
fn build_transcribe_command_with_language() {
    let cmd = stt::build_transcribe_command("/tmp/audio.wav", Some("fr"));
    let args: Vec<&OsStr> = cmd.get_args().collect();
    let script = args[1].to_string_lossy();
    assert!(script.contains("language='fr'"));
}

#[test]
fn build_transcribe_command_no_language_defaults_to_en() {
    let cmd = stt::build_transcribe_command("/tmp/audio.wav", None);
    let args: Vec<&OsStr> = cmd.get_args().collect();
    let script = args[1].to_string_lossy();
    assert!(script.contains("language='en'"));
}

#[test]
fn build_transcribe_command_preserves_audio_path() {
    let cmd = stt::build_transcribe_command("/home/user/recording.wav", None);
    let args: Vec<&OsStr> = cmd.get_args().collect();
    let script = args[1].to_string_lossy();
    assert!(script.contains("/home/user/recording.wav"));
}

#[test]
fn build_transcribe_command_uses_whisper_model() {
    let cmd = stt::build_transcribe_command("/tmp/a.wav", Some("ja"));
    let args: Vec<&OsStr> = cmd.get_args().collect();
    let script = args[1].to_string_lossy();
    assert!(script.contains("whisper-large-v3-turbo"));
    assert!(script.contains("language='ja'"));
}
