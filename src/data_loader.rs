use crate::models::{Dish, DishRow, Order, OrderRow};
use anyhow::{Result, bail};
use std::fs::{self, OpenOptions};
use std::io::Read;
use std::path::Path;

/// Default CSV location for the menu data used by the prototype.
pub const DISHES_PATH: &str = "data/dishes.csv";

/// Default CSV location for the historical order data used by collaborative filtering.
pub const ORDERS_PATH: &str = "data/orders.csv";

/// Small fallback dish dataset written when `data/dishes.csv` is missing.
///
/// The prototype should run immediately after `cargo run`, so this embedded
/// sample prevents a missing-file error during demonstrations.
const SAMPLE_DISHES: &str = r#"dish_id,name,ingredients,category,tags
D01,Nasi Lemak,"rice,coconut milk,pandan,sambal,egg,anchovies,peanuts,cucumber",main,"spicy,malay,signature"
D02,Rendang Daging,"beef,coconut milk,kerisik,lemongrass,galangal,chili,garlic,ginger",main,"spicy,malay,beef"
D03,Ayam Masak Merah,"chicken,tomato,chili,onion,garlic,ginger,spices",main,"spicy,chicken,malay"
D04,Laksa,"noodles,fish,tamarind,laksa leaves,chili,onion,cucumber",main,"spicy,noodle,fish"
D05,Ketupat,"rice,coconut leaf",side,"rice,traditional,malay"
D06,Kuih Seri Muka,"glutinous rice,coconut milk,pandan,sugar,egg",dessert,"sweet,kuih,traditional"
D07,Sambal Sotong,"squid,chili,onion,tamarind,garlic,sambal",main,"spicy,seafood,malay"
D08,Sayur Lodeh,"cabbage,carrot,long beans,coconut milk,turmeric,tofu",main,"vegetarian,coconut,malay"
D09,Chicken Satay,"chicken,peanut sauce,lemongrass,turmeric,shallot,garlic",main,"grilled,spicy,signature"
D10,Beef Satay,"beef,peanut sauce,lemongrass,turmeric,shallot,garlic",main,"grilled,beef,malay"
"#;

/// Small fallback order log written when `data/orders.csv` is missing.
///
/// These orders create enough co-order relationships for the collaborative
/// filtering section to produce visible demo results.
const SAMPLE_ORDERS: &str = r#"order_id,session_user_id,ordered_dishes,timestamp
O001,U01,"D01,D03,D09",2026-01-01 12:30
O002,U02,"D02,D05,D08",2026-01-01 13:00
O003,U03,"D01,D09,D06",2026-01-02 12:20
O004,U04,"D04,D06",2026-01-02 13:15
O005,U05,"D03,D05,D08",2026-01-03 12:10
O006,U06,"D07,D01,D09",2026-01-03 13:40
O007,U07,"D10,D02,D05",2026-01-04 12:35
O008,U08,"D08,D05,D06",2026-01-04 14:00
"#;

/// Splits a comma-separated CSV field into lowercase values.
///
/// The same cleaning rule is used for ingredients and tags:
/// split by comma, trim whitespace, lowercase, and remove empty values.
pub fn split_csv_field(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|item| item.trim().to_lowercase())
        .filter(|item| !item.is_empty())
        .collect()
}

/// Creates the `data` folder and sample CSV files if they do not already exist.
///
/// Existing files are never overwritten. This lets the user replace the sample
/// CSVs with their own FYP dataset without losing work when the app starts.
pub fn generate_sample_data_if_missing() -> Result<()> {
    fs::create_dir_all("data")?;

    if !Path::new(DISHES_PATH).exists() {
        fs::write(DISHES_PATH, SAMPLE_DISHES)?;
    }

    if !Path::new(ORDERS_PATH).exists() {
        fs::write(ORDERS_PATH, SAMPLE_ORDERS)?;
    }

    Ok(())
}

/// Loads and cleans dish records from `data/dishes.csv`.
///
/// `DishRow` represents the raw CSV shape, while `Dish` is the cleaned model
/// used by the recommender. Dish IDs are uppercased so comparisons are stable.
pub fn load_dishes(path: &str) -> Result<Vec<Dish>> {
    let file = fs::File::open(path)?;
    parse_dishes_from_reader(file)
}

/// Parses dish records from any reader using the same rules as startup loading.
///
/// The web admin CSV import uses this function so imported data is cleaned in
/// exactly the same way as `data/dishes.csv`: IDs are uppercased, ingredients
/// and tags are split/lowercased, and optional image columns stay backward
/// compatible.
pub fn parse_dishes_from_reader<R: Read>(reader: R) -> Result<Vec<Dish>> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(reader);
    let headers = reader.headers()?.clone();
    validate_required_headers(
        &headers,
        &["dish_id", "name", "ingredients", "category", "tags"],
    )?;
    let mut dishes = Vec::new();

    for result in reader.records() {
        let record = result?;
        if record.iter().all(|field| field.trim().is_empty()) {
            continue;
        }

        let row: DishRow = record.deserialize(Some(&headers))?;

        dishes.push(clean_dish_row(row));
    }

    Ok(dishes)
}

/// Serializes cleaned dish models back into a CSV string.
///
/// This supports lightweight admin export without coupling the web handler to
/// CSV column details. Optional image fields are included so future datasets can
/// keep local image paths while older five-column CSV files still import fine.
pub fn dishes_to_csv(dishes: &[Dish]) -> Result<String> {
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer.write_record([
        "dish_id",
        "name",
        "ingredients",
        "category",
        "tags",
        "image_path",
        "image_source_url",
    ])?;

    for dish in dishes {
        let ingredients = dish.ingredients.join(",");
        let tags = dish.tags.join(",");
        let image_path = dish.image_path.clone().unwrap_or_default();
        let image_source_url = dish.image_source_url.clone().unwrap_or_default();
        writer.write_record([
            dish.dish_id.as_str(),
            dish.name.as_str(),
            ingredients.as_str(),
            dish.category.as_str(),
            tags.as_str(),
            image_path.as_str(),
            image_source_url.as_str(),
        ])?;
    }

    let bytes = writer.into_inner()?;
    Ok(String::from_utf8(bytes)?)
}

fn clean_dish_row(row: DishRow) -> Dish {
    Dish {
        dish_id: row.dish_id.trim().to_uppercase(),
        name: row.name.trim().to_string(),
        ingredients: split_csv_field(&row.ingredients),
        category: row.category.trim().to_lowercase(),
        tags: split_csv_field(&row.tags),
        // Optional image columns are intentionally not required in the CSV.
        // Empty strings are normalized to None so the image lookup can fall
        // back to assets/dishes/{dish_id}.jpg, .png, or .jpeg.
        image_path: clean_optional_field(row.image_path),
        image_source_url: clean_optional_field(row.image_source_url),
    }
}

/// Converts an optional CSV field into a clean optional string.
///
/// This small helper keeps CSV parsing tolerant: missing image columns and
/// blank image cells both become `None`, preserving older five-column datasets.
fn clean_optional_field(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// Loads and cleans order records from `data/orders.csv`.
///
/// Ordered dish IDs are split by comma and uppercased. The collaborative
/// filtering algorithm later uses these vectors to count item-item co-orders.
pub fn load_orders(path: &str) -> Result<Vec<Order>> {
    let file = fs::File::open(path)?;
    parse_orders_from_reader(file)
}

/// Parses order records from any reader using the same rules as startup loading.
///
/// Admin CSV import for historical order logs can use this without duplicating
/// co-ordering preparation logic in the web layer.
pub fn parse_orders_from_reader<R: Read>(reader: R) -> Result<Vec<Order>> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(reader);
    let headers = reader.headers()?.clone();
    validate_required_headers(
        &headers,
        &["order_id", "session_user_id", "ordered_dishes", "timestamp"],
    )?;
    let mut orders = Vec::new();

    for result in reader.records() {
        let record = result?;
        if record.iter().all(|field| field.trim().is_empty()) {
            continue;
        }

        let row: OrderRow = record.deserialize(Some(&headers))?;

        orders.push(clean_order_row(row));
    }

    Ok(orders)
}

/// Serializes order models into the CSV format used by the recommender.
pub fn orders_to_csv(orders: &[Order]) -> Result<String> {
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer.write_record(["order_id", "session_user_id", "ordered_dishes", "timestamp"])?;

    for order in orders {
        let ordered_dishes = order.ordered_dishes.join(",");
        writer.write_record([
            order.order_id.as_str(),
            order.session_user_id.as_str(),
            ordered_dishes.as_str(),
            order.timestamp.as_str(),
        ])?;
    }

    let bytes = writer.into_inner()?;
    Ok(String::from_utf8(bytes)?)
}

fn clean_order_row(row: OrderRow) -> Order {
    let ordered_dishes = row
        .ordered_dishes
        .split(',')
        .map(|dish_id| dish_id.trim().to_uppercase())
        .filter(|dish_id| !dish_id.is_empty())
        .collect();

    Order {
        order_id: row.order_id.trim().to_string(),
        session_user_id: row.session_user_id.trim().to_string(),
        ordered_dishes,
        timestamp: row.timestamp.trim().to_string(),
    }
}

/// Validates that a CSV file contains the columns required by the prototype.
///
/// Serde's missing-field error is technically correct, but it is not friendly
/// enough for restaurant staff. This helper produces a short admin-facing error
/// such as `CSV missing required column(s): ingredients, tags`.
fn validate_required_headers(headers: &csv::StringRecord, required: &[&str]) -> Result<()> {
    let normalized_headers = headers
        .iter()
        .map(|header| header.trim().to_lowercase())
        .collect::<Vec<_>>();

    let missing = required
        .iter()
        .filter(|column| !normalized_headers.iter().any(|header| header == **column))
        .copied()
        .collect::<Vec<_>>();

    if !missing.is_empty() {
        bail!("CSV missing required column(s): {}", missing.join(", "));
    }

    Ok(())
}

/// Appends a simulated order to `data/orders.csv`.
///
/// This is optional in the GUI. Keeping it separate from the in-memory update
/// makes the demo clear: one path changes behaviour only for the current app
/// session, while this path persists the new behavioural data.
#[allow(dead_code)]
pub fn append_order_to_csv(order: &Order, path: &str) -> Result<()> {
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    let mut writer = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(file);

    writer.write_record([
        order.order_id.as_str(),
        order.session_user_id.as_str(),
        order.ordered_dishes.join(",").as_str(),
        order.timestamp.as_str(),
    ])?;
    writer.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parses_older_five_column_dishes_csv() {
        let csv = r#"dish_id,name,ingredients,category,tags
D01,Nasi Lemak,"rice,coconut milk",main,"spicy,signature"
"#;

        let dishes = parse_dishes_from_reader(Cursor::new(csv)).expect("CSV should parse");

        assert_eq!(dishes.len(), 1);
        assert_eq!(dishes[0].dish_id, "D01");
        assert_eq!(dishes[0].ingredients, vec!["rice", "coconut milk"]);
        assert_eq!(dishes[0].image_path, None);
    }

    #[test]
    fn parses_image_aware_dishes_csv() {
        let csv = r#"dish_id,name,ingredients,category,tags,image_path,image_source_url
D09,Chicken Satay,"chicken,peanut sauce",main,"grilled,signature",assets/dishes/D09.jpg,https://example.test/satay
"#;

        let dishes = parse_dishes_from_reader(Cursor::new(csv)).expect("CSV should parse");

        assert_eq!(
            dishes[0].image_path.as_deref(),
            Some("assets/dishes/D09.jpg")
        );
        assert_eq!(
            dishes[0].image_source_url.as_deref(),
            Some("https://example.test/satay")
        );
    }

    #[test]
    fn exports_dishes_with_image_columns() {
        let dishes = vec![Dish {
            dish_id: "D01".to_string(),
            name: "Nasi Lemak".to_string(),
            ingredients: vec!["rice".to_string(), "egg".to_string()],
            category: "main".to_string(),
            tags: vec!["spicy".to_string()],
            image_path: Some("assets/dishes/D01.jpg".to_string()),
            image_source_url: None,
        }];

        let csv = dishes_to_csv(&dishes).expect("export should work");

        assert!(csv.contains("image_path"));
        assert!(csv.contains("assets/dishes/D01.jpg"));
    }

    #[test]
    fn parses_orders_for_collaborative_filtering() {
        let csv = r#"order_id,session_user_id,ordered_dishes,timestamp
O001,U01,"d01, D02",2026-01-01 12:30
"#;

        let orders = parse_orders_from_reader(Cursor::new(csv)).expect("CSV should parse");

        assert_eq!(orders[0].ordered_dishes, vec!["D01", "D02"]);
    }

    #[test]
    fn dish_csv_reports_missing_required_columns() {
        let csv = r#"dish_id,name,category
D01,Nasi Lemak,main
"#;

        let error = parse_dishes_from_reader(Cursor::new(csv))
            .expect_err("missing required columns should be reported");

        assert!(error.to_string().contains("ingredients"));
        assert!(error.to_string().contains("tags"));
    }

    #[test]
    fn order_csv_reports_missing_required_columns() {
        let csv = r#"order_id,session_user_id,timestamp
O001,U01,2026-01-01 12:30
"#;

        let error = parse_orders_from_reader(Cursor::new(csv))
            .expect_err("missing ordered_dishes should be reported");

        assert!(error.to_string().contains("ordered_dishes"));
    }
}
