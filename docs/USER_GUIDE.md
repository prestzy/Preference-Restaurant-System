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
- Select button.

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

Click **Select** on menu cards. Selected dishes are passed directly into the recommendation engine as the current order context.

Manual selected dish ID input still exists as a fallback for demos or keyboard-driven testing.

### Enter Preferences

Preference fields accept comma-separated values:

```text
Liked ingredients: chicken, rice, egg
Disliked ingredients: beef, anchovies
Preferred tags: spicy, signature
```

Recommendations refresh automatically when preference text or selected dishes change.

### Understand Recommendations

Each recommendation shows:

- Dish name and ID.
- Ingredient score.
- Co-order score.
- Final hybrid score.
- Plain-language explanation.

Example:

```text
Recommended because it contains preferred ingredient(s): chicken and often ordered with selected dish(es): D01.
```

## Evaluation

The Evaluation page shows lightweight prototype testing values:

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
