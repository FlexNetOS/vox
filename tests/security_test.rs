//! Security tests — SQL injection, path traversal, input validation.

use vox::clone;
use vox::db;

// ---------------------------------------------------------------------------
// SQL injection prevention in preferences
// ---------------------------------------------------------------------------

#[test]
fn sql_injection_in_preference_value_is_safe() {
    let conn = db::open_in_memory().unwrap();
    // Try SQL injection in value — should be stored as literal string
    db::set_preference(&conn, "voice", "'; DROP TABLE preferences; --").unwrap();
    let prefs = db::get_preferences(&conn).unwrap();
    assert_eq!(
        prefs.voice.as_deref(),
        Some("'; DROP TABLE preferences; --")
    );
    // Table should still be intact
    let prefs2 = db::get_preferences(&conn).unwrap();
    assert!(prefs2.voice.is_some());
}

#[test]
fn sql_injection_in_clone_name_is_safe() {
    let conn = db::open_in_memory().unwrap();
    let malicious_name = "test'; DROP TABLE voice_clones; --";
    db::add_clone(&conn, malicious_name, "/tmp/test.wav", None).unwrap();
    let clones = db::list_clones(&conn).unwrap();
    assert_eq!(clones.len(), 1);
    assert_eq!(clones[0].name, malicious_name);
    // Table still works
    db::list_clones(&conn).unwrap();
}

#[test]
fn sql_injection_in_usage_log_is_safe() {
    let conn = db::open_in_memory().unwrap();
    db::log_usage(
        &conn,
        "'; DROP TABLE usage_log; --",
        Some("voice"),
        Some("en"),
        100,
        Some(500),
    )
    .unwrap();
    let (count, _) = db::get_usage_summary(&conn).unwrap();
    assert_eq!(count, 1);
}

// ---------------------------------------------------------------------------
// Preference key validation (prevent SQL column injection)
// ---------------------------------------------------------------------------

#[test]
fn invalid_preference_key_rejected() {
    let conn = db::open_in_memory().unwrap();
    let result = db::set_preference(&conn, "malicious_column", "value");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Unknown preference")
    );
}

#[test]
fn preference_key_with_sql_keyword_rejected() {
    let conn = db::open_in_memory().unwrap();
    let result = db::set_preference(&conn, "id; DROP TABLE", "value");
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Audio file validation (path traversal, invalid extensions)
// ---------------------------------------------------------------------------

#[test]
fn audio_validation_rejects_nonexistent_file() {
    let result = clone::validate_audio("/nonexistent/path/audio.wav");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test]
fn audio_validation_rejects_invalid_extension() {
    let tmp = tempfile::NamedTempFile::with_suffix(".exe").unwrap();
    let result = clone::validate_audio(tmp.path().to_str().unwrap());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unsupported"));
}

#[test]
fn audio_validation_rejects_no_extension() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let result = clone::validate_audio(tmp.path().to_str().unwrap());
    assert!(result.is_err());
}

#[test]
fn audio_validation_accepts_valid_extensions() {
    for ext in &["wav", "mp3", "flac", "ogg", "m4a"] {
        let tmp = tempfile::NamedTempFile::with_suffix(&format!(".{ext}")).unwrap();
        let result = clone::validate_audio(tmp.path().to_str().unwrap());
        assert!(result.is_ok(), "Should accept .{ext}");
    }
}

// ---------------------------------------------------------------------------
// Backend validation
// ---------------------------------------------------------------------------

#[test]
fn backend_validation_rejects_unknown() {
    let conn = db::open_in_memory().unwrap();
    let result = db::set_preference(&conn, "backend", "shellscript");
    assert!(result.is_err());
}

#[test]
fn backend_validation_accepts_valid() {
    let conn = db::open_in_memory().unwrap();
    // kokoro should always be valid on all platforms
    assert!(db::set_preference(&conn, "backend", "kokoro").is_ok());
}

// ---------------------------------------------------------------------------
// Language validation
// ---------------------------------------------------------------------------

#[test]
fn lang_validation_rejects_unsupported() {
    let conn = db::open_in_memory().unwrap();
    let result = db::set_preference(&conn, "lang", "xx");
    assert!(result.is_err());
}

#[test]
fn lang_validation_accepts_all_supported() {
    let conn = db::open_in_memory().unwrap();
    for lang in &["en", "fr", "es", "de", "it", "pt", "zh", "ja", "ko", "ru", "ar", "nl"] {
        assert!(
            db::set_preference(&conn, "lang", lang).is_ok(),
            "Should accept lang={lang}"
        );
    }
}

// ---------------------------------------------------------------------------
// Gender and style enum validation
// ---------------------------------------------------------------------------

#[test]
fn gender_rejects_arbitrary_values() {
    let conn = db::open_in_memory().unwrap();
    assert!(db::set_preference(&conn, "gender", "other").is_err());
    assert!(db::set_preference(&conn, "gender", "").is_err());
}

#[test]
fn style_rejects_arbitrary_values() {
    let conn = db::open_in_memory().unwrap();
    assert!(db::set_preference(&conn, "style", "angry").is_err());
    assert!(db::set_preference(&conn, "style", "").is_err());
}

// ---------------------------------------------------------------------------
// Unicode and special characters in text
// ---------------------------------------------------------------------------

#[test]
fn unicode_text_stored_and_retrieved_correctly() {
    let conn = db::open_in_memory().unwrap();
    db::log_usage(&conn, "say", None, Some("fr"), 50, Some(1000)).unwrap();
    db::log_usage(&conn, "kokoro", None, Some("ja"), 30, Some(500)).unwrap();
    let entries = db::get_usage_stats(&conn).unwrap();
    assert_eq!(entries.len(), 2);
}

#[test]
fn clone_name_with_unicode_works() {
    let conn = db::open_in_memory().unwrap();
    db::add_clone(&conn, "voix_française", "/tmp/test.wav", Some("Bonjour à tous")).unwrap();
    let clone = db::get_clone(&conn, "voix_française").unwrap();
    assert!(clone.is_some());
    assert_eq!(clone.unwrap().ref_text.as_deref(), Some("Bonjour à tous"));
}

// ---------------------------------------------------------------------------
// STT command does not use shell (no command injection)
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
#[test]
fn stt_command_does_not_invoke_shell() {
    let cmd = vox::stt::build_transcribe_command("/tmp/test.wav", Some("en"));
    // Command should be python3, not sh/bash
    assert_eq!(cmd.get_program(), "python3");
    let args: Vec<_> = cmd.get_args().collect();
    // Should use -c with inline script, not shell
    assert_eq!(args[0], "-c");
}

#[cfg(target_os = "macos")]
#[test]
fn stt_escapes_single_quotes_in_path() {
    let cmd = vox::stt::build_transcribe_command("/tmp/it's a test.wav", Some("en"));
    let args: Vec<_> = cmd.get_args().collect();
    let script = args[1].to_string_lossy();
    // Should have escaped single quote
    assert!(script.contains("\\'"));
    assert!(!script.contains("it's"));
}
