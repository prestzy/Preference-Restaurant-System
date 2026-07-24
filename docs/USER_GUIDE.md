# User Guide

## Start the Web App

```powershell
cargo run
```

Open:

```text
http://127.0.0.1:3000/
```

The app uses localhost by default. For phone testing, start it with:

```powershell
$env:APP_HOST="0.0.0.0"
cargo run
```

Then open `http://<computer-lan-ip>:3000/` from a phone on the same local
network. Use `ipconfig` to find the computer's IPv4 address. Registration and
admin login use normal relative form submissions and separate host-only
session cookies, so the browser must keep using the same hostname/IP.

## Customer Menu

The Home page is the QR customer menu.

Customers can:

- Search by dish name, dish ID, ingredient, category, or tag.
- View live search suggestions with dish image, category, price, and match reason.
- Filter by category chips.
- Select liked ingredients, disliked ingredients, and preferred tags.
- View “Recommended for You”.
- Open dish details.
- Add dishes to the cart.
- Place a prototype order.

The recommendation section updates after preference chips change or cart contents change.

## Live Search Suggestions

Start typing in:

```text
Search dishes, ingredients, or taste...
```

The dropdown shows matching dishes immediately. Each suggestion includes:

- image or placeholder
- dish name
- category
- price
- simple match reason such as `ingredient: chicken` or `tag: spicy`

Click a suggestion to scroll to the matching dish card.

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

The server validates dish IDs and creates a live in-memory order. The latest order ID is stored in the browser so the Orders page can show the current status for this server session.

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

When staff mark an order as **Completed**, it moves out of the active Live Orders table and is added immediately to **Historical Orders** for the current server session. This also makes the completed order available as collaborative recommendation evidence.

Cancelled orders remain visible in the live order table with Cancelled status. This is still in-memory prototype state, not persistent order history.

## Dish Management

The Admin page can add or update dishes in memory.

Fields:

- Dish ID, optional
- Dish name
- Category
- Ingredients
- Tags
- Image path

If Dish ID is blank, the system generates the next `Dxx` ID. Changes are in memory for the current server session.

## Recommendation Testing

The Admin page includes a recommendation testing section.

Select:

- liked ingredients
- disliked ingredients
- preferred tags
- selected dish/order context
- time context: Any, Breakfast, Lunch, Dinner, or Dessert/Snack
- ranking method: hybrid, content-based, or co-ordering

Click **Run Recommendation Test**.

The result table shows:

- dish ID and name
- category
- content score
- co-order score
- popularity score
- time/business score
- hybrid score
- support
- confidence
- lift
- explanation
- matched liked ingredients
- matched preferred tags
- co-order influence

Hybrid scoring uses:

```text
0.45 content + 0.25 co-order + 0.20 popularity + 0.10 time/business
```

Popularity fallback keeps recommendations visible when the customer has not selected preferences or cart context. Association metrics help explain co-ordering evidence for FYP screenshots and Chapter 4 discussion.

This section is useful for FYP demonstration and evaluation discussion.
