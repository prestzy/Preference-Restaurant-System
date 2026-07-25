//! Safe replacement for small CSV/JSONL persistence files.
//!
//! Replacement writes must never truncate the last valid file before the new
//! payload is complete. This module writes and syncs a sibling temporary file,
//! keeps a short backup on Windows-compatible replacement paths, and restores
//! the backup if the final rename fails.

use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Replaces `target` only after `payload` has been fully written and synced.
pub(crate) fn replace_file_safely(target: &Path, payload: &[u8]) -> Result<()> {
    if let Some(parent) = target
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }

    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let file_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("application-data");
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let temporary = parent.join(format!(".{file_name}.{suffix}.tmp"));
    let backup = parent.join(format!(".{file_name}.{suffix}.bak"));

    let mut file = fs::File::create(&temporary)?;
    file.write_all(payload)?;
    file.sync_all()?;

    let had_target = target.exists();
    if had_target {
        fs::rename(target, &backup)?;
    }

    if let Err(error) = fs::rename(&temporary, target) {
        if had_target {
            let _ = fs::rename(&backup, target);
        }
        let _ = fs::remove_file(&temporary);
        return Err(error.into());
    }

    if had_target {
        let _ = fs::remove_file(backup);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replacement_writes_complete_payload() {
        let target = std::env::temp_dir().join(format!(
            "fyp-atomic-replace-{}-{}.txt",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should follow Unix epoch")
                .as_nanos()
        ));
        fs::write(&target, b"old").expect("fixture should be writable");

        replace_file_safely(&target, b"new complete payload").expect("replacement should succeed");

        assert_eq!(
            fs::read(&target).expect("replacement should be readable"),
            b"new complete payload"
        );
        let _ = fs::remove_file(target);
    }
}
