use crate::models::{Dish, DishRow, Order, OrderRow};
use anyhow::Result;
use std::fs::{self, OpenOptions};
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
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_path(path)?;
    let headers = reader.headers()?.clone();
    let mut dishes = Vec::new();

    for result in reader.records() {
        let record = result?;
        if record.iter().all(|field| field.trim().is_empty()) {
            continue;
        }

        let row: DishRow = record.deserialize(Some(&headers))?;

        let dish = Dish {
            dish_id: row.dish_id.trim().to_uppercase(),
            name: row.name.trim().to_string(),
            ingredients: split_csv_field(&row.ingredients),
            category: row.category.trim().to_lowercase(),
            tags: split_csv_field(&row.tags),
            // Optional image columns are intentionally not required in the CSV.
            // Empty strings are normalized to None so the image loader can fall
            // back to assets/dishes/{dish_id}.jpg, .png, or .jpeg.
            image_path: clean_optional_field(row.image_path),
            image_source_url: clean_optional_field(row.image_source_url),
        };

        dishes.push(dish);
    }

    Ok(dishes)
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
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_path(path)?;
    let headers = reader.headers()?.clone();
    let mut orders = Vec::new();

    for result in reader.records() {
        let record = result?;
        if record.iter().all(|field| field.trim().is_empty()) {
            continue;
        }

        let row: OrderRow = record.deserialize(Some(&headers))?;

        let ordered_dishes = row
            .ordered_dishes
            .split(',')
            .map(|dish_id| dish_id.trim().to_uppercase())
            .filter(|dish_id| !dish_id.is_empty())
            .collect();

        let order = Order {
            order_id: row.order_id.trim().to_string(),
            session_user_id: row.session_user_id.trim().to_string(),
            ordered_dishes,
            timestamp: row.timestamp.trim().to_string(),
        };

        orders.push(order);
    }

    Ok(orders)
}

/// Appends a simulated order to `data/orders.csv`.
///
/// This is optional in the GUI. Keeping it separate from the in-memory update
/// makes the demo clear: one path changes behaviour only for the current app
/// session, while this path persists the new behavioural data.
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
