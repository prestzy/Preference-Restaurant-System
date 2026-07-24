//! JSON Lines persistence for recommendation learning events.

use crate::recommender::learning_timeline::RecommendationLearningEvent;
use anyhow::{Context, Result};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

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
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, payload).with_context(|| format!("failed to rewrite learning timeline {path}"))
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
}
