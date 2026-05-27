# Data Format

The app uses CSV files so the dataset is easy to edit for FYP demonstrations.

## Folder Structure

```text
data/
  dishes.csv
  orders.csv
assets/
  dish_image_sources.csv
  dishes/
```

If the CSV files are missing, the app creates sample data automatically.

## `data/dishes.csv`

Required columns:

```csv
dish_id,name,ingredients,category,tags
```

Optional image columns:

```csv
image_path,image_source_url
```

Older CSV files with only the five required columns still work. Missing optional
image fields are treated as blank.

Example:

```csv
D01,Nasi Lemak,"rice,coconut milk,pandan,sambal,egg,anchovies,peanuts,cucumber",main,"spicy,malay,signature"
```

### Cleaning Rules

When dishes are loaded:

- `dish_id` is trimmed and converted to uppercase.
- `name` is trimmed.
- `ingredients` are split by comma, trimmed, lowercased, and empty values are removed.
- `category` is trimmed and lowercased.
- `tags` are split by comma, trimmed, lowercased, and empty values are removed.
- `image_path` is trimmed when present; blank or missing values become empty optional values.
- `image_source_url` is trimmed when present and kept only for source traceability.

### Dish Image Lookup

Runtime image loading is local only. The app does not download or hotlink images
while the GUI is running.

For each dish, image lookup uses this order:

1. Use `image_path` from `data/dishes.csv` if the optional column exists and the file exists.
2. Try `assets/dishes/{dish_id}.jpg`.
3. Try `assets/dishes/{dish_id}.png`.
4. Try `assets/dishes/{dish_id}.jpeg`.
5. Show a `No image` placeholder if no local image exists.

Examples:

```text
assets/dishes/D01.jpg
assets/dishes/D02.png
assets/dishes/D03.jpeg
```

Images are displayed only in the customer-facing menu cards on **Explore &
Recommend** and in recommendation cards on **Evaluation**.

## `assets/dish_image_sources.csv`

This file records where local images came from.

Required columns:

```csv
dish_id,dish_name,image_url,license,source_page,local_path
```

Example:

```csv
D01,Nasi Lemak,https://upload.wikimedia.org/example.jpg,CC BY 4.0,https://commons.wikimedia.org/wiki/File:Example.jpg,assets/dishes/D01.jpg
```

If a dish has no image yet, leave `local_path` blank or mark the source fields as
`not downloaded`.

## `data/orders.csv`

Required columns:

```csv
order_id,session_user_id,ordered_dishes,timestamp
```

Example:

```csv
O001,U01,"D01,D03",2026-01-01 12:30
O002,U02,"D02,D04",2026-01-01 13:00
```

### Cleaning Rules

When orders are loaded:

- `order_id` is trimmed.
- `session_user_id` is trimmed.
- `ordered_dishes` is split by comma, trimmed, uppercased, and empty values are removed.
- `timestamp` is trimmed.

Blank rows are skipped.

## Extending the Dataset

To add a dish:

1. Add a row to `data/dishes.csv`.
2. Give the dish a unique uppercase-style ID such as `D31`.
3. Add ingredients and tags as comma-separated values.
4. Optionally place an image at `assets/dishes/D31.jpg`, `assets/dishes/D31.png`, or `assets/dishes/D31.jpeg`.
5. Optionally add image source details to `assets/dish_image_sources.csv`.
6. Save the CSV and restart the application.

To improve collaborative filtering:

1. Add more rows to `data/orders.csv`.
2. Include two or more dish IDs in `ordered_dishes`.
3. Re-run the application.

More co-order examples give the collaborative filtering matrix stronger signals.
