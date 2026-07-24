use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::path::Path;

pub const ORDER_DETAILS_PATH: &str = "data/order_details.csv";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrderDetailRecord {
    pub web_order_id: String,
    #[serde(default)]
    pub historical_order_id: String,
    pub customer_name: String,
    pub customer_phone: String,
    #[serde(default)]
    pub table_number: String,
    #[serde(default)]
    pub note: String,
    pub dish_ids: String,
    pub total: String,
    pub status: String,
    pub created_at: String,
}

pub fn load_order_details(path: &str) -> Result<Vec<OrderDetailRecord>> {
    if !Path::new(path).exists() {
        return Ok(Vec::new());
    }

    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_path(path)?;
    let mut records = Vec::new();
    for result in reader.deserialize() {
        records.push(result?);
    }
    Ok(records)
}

pub fn append_order_detail(record: &OrderDetailRecord, path: &str) -> Result<()> {
    ensure_parent(path)?;
    let has_content = Path::new(path).exists() && fs::metadata(path)?.len() > 0;
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    let mut writer = csv::WriterBuilder::new()
        .has_headers(!has_content)
        .from_writer(file);
    writer.serialize(record)?;
    writer.flush()?;
    Ok(())
}

pub fn rewrite_order_details(records: &[OrderDetailRecord], path: &str) -> Result<()> {
    ensure_parent(path)?;
    let mut writer = csv::Writer::from_path(path)?;
    for record in records {
        writer.serialize(record)?;
    }
    writer.flush()?;
    Ok(())
}

fn ensure_parent(path: &str) -> Result<()> {
    if let Some(parent) = Path::new(path)
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn order_details_round_trip() {
        let path = std::env::temp_dir().join(format!(
            "fyp_order_details_{}.csv",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let record = OrderDetailRecord {
            web_order_id: "WEB001".to_string(),
            customer_name: "Ali".to_string(),
            customer_phone: "60123456789".to_string(),
            dish_ids: "D01,D02".to_string(),
            total: "RM 20".to_string(),
            status: "Pending".to_string(),
            created_at: "2026-07-24 15:30".to_string(),
            ..OrderDetailRecord::default()
        };

        append_order_detail(&record, path.to_str().unwrap()).unwrap();
        let loaded = load_order_details(path.to_str().unwrap()).unwrap();
        let _ = fs::remove_file(path);

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].customer_phone, "60123456789");
    }
}
