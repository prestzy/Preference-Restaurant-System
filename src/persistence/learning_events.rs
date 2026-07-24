//! JSON Lines persistence for recommendation learning events.

use crate::recommender::learning_timeline::RecommendationLearningEvent;
use anyhow::{Context, Result};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub const LEARNING_EVENTS_PATH: &str = "data/recommendation_learning_events.jsonl";

pub fn load_learning_events(path: &str) -> Result<Vec<RecommendationLearningEvent>> {
    if !Path::new(path).exists() {
        return Ok(Vec::new());
    }
    let file =
        fs::File::open(path).with_context(|| format!("failed to open learning timeline {path}"))?;
    BufReader::new(file)
        .lines()
        .enumerate()
        .filter_map(|(index, line)| match line {
            Ok(line) if line.trim().is_empty() => None,
            Ok(line) => Some(
                serde_json::from_str(&line)
                    .with_context(|| format!("invalid timeline JSON on line {}", index + 1)),
            ),
            Err(error) => Some(Err(error.into())),
        })
        .collect()
}

/// Appends an event unless its durable historical order ID already exists.
pub fn append_learning_event(event: &RecommendationLearningEvent, path: &str) -> Result<bool> {
    let existing = load_learning_events(path)?;
    if existing
        .iter()
        .any(|item| item.historical_order_id == event.historical_order_id)
    {
        return Ok(false);
    }
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(&mut file, event)?;
    file.write_all(b"\n")?;
    file.flush()?;
    Ok(true)
}

/// Replaces the timeline only after every event has serialized successfully.
pub fn rewrite_learning_events(events: &[RecommendationLearningEvent], path: &str) -> Result<()> {
    let mut payload = Vec::new();
    for event in events {
        serde_json::to_writer(&mut payload, event)?;
        payload.push(b'\n');
    }
    let target = Path::new(path);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    replace_file_safely(target, &payload)
        .with_context(|| format!("failed to rewrite learning timeline {path}"))
}

/// Writes a complete replacement before moving it over the active timeline.
///
/// Windows cannot rename a new file over an existing destination. The short
/// backup step therefore keeps the previous valid file recoverable if the
/// final move fails. This helper is deliberately private to timeline
/// persistence and never receives the historical order CSV path.
fn replace_file_safely(target: &Path, payload: &[u8]) -> Result<()> {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let file_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("recommendation_learning_events.jsonl");
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
    use crate::recommender::learning_timeline::RecommendationLearningEvent;

    fn event(id: &str) -> RecommendationLearningEvent {
        RecommendationLearningEvent {
            event_id: format!("LEARN-{id}"),
            historical_order_id: id.to_string(),
            completed_at: "2026-01-01 10:00".to_string(),
            dish_ids: vec!["D01".to_string()],
            total_orders_before: 0,
            total_orders_after: 1,
            popularity_changes: vec![],
            pair_changes: vec![],
            rank_changes: vec![],
            summary: "Evidence changed.".to_string(),
        }
    }

    #[test]
    fn jsonl_append_is_idempotent_and_reloadable() {
        let path = std::env::temp_dir().join(format!(
            "fyp-learning-{}-{}.jsonl",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path_text = path.to_string_lossy();
        assert!(append_learning_event(&event("O001"), &path_text).unwrap());
        assert!(!append_learning_event(&event("O001"), &path_text).unwrap());
        assert_eq!(load_learning_events(&path_text).unwrap().len(), 1);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn rewrite_can_safely_clear_an_existing_timeline() {
        let path: std::path::PathBuf = std::env::temp_dir().join(format!(
            "fyp-learning-clear-{}-{}.jsonl",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path_text = path.to_string_lossy();
        rewrite_learning_events(&[event("O001"), event("O002")], &path_text).unwrap();
        assert_eq!(load_learning_events(&path_text).unwrap().len(), 2);

        rewrite_learning_events(&[], &path_text).unwrap();
        assert!(load_learning_events(&path_text).unwrap().is_empty());
        assert_eq!(fs::read_to_string(&path).unwrap(), "");
        let _ = fs::remove_file(path);
    }
}
