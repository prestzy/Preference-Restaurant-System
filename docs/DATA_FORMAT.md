# Data Format

The app uses CSV files so the dataset is easy to edit for FYP demonstrations.

## Folder Structure

```text
data/
  dishes.csv
  orders.csv
```

If the CSV files are missing, the app creates sample data automatically.

## `data/dishes.csv`

Required columns:

```csv
dish_id,name,ingredients,category,tags
```

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
4. Save the CSV and restart the application.

To improve collaborative filtering:

1. Add more rows to `data/orders.csv`.
2. Include two or more dish IDs in `ordered_dishes`.
3. Re-run the application.

More co-order examples give the collaborative filtering matrix stronger signals.
