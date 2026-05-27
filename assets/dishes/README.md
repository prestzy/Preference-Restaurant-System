# Dish Images

Place local dish thumbnails in this folder.

The app first uses `image_path` from `data/dishes.csv` when that optional column is present. If no path is provided, it automatically tries:

- `assets/dishes/{dish_id}.jpg`
- `assets/dishes/{dish_id}.png`
- `assets/dishes/{dish_id}.jpeg`

Examples:

- `assets/dishes/D01.jpg`
- `assets/dishes/D02.png`
- `assets/dishes/D03.jpeg`

If no matching file exists, the GUI shows a `No image` placeholder instead of failing.
