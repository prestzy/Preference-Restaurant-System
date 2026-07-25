# Data Model and Persistence

## Dishes

`data/dishes.csv` is loaded at startup.

Required columns:

```csv
dish_id,name,ingredients,category,tags
```

Optional backward-compatible columns:

```csv
image_path,image_source_url
```

Example:

```csv
D01,Nasi Lemak,"rice,coconut milk,pandan,sambal,egg",main,"spicy,malay,signature",assets/dishes/D01.jpg,
```

Rules:

- `dish_id`, `name`, category, and at least one ingredient must be non-empty.
- Dish IDs are normalised to uppercase and must be unique
  case-insensitively.
- Ingredients and tags are split by comma, trimmed, lowercased, and emptied
  values are removed.
- `image_source_url` is provenance only and is never hotlinked at runtime.

Image lookup order:

1. existing local `image_path`;
2. `assets/dishes/{DISH_ID}.jpg`;
3. `assets/dishes/{DISH_ID}.png`;
4. `assets/dishes/{DISH_ID}.jpeg`;
5. themed placeholder.

Image sources should be recorded in `assets/dish_image_sources.csv`.

## Historical Orders

`data/orders.csv` columns:

```csv
order_id,session_user_id,ordered_dishes,timestamp
```

Example:

```csv
O061,U21,"D01,D09,D30","2026-07-25 14:30"
```

Rules:

- Order IDs are uppercase and unique.
- Session IDs are non-empty.
- `ordered_dishes` is one quoted CSV field containing uppercase IDs.
- Duplicate dish IDs inside one basket are removed, preserving first order.
- Every referenced dish must exist.
- Timestamp format is `%Y-%m-%d %H:%M`.

When a live order first reaches `Completed`, persistence finds the maximum
existing `Oxxx` and `Uxx`, generates the next identifiers, and appends through
the CSV writer. The live web ID is not written into historical CSV. An
idempotency flag prevents repeated completion from appending twice.

## Runtime Order Details

`data/order_details.csv` is runtime-only and ignored by Git because it can
contain customer contact data. The safe schema example is:

`data/order_details.example.csv`

It includes the live order ID, customer/session information, line items, total,
status, and completion reference needed by the current installation. Replacement
rewrites use the atomic-file helper.

## Recommendation Learning Events

`data/recommendation_learning_events.jsonl` is an ignored, derived timeline.
Each line is one JSON event created from a real completed order. It can be
cleared and rebuilt from durable historical orders. It is not the source of
recommendation evidence.

## Availability and Prices

Availability changes and dish edits are in memory for the current process;
admins can export the current dish view to CSV. Prototype prices are derived by
the application and are not currently columns in `dishes.csv`. These are known
persistence limitations.

## Data Validation Failures

Startup stops with contextual errors for missing columns, duplicate IDs, invalid
timestamps, empty required fields, and unknown dish references. This is
intentional: silently dropping corrupted historical evidence would make
recommendation explanations unreliable.

## Safe Dataset Extension

1. Stop the server.
2. Back up the CSV.
3. Add a unique dish row with normalised comma-separated fields.
4. Add a licensed local image and source record if available.
5. Ensure every order reference points to an existing dish.
6. Run `cargo test`.
7. Start the server and inspect startup output.
8. Verify search, menu, preference options, and recommendations.

Never commit real names, phone numbers, credentials, session cookies, or private
order details.

