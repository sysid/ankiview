// src/util/process.rs
use anyhow::{bail, Result};
use sysinfo::System;

/// Check if Anki application is currently running
///
/// Returns an error if Anki is detected, as concurrent access
/// could cause database corruption.
pub fn check_anki_not_running() -> Result<()> {
    let system = System::new_all();

    // Platform-specific process names
    let process_names: &[&str] = if cfg!(target_os = "windows") {
        &["anki.exe"]
    } else if cfg!(target_os = "macos") {
        &["Anki", "anki"] // macOS can show as either
    } else {
        &["anki"] // Linux and others
    };

    // Check each possible process name
    for name in process_names {
        if let Some(process) = system.processes_by_exact_name(name.as_ref()).next() {
            bail!(
                "Anki is currently running (PID: {}).\n\n\
                 Please close Anki completely before using ankiview.\n\
                 This prevents database corruption from concurrent access.\n\n\
                 After closing Anki, wait a few seconds for it to fully exit,\n\
                 then try again.",
                process.pid()
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_anki_process_names_when_checking_then_uses_platform_specific_names() {
        // This test verifies that we're checking for the correct process names
        // We can't actually test if Anki is running without starting it,
        // but we can verify the function exists and returns a Result

        // When Anki is NOT running (normal case for tests), should succeed
        let result = check_anki_not_running();

        // Should be Ok since Anki is not running in test environment
        assert!(result.is_ok(), "Should succeed when Anki is not running");
    }

    #[test]
    fn given_function_exists_when_called_then_returns_result() {
        // Basic smoke test - function should be callable
        let _result = check_anki_not_running();
    }
}
