// src/util/lock.rs
use anyhow::{bail, Context, Result};
use rusqlite::{Connection, OpenFlags};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::Duration;

const SQLITE_MAGIC: &[u8; 16] = b"SQLite format 3\0";

// Bounded enough to tolerate microsecond-scale checkpoint contention but fail
// fast on any real holder. Must be well below any real Anki transaction.
const PROBE_BUSY_TIMEOUT: Duration = Duration::from_millis(50);

/// Verify the collection SQLite file is not currently locked by another
/// process before we let the anki crate open it.
///
/// Strategy: open our own rusqlite connection and attempt `BEGIN EXCLUSIVE`.
/// EXCLUSIVE requires acquiring an EXCLUSIVE file lock — it fails with
/// SQLITE_BUSY if any other connection (reader or writer) holds the DB.
/// This is the same lock state machine SQLite (and therefore Anki) uses, so
/// it catches every holder regardless of process name, launcher, or shell.
///
/// On success, the probe releases the lock immediately via ROLLBACK; the
/// caller is expected to then open the DB via the anki crate, which will
/// take its own lock. The probe-to-open window is microseconds; any holder
/// racing into that window will be caught by the caller's error handling
/// (see `is_sqlite_busy_error` below).
pub fn check_collection_not_locked(path: &Path) -> Result<()> {
    verify_sqlite_magic(path)?;

    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .with_context(|| format!("Failed to open {} for lock probe", path.display()))?;

    conn.busy_timeout(PROBE_BUSY_TIMEOUT)
        .context("Failed to configure busy_timeout on probe connection")?;

    match conn.execute_batch("BEGIN EXCLUSIVE; ROLLBACK;") {
        Ok(()) => Ok(()),
        Err(e) if is_rusqlite_busy(&e) => bail!(locked_message(path)),
        Err(e) => Err(e).with_context(|| "Unexpected error probing collection lock"),
    }
}

/// Standardised user-facing error when the collection is held by another
/// process. Kept as a function so the call site in `infrastructure/anki.rs`
/// can emit the same message when CollectionBuilder hits a lock in the
/// TOCTOU window after the probe.
pub fn locked_message(path: &Path) -> String {
    format!(
        "Collection is locked by another process: {}\n\n\
         Close Anki (or any other process holding this file) completely,\n\
         wait a few seconds for locks to release, then retry.\n\n\
         This check prevents database corruption from concurrent writes.",
        path.display()
    )
}

/// Returns true if any error in the anyhow chain is a rusqlite
/// SQLITE_BUSY / SQLITE_LOCKED. Used at the AnkiRepository::new call site
/// to catch races between the probe and CollectionBuilder::build.
pub fn is_sqlite_busy_error(err: &anyhow::Error) -> bool {
    // Strongest signal: a typed rusqlite error anywhere in the chain.
    let typed_match = err.chain().any(|cause| {
        cause
            .downcast_ref::<rusqlite::Error>()
            .is_some_and(is_rusqlite_busy)
    });
    if typed_match {
        return true;
    }

    // Fallback when the upstream wraps rusqlite in an opaque type (e.g. the
    // anki crate splits `DbError` and `Locked` into separate chain nodes
    // whose Display renders jointly as "DbError: Locked"). Check both the
    // joined display and each node individually.
    if busy_text_signal(&format!("{:#}", err)) {
        return true;
    }
    err.chain().any(|cause| busy_text_signal(&cause.to_string()))
}

/// Textual fallback. Patterns are stable across SQLite versions and across
/// the anki crate's error display.
fn busy_text_signal(msg: &str) -> bool {
    let lower = msg.to_ascii_lowercase();
    // Raw SQLite messages
    lower.contains("database is locked")
        || lower.contains("database is busy")
        // anki crate wraps rusqlite::Error::SqliteFailure in its own DbError
        // whose Display renders as e.g. "DbError: Locked" / "DbError: Busy"
        || lower.contains("dberror: locked")
        || lower.contains("dberror: busy")
}

fn is_rusqlite_busy(e: &rusqlite::Error) -> bool {
    matches!(
        e,
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error {
                code: rusqlite::ErrorCode::DatabaseBusy,
                ..
            },
            _,
        ) | rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error {
                code: rusqlite::ErrorCode::DatabaseLocked,
                ..
            },
            _,
        )
    )
}

fn verify_sqlite_magic(path: &Path) -> Result<()> {
    let mut buf = [0u8; 16];
    let mut file = File::open(path)
        .with_context(|| format!("Cannot open {}", path.display()))?;
    file.read_exact(&mut buf)
        .with_context(|| format!("Cannot read SQLite header of {}", path.display()))?;
    if &buf != SQLITE_MAGIC {
        bail!(
            "{} is not a SQLite database (bad magic header)",
            path.display()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn fixture_collection_path() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/test_collection/User 1/collection.anki2")
    }

    fn copy_fixture_to_tempfile() -> NamedTempFile {
        let src = fixture_collection_path();
        let mut tmp = NamedTempFile::new().expect("tempfile");
        let bytes = std::fs::read(&src).expect("read fixture");
        tmp.write_all(&bytes).expect("write fixture bytes");
        tmp.flush().expect("flush");
        tmp
    }

    #[test]
    fn given_valid_unlocked_collection_when_probing_then_ok() {
        let tmp = copy_fixture_to_tempfile();
        check_collection_not_locked(tmp.path()).expect("unlocked probe must succeed");
    }

    #[test]
    fn given_nonexistent_file_when_probing_then_fails() {
        let result = check_collection_not_locked(Path::new("/nonexistent/does-not-exist.anki2"));
        assert!(result.is_err(), "probe on missing path must fail");
    }

    #[test]
    fn given_non_sqlite_file_when_probing_then_fails_with_magic_error() {
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(b"not a sqlite database at all").unwrap();
        tmp.flush().unwrap();

        let err = check_collection_not_locked(tmp.path()).unwrap_err();
        let msg = format!("{:#}", err);
        assert!(
            msg.contains("not a SQLite database"),
            "expected magic-byte error, got: {msg}"
        );
    }

    #[test]
    fn given_empty_file_when_probing_then_fails() {
        let tmp = NamedTempFile::new().unwrap();
        let result = check_collection_not_locked(tmp.path());
        assert!(result.is_err(), "empty file must not pass probe");
    }

    #[test]
    fn given_short_file_under_16_bytes_when_probing_then_fails() {
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(b"SQLite").unwrap(); // 6 bytes — below magic length
        tmp.flush().unwrap();

        let result = check_collection_not_locked(tmp.path());
        assert!(result.is_err(), "truncated header must not pass probe");
    }

    #[test]
    fn given_exclusive_lock_held_when_probing_then_fails_busy() {
        let tmp = copy_fixture_to_tempfile();
        let guard = Connection::open(tmp.path()).expect("open guard");
        guard
            .execute_batch("BEGIN EXCLUSIVE;")
            .expect("acquire exclusive");

        let err = check_collection_not_locked(tmp.path()).unwrap_err();
        let msg = format!("{:#}", err);
        assert!(
            msg.contains("locked by another process"),
            "expected lock error, got: {msg}"
        );

        // Explicitly release to make intent clear, even though Drop would also.
        guard.execute_batch("ROLLBACK;").ok();
    }

    #[test]
    fn given_reserved_lock_held_when_probing_then_fails_busy() {
        let tmp = copy_fixture_to_tempfile();
        let guard = Connection::open(tmp.path()).expect("open guard");
        guard
            .execute_batch("BEGIN IMMEDIATE;")
            .expect("acquire reserved");

        let err = check_collection_not_locked(tmp.path()).unwrap_err();
        let msg = format!("{:#}", err);
        assert!(
            msg.contains("locked by another process"),
            "expected lock error, got: {msg}"
        );

        guard.execute_batch("ROLLBACK;").ok();
    }

    #[test]
    fn given_probe_on_busy_db_when_probing_then_returns_within_busy_timeout() {
        let tmp = copy_fixture_to_tempfile();
        let guard = Connection::open(tmp.path()).expect("open guard");
        guard
            .execute_batch("BEGIN EXCLUSIVE;")
            .expect("acquire exclusive");

        let start = std::time::Instant::now();
        let _ = check_collection_not_locked(tmp.path());
        let elapsed = start.elapsed();
        assert!(
            elapsed < Duration::from_millis(500),
            "probe took too long under contention: {:?}",
            elapsed
        );

        guard.execute_batch("ROLLBACK;").ok();
    }

    #[test]
    fn given_lock_released_when_reprobing_then_succeeds() {
        let tmp = copy_fixture_to_tempfile();
        {
            let guard = Connection::open(tmp.path()).expect("open guard");
            guard
                .execute_batch("BEGIN EXCLUSIVE;")
                .expect("acquire exclusive");
            assert!(check_collection_not_locked(tmp.path()).is_err());
            guard.execute_batch("ROLLBACK;").expect("release");
        }
        check_collection_not_locked(tmp.path())
            .expect("probe must succeed after lock released");
    }

    #[test]
    fn given_rusqlite_busy_error_when_checking_is_busy_then_true() {
        let tmp = copy_fixture_to_tempfile();
        let guard = Connection::open(tmp.path()).expect("guard");
        guard.execute_batch("BEGIN EXCLUSIVE;").expect("lock");

        let probe = Connection::open_with_flags(
            tmp.path(),
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .unwrap();
        probe.busy_timeout(Duration::from_millis(10)).unwrap();

        let err = probe.execute_batch("BEGIN EXCLUSIVE;").unwrap_err();
        assert!(is_rusqlite_busy(&err));

        let anyhow_err: anyhow::Error = err.into();
        assert!(is_sqlite_busy_error(&anyhow_err));

        guard.execute_batch("ROLLBACK;").ok();
    }

    #[test]
    fn given_unrelated_error_when_checking_is_busy_then_false() {
        let err: anyhow::Error = anyhow::anyhow!("disk full");
        assert!(!is_sqlite_busy_error(&err));
    }

    #[test]
    fn given_text_signal_in_error_chain_when_checking_is_busy_then_true() {
        // Covers the case where the anki crate has wrapped the rusqlite error
        // in its own opaque type but the canonical SQLite message survives.
        let err: anyhow::Error = anyhow::anyhow!("failed: database is locked");
        assert!(is_sqlite_busy_error(&err));
    }

    #[test]
    fn given_anki_dberror_locked_in_chain_when_checking_is_busy_then_true() {
        // Matches the real wrapper the anki crate uses around rusqlite busy.
        let err: anyhow::Error = anyhow::anyhow!("upstream: DbError: Locked: something");
        assert!(is_sqlite_busy_error(&err));
    }

    #[test]
    fn given_anki_dberror_busy_in_chain_when_checking_is_busy_then_true() {
        let err: anyhow::Error = anyhow::anyhow!("upstream: DbError: Busy");
        assert!(is_sqlite_busy_error(&err));
    }
}
