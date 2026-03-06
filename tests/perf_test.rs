//! Performance tests — DB operations, bulk operations, response times.

use vox::db;

// ---------------------------------------------------------------------------
// DB open performance (should be <100ms even with migrations)
// ---------------------------------------------------------------------------

#[test]
fn db_open_is_fast() {
    let start = std::time::Instant::now();
    let _conn = db::open_in_memory().unwrap();
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 100,
        "DB open took {}ms, expected <100ms",
        elapsed.as_millis()
    );
}

// ---------------------------------------------------------------------------
// Bulk usage logging (should handle 1000 inserts in <1s)
// ---------------------------------------------------------------------------

#[test]
fn bulk_usage_log_performance() {
    let conn = db::open_in_memory().unwrap();
    let start = std::time::Instant::now();
    for i in 0..1000 {
        db::log_usage(
            &conn,
            if i % 3 == 0 { "say" } else { "kokoro" },
            Some("default"),
            Some(if i % 2 == 0 { "fr" } else { "en" }),
            50 + (i % 200),
            Some(500 + (i as u64 % 3000)),
        )
        .unwrap();
    }
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 1000,
        "1000 inserts took {}ms, expected <1000ms",
        elapsed.as_millis()
    );
}

// ---------------------------------------------------------------------------
// Stats query performance after bulk data
// ---------------------------------------------------------------------------

#[test]
fn stats_query_fast_after_bulk_data() {
    let conn = db::open_in_memory().unwrap();

    // Seed 500 entries across multiple backends and languages
    for i in 0..500 {
        let backend = match i % 4 {
            0 => "say",
            1 => "kokoro",
            2 => "qwen",
            _ => "qwen-native",
        };
        let lang = match i % 5 {
            0 => "fr",
            1 => "en",
            2 => "ja",
            3 => "de",
            _ => "es",
        };
        db::log_usage(&conn, backend, None, Some(lang), 100, Some(1000)).unwrap();
    }

    // Summary query
    let start = std::time::Instant::now();
    let (count, total_chars) = db::get_usage_summary(&conn).unwrap();
    let elapsed = start.elapsed();
    assert_eq!(count, 500);
    assert_eq!(total_chars, 50000);
    assert!(
        elapsed.as_millis() < 50,
        "Summary query took {}ms, expected <50ms",
        elapsed.as_millis()
    );

    // Backend stats aggregation
    let start = std::time::Instant::now();
    let backend_stats = db::get_backend_stats(&conn).unwrap();
    let elapsed = start.elapsed();
    assert_eq!(backend_stats.len(), 4);
    assert!(
        elapsed.as_millis() < 50,
        "Backend stats took {}ms, expected <50ms",
        elapsed.as_millis()
    );

    // Language stats aggregation
    let start = std::time::Instant::now();
    let lang_stats = db::get_lang_stats(&conn).unwrap();
    let elapsed = start.elapsed();
    assert_eq!(lang_stats.len(), 5);
    assert!(
        elapsed.as_millis() < 50,
        "Lang stats took {}ms, expected <50ms",
        elapsed.as_millis()
    );

    // Total duration
    let start = std::time::Instant::now();
    let total_ms = db::get_total_duration_ms(&conn).unwrap();
    let elapsed = start.elapsed();
    assert_eq!(total_ms, 500_000);
    assert!(
        elapsed.as_millis() < 50,
        "Total duration took {}ms, expected <50ms",
        elapsed.as_millis()
    );
}

// ---------------------------------------------------------------------------
// Preference read/write cycle performance
// ---------------------------------------------------------------------------

#[test]
fn preference_read_write_cycle_fast() {
    let conn = db::open_in_memory().unwrap();
    let start = std::time::Instant::now();
    for _ in 0..100 {
        db::set_preference(&conn, "voice", "test_voice").unwrap();
        db::get_preferences(&conn).unwrap();
    }
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 500,
        "100 preference cycles took {}ms, expected <500ms",
        elapsed.as_millis()
    );
}

// ---------------------------------------------------------------------------
// Clone operations performance
// ---------------------------------------------------------------------------

#[test]
fn bulk_clone_operations() {
    let conn = db::open_in_memory().unwrap();

    // Add 50 clones
    let start = std::time::Instant::now();
    for i in 0..50 {
        db::add_clone(
            &conn,
            &format!("clone_{i}"),
            &format!("/tmp/audio_{i}.wav"),
            Some(&format!("Reference text {i}")),
        )
        .unwrap();
    }
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 500,
        "50 clone inserts took {}ms, expected <500ms",
        elapsed.as_millis()
    );

    // List all clones
    let start = std::time::Instant::now();
    let clones = db::list_clones(&conn).unwrap();
    let elapsed = start.elapsed();
    assert_eq!(clones.len(), 50);
    assert!(
        elapsed.as_millis() < 50,
        "List 50 clones took {}ms, expected <50ms",
        elapsed.as_millis()
    );

    // Lookup single clone
    let start = std::time::Instant::now();
    for i in 0..50 {
        db::get_clone(&conn, &format!("clone_{i}")).unwrap();
    }
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 100,
        "50 clone lookups took {}ms, expected <100ms",
        elapsed.as_millis()
    );
}

// ---------------------------------------------------------------------------
// Usage stats query with no data (no crash, fast)
// ---------------------------------------------------------------------------

#[test]
fn stats_on_empty_db_is_fast() {
    let conn = db::open_in_memory().unwrap();
    let start = std::time::Instant::now();
    let (count, chars) = db::get_usage_summary(&conn).unwrap();
    let backend = db::get_backend_stats(&conn).unwrap();
    let lang = db::get_lang_stats(&conn).unwrap();
    let dur = db::get_total_duration_ms(&conn).unwrap();
    let elapsed = start.elapsed();

    assert_eq!(count, 0);
    assert_eq!(chars, 0);
    assert!(backend.is_empty());
    assert!(lang.is_empty());
    assert_eq!(dur, 0);
    assert!(
        elapsed.as_millis() < 50,
        "Empty stats took {}ms, expected <50ms",
        elapsed.as_millis()
    );
}
