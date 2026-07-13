# Dish Images

Place local dish thumbnails in this folder.

The web app first uses `image_path` from `data/dishes.csv` when that optional column is present and the file exists. If no path is provided, it automatically tries:

- `assets/dishes/{dish_id}.jpg`
- `assets/dishes/{dish_id}.png`
- `assets/dishes/{dish_id}.jpeg`

Examples:

- `assets/dishes/D01.jpg`
- `assets/dishes/D02.png`
- `assets/dishes/D03.jpeg`

If no matching file exists, the customer menu, recommendation cards, dish detail modal, and admin dish preview show a themed placeholder instead of failing.

Record image sources in:

```text
assets/dish_image_sources.csv
```

Runtime image loading is local only. Do not hotlink external image URLs in the UI.
