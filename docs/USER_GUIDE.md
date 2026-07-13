# User Guide

## Start the Web App

```powershell
cargo run
```

Open:

```text
http://127.0.0.1:3000/
```

The app is bound to localhost for development. For phone testing from another device, the bind address would need to be changed later.

## Customer Menu

The Home page is the QR customer menu.

Customers can:

- Search by dish name, dish ID, ingredient, category, or tag.
- Filter by category chips.
- Select liked ingredients, disliked ingredients, and preferred tags.
- View “Recommended for You”.
- Open dish details.
- Add dishes to the cart.
- Place a prototype order.

The recommendation section updates after preference chips change or cart contents change.

## Preference Chips

Preference options are generated from the loaded dish dataset:

- Liked Ingredients
- Disliked Ingredients
- Preferred Tags

If an ingredient is selected as liked, it is removed from disliked, and the opposite also applies. This avoids sending contradictory preference data to the recommender.

## Dish Details

Click **Details** on a menu or recommendation card.

The detail modal shows:

- local image or placeholder
- dish ID
- category
- ingredients
- tags
- price placeholder
- recommendation reason when the dish is recommended
- Add to Cart button

## Cart and Checkout

Open:

```text
http://127.0.0.1:3000/cart
```

The cart uses browser `localStorage` for the prototype. Customers can:

- increase quantity
- decrease quantity
- remove item
- view subtotal and total
- place a prototype order

Checkout sends selected dish IDs to:

```text
POST /api/orders
```

The server validates dish IDs and creates a live in-memory order.

## Admin Page

Open:

```text
http://127.0.0.1:3000/admin
```

Admin tools include:

- dashboard metrics
- most frequent dishes
- common co-order pairs
- live order table
- live order status update
- dish management
- CSV import/export
- recommendation testing
- historical order table

## Live Orders

Place a customer order from the cart, then open the Admin page.

The new order appears under **Live Orders**. Staff can change status:

- Pending
- Preparing
- Ready
- Completed
- Cancelled

Live orders are in memory for the current server session.

## Dish Management

The Admin page can add or update dishes in memory.

Fields:

- Dish ID, optional
- Dish name
- Category
- Ingredients
- Tags
- Image path

If Dish ID is blank, the system generates the next `Dxx` ID. Use CSV export if you want to save the current in-memory dish list.

## CSV Tools

Admin CSV tools support:

- import dishes CSV
- export dishes CSV
- import historical orders CSV
- export historical orders CSV

Imports replace the matching in-memory dataset for the current server session.

## Recommendation Testing

The Admin page includes a recommendation testing section.

Select:

- liked ingredients
- disliked ingredients
- preferred tags
- selected dish/order context

Click **Run Recommendation Test**.

The result table shows:

- dish name
- content score
- co-order score
- hybrid score
- explanation
- matched liked ingredients
- matched preferred tags
- co-order influence

This section is useful for FYP demonstration and evaluation discussion.
