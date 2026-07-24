# Data Format

The prototype uses CSV so the FYP dataset is easy to inspect, edit, import, and export.

## Folder Structure

```text
data/
  dishes.csv
  orders.csv
assets/
  dish_image_sources.csv
  dishes/
```

If `data/dishes.csv` or `data/orders.csv` is missing, the app creates sample files on startup.

## `data/dishes.csv`

Required columns:

```csv
dish_id,name,ingredients,category,tags
```

Optional image columns:

```csv
image_path,image_source_url
```

Older five-column CSV files still work.

Example:

```csv
dish_id,name,ingredients,category,tags,image_path,image_source_url
D01,Nasi Lemak,"rice,coconut milk,pandan,sambal,egg,anchovies,peanuts,cucumber",main,"spicy,malay,signature",assets/dishes/D01.jpg,https://example.test/source
```

### Dish Cleaning Rules

When dishes load:

- `dish_id` is trimmed and uppercased.
- `name` is trimmed.
- `ingredients` are split by comma, trimmed, lowercased, and empty values are removed.
- `category` is trimmed and lowercased.
- `tags` are split by comma, trimmed, lowercased, and empty values are removed.
- blank `image_path` becomes empty optional data.
- blank `image_source_url` becomes empty optional data.

### Price

The current prototype uses a generated placeholder price based on dish ID. A real `price` column can be added later.

## Dish Images

Runtime image loading is local only. The app does not hotlink online images.

Lookup order:

1. `image_path` from `data/dishes.csv`, if present and file exists.
2. `assets/dishes/{dish_id}.jpg`
3. `assets/dishes/{dish_id}.png`
4. `assets/dishes/{dish_id}.jpeg`
5. Catppuccin-themed placeholder if no image exists.

Examples:

```text
assets/dishes/D01.jpg
assets/dishes/D02.png
assets/dishes/D03.jpeg
```

Images are shown in:

- Recommended for You cards
- Menu cards
- Dish detail modal
- Admin dish management preview

## `assets/dish_image_sources.csv`

Use this file to record where local images came from.

Required columns:

```csv
dish_id,dish_name,image_url,license,source_page,local_path
```

Example:

```csv
D01,Nasi Lemak,https://example.test/nasi-lemak.jpg,Creative Commons,https://example.test/page,assets/dishes/D01.jpg
```

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

### Order Cleaning Rules

When orders load:

- `order_id` is trimmed.
- `session_user_id` is trimmed.
- `ordered_dishes` is split by comma, trimmed, uppercased, and empty values are removed.
- `timestamp` is trimmed.
- blank rows are skipped.

Historical order logs power the co-order collaborative filtering matrix.

## Admin Import and Export

The Admin page can:

- import dish CSV from a file picker
- preview dish CSV rows before applying
- replace all dishes or merge imported dishes by `dish_id`
- reload dishes from `data/dishes.csv`
- export current in-memory dishes
- import historical order CSV from a file picker
- preview historical order CSV rows before applying
- reload historical orders from `data/orders.csv`
- export current historical orders
- export completed checkout orders from the current server session

Imports affect the running server session. Use export to download the current in-memory dataset.

## Extending the Dataset

To add a dish manually:

1. Add a row to `data/dishes.csv`.
2. Use a unique dish ID such as `D31`.
3. Add ingredients and tags as comma-separated values.
4. Optionally add `image_path`.
5. Optionally add image source details to `assets/dish_image_sources.csv`.
6. Restart the app or import the CSV through Admin.

To improve collaborative filtering:

1. Add more rows to `data/orders.csv`.
2. Include at least two dish IDs in `ordered_dishes`.
3. Restart the app or import the order CSV through Admin.

More co-order examples produce stronger collaborative recommendation evidence.
