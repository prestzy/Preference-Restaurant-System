# User Guide

## Starting the App

Run:

```powershell
cargo run
```

The desktop window opens after CSV data is loaded.

## Dashboard

The Dashboard gives a short explanation of the system and shows the number of loaded dishes and historical orders.

## Explore & Recommend

This is the main user workflow.

### Browse the Menu

The menu shows dish cards with:

- Dish ID.
- Dish name.
- Category.
- Ingredients.
- Tags.
- **Select Dish** button.

### Search the Menu

Use the menu search box to filter by:

- Dish ID.
- Dish name.
- Category.
- Ingredient.
- Tag.

Multiple terms can be separated using:

```text
comma, semicolon; pipe | newline
```

Examples:

```text
chicken, spicy
rice; signature
D01 | main
```

Search modes:

- **Match Any**: show dishes matching at least one term.
- **Match All**: show dishes matching every term.

Active filter tokens appear below the search field.

### Select Dishes

Click **Select Dish** on menu cards. Selected dishes appear in the **Selected Dishes** section and are passed directly into the recommendation engine as the current order context.

### Enter Preferences

Preference options are generated from the loaded menu data.

Use the selectable chips under:

- **Liked Ingredients**
- **Disliked Ingredients**
- **Preferred Tags**

The same ingredient cannot stay in both liked and disliked lists. If you select an ingredient as liked, it is removed from disliked. If you select it as disliked, it is removed from liked.

Recommendations refresh automatically when selected preference chips or selected dishes change.

### View Recommendations

Recommendation results are shown on the **Evaluation** page, not on Explore & Recommend. This keeps Explore focused on menu browsing, preference selection, and the cart.

Each recommendation card shows:

- Dish name and ID.
- Ingredient score.
- Co-order score.
- Final hybrid score.
- Matched liked ingredients.
- Matched preferred tags.
- Disliked ingredient exclusion status.
- Selected cart dish that influenced the co-order score.
- Plain-language explanation.

Example:

```text
Recommended because it contains preferred ingredient(s): chicken and often ordered with selected dish(es): D01.
```

## Evaluation

The Evaluation page shows recommendation output and lightweight prototype testing values:

- Recommendation cards with detailed score explanations.
- Number of available recommendations.
- Dishes evaluated after filtering.
- Dishes excluded due to disliked ingredients.
- Already selected dishes skipped.
- Category diversity count in the top 5 recommendations.

These are demo support metrics, not a full academic recommender-system evaluation.

## Admin / Demo Tools

Order simulation is placed in a separate admin/demo section because it is not a normal customer step.

Use it to create a simulated order such as:

```text
D01, D09, D30
```

After insertion:

- The order is added to memory.
- Collaborative filtering uses the new order immediately.
- Recommendations refresh.

You may also enable:

```text
Also append simulated order to data/orders.csv
```

If CSV append fails, the in-memory demo order still remains active for the current session.
