use crate::models::Order;
use crate::preferences::PreferenceOptions;
use crate::web::state::{
    AdminView, DishView, LiveOrder, MenuView, OrderStatus, RecommendationView,
};

/// Renders the customer-facing QR menu home page.
pub fn customer_menu_page(view: &MenuView) -> String {
    let recommended_cards = view
        .recommended
        .iter()
        .map(recommended_card)
        .collect::<String>();
    let menu_cards = view.dishes.iter().map(menu_card).collect::<String>();
    let preference_panel = preference_panel(&view.preference_options, "customer");
    let category_chips = ["All", "Main", "Side", "Appetizer", "Dessert"]
        .iter()
        .map(|category| {
            format!(
                r#"<button class="chip{}" data-category-chip="{}">{}</button>"#,
                if *category == "All" { " active" } else { "" },
                escape_attr(category),
                escape_html(category)
            )
        })
        .collect::<String>();

    let content = format!(
        r#"
        <header class="hero">
            <p class="eyebrow">QR Restaurant Ordering</p>
            <h1>Preference-Driven Menu</h1>
            <p>Browse dishes, tune preferences, receive explainable recommendations, and build a cart from your phone.</p>
        </header>

        <section class="search-panel" aria-label="Search menu">
            <label class="search-box">
                <span class="search-icon">⌕</span>
                <input id="search-input" type="search" placeholder="Search dishes, ingredients, or taste..." autocomplete="off">
            </label>
        </section>

        <section class="category-strip" aria-label="Category filters">
            {category_chips}
        </section>

        {preference_panel}

        <section class="recommendation-band" aria-labelledby="recommended-title">
            <div class="section-heading">
                <div>
                    <h2 id="recommended-title">Recommended for You</h2>
                    <p>Based on your preferences and ordering patterns</p>
                </div>
            </div>
            <div class="recommended-row" id="recommended-row">
                {recommended_cards}
            </div>
            <div class="evaluation-strip" id="recommendation-stats"></div>
        </section>

        <section class="menu-section" aria-labelledby="menu-title">
            <div class="section-heading">
                <div>
                    <h2 id="menu-title">Menu</h2>
                    <p><span id="visible-count">{}</span> dish(es) available</p>
                </div>
            </div>
            <div class="dish-grid" id="dish-grid">
                {menu_cards}
            </div>
        </section>

        {}
        "#,
        view.dishes.len(),
        dish_detail_modal()
    );

    page_shell(
        "Home",
        "home",
        &content,
        &view.dishes_json,
        &view.recommendations_json,
        &view.preference_options_json,
    )
}

/// Renders the prototype cart page.
pub fn cart_page(view: &MenuView) -> String {
    let content = r#"
        <section class="plain-page">
            <h1>Cart</h1>
            <p>Review selected dishes. Checkout creates a live in-memory order for the staff/admin page.</p>
            <div id="cart-page-items" class="cart-list"></div>
            <div class="cart-summary">
                <strong id="cart-page-total">RM 0</strong>
                <button class="primary-action" id="checkout-button" type="button">Place Prototype Order</button>
            </div>
            <p class="status-message" id="checkout-status"></p>
        </section>
    "#;

    page_shell(
        "Cart",
        "cart",
        content,
        &view.dishes_json,
        &view.recommendations_json,
        &view.preference_options_json,
    )
}

/// Renders a simple customer orders placeholder page.
pub fn orders_page(view: &MenuView) -> String {
    let content = format!(
        r#"
        <section class="plain-page">
            <h1>Orders</h1>
            <p>This prototype page represents where customers would review current and past table orders.</p>
            <div class="info-card">
                <strong>Loaded order records</strong>
                <span>{}</span>
            </div>
        </section>
        "#,
        view.order_count
    );

    page_shell(
        "Orders",
        "orders",
        &content,
        &view.dishes_json,
        &view.recommendations_json,
        &view.preference_options_json,
    )
}

/// Renders the staff/admin dashboard and management page.
pub fn admin_page(view: &MenuView, admin: &AdminView) -> String {
    let dashboard = admin_dashboard(admin);
    let live_orders = live_orders_table(&admin.live_orders);
    let historical_orders = historical_orders_table(&admin.historical_orders);
    let dish_management = dish_management_panel(&admin.dishes);
    let recommendation_tester = recommendation_tester(&admin.preference_options, &admin.dishes);
    let csv_tools = csv_tools_panel();

    let content = format!(
        r#"
        <section class="plain-page admin-page">
            <h1>Admin</h1>
            <p>Staff management tools for live orders, dishes, CSV data, and recommendation testing.</p>
            {dashboard}
            {live_orders}
            {dish_management}
            {csv_tools}
            {recommendation_tester}
            {historical_orders}
        </section>
        "#
    );

    page_shell(
        "Admin",
        "admin",
        &content,
        &view.dishes_json,
        &view.recommendations_json,
        &view.preference_options_json,
    )
}

fn page_shell(
    title: &str,
    active: &str,
    content: &str,
    dishes_json: &str,
    recommendations_json: &str,
    preference_options_json: &str,
) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{}</title>
    <link rel="stylesheet" href="/static/app.css">
    <script>
        window.MENU_DISHES = {};
        window.RECOMMENDATIONS = {};
        window.PREFERENCE_OPTIONS = {};
    </script>
    <script src="/static/app.js" defer></script>
</head>
<body>
    <main class="app-shell">
        {content}
    </main>
    {}
</body>
</html>"#,
        escape_html(title),
        dishes_json,
        recommendations_json,
        preference_options_json,
        bottom_nav(active)
    )
}

fn preference_panel(options: &PreferenceOptions, scope: &str) -> String {
    format!(
        r#"
        <section class="preference-card" data-preference-scope="{scope}">
            <details>
                <summary>Personalise Recommendations</summary>
                <p>Select preferences from actual menu ingredients and tags.</p>
                {}
                {}
                {}
                <button class="ghost-action" type="button" data-clear-preferences>Clear preferences</button>
            </details>
        </section>
        "#,
        preference_group(
            "Liked Ingredients",
            "Ingredients the customer wants more often.",
            "liked_ingredients",
            &options.ingredients
        ),
        preference_group(
            "Disliked Ingredients",
            "Dishes containing these are excluded.",
            "disliked_ingredients",
            &options.ingredients
        ),
        preference_group(
            "Preferred Tags",
            "Tags add a smaller content-score bonus.",
            "preferred_tags",
            &options.tags
        )
    )
}

fn preference_group(title: &str, help: &str, kind: &str, values: &[String]) -> String {
    let chips = values
        .iter()
        .map(|value| {
            format!(
                r#"<button class="mini-chip" type="button" data-preference-kind="{kind}" data-preference-value="{}">{}</button>"#,
                escape_attr(value),
                escape_html(value)
            )
        })
        .collect::<String>();

    format!(
        r#"
        <div class="preference-group">
            <h3>{}</h3>
            <p>{}</p>
            <div class="mini-chip-row">{chips}</div>
        </div>
        "#,
        escape_html(title),
        escape_html(help)
    )
}

fn recommended_card(recommendation: &RecommendationView) -> String {
    let dish = &recommendation.dish;
    format!(
        r#"
        <article class="recommendation-card" data-dish-id="{dish_id}">
            {}
            <div class="card-body">
                <h3>{name}</h3>
                <p>{category}</p>
                <span class="reason">{reason}</span>
                <strong>{price}</strong>
                <div class="card-actions">
                    <button class="add-button" data-add-cart="{dish_id}" type="button">Add</button>
                    <button class="ghost-action" data-view-dish="{dish_id}" type="button">Details</button>
                </div>
            </div>
        </article>
        "#,
        image_block(dish, "compact"),
        dish_id = escape_attr(&dish.dish_id),
        name = escape_html(&dish.name),
        category = escape_html(&dish.category),
        reason = escape_html(&recommendation.explanation),
        price = escape_html(&dish.price)
    )
}

fn menu_card(dish: &DishView) -> String {
    let tags = dish
        .tags
        .iter()
        .take(3)
        .map(|tag| format!(r#"<span class="tag">{}</span>"#, escape_html(tag)))
        .collect::<String>();
    let ingredients = dish.ingredients.join(", ");
    let search_text = format!(
        "{} {} {} {} {}",
        dish.dish_id,
        dish.name,
        dish.category,
        dish.tags.join(" "),
        dish.ingredients.join(" ")
    )
    .to_lowercase();

    format!(
        r#"
        <article class="dish-card" data-dish-id="{dish_id}" data-category="{category_lower}" data-search="{search_text}">
            {}
            <div class="dish-content">
                <div class="dish-title-row">
                    <div>
                        <h3>{name}</h3>
                        <p>{category}</p>
                    </div>
                    {badge}
                </div>
                <div class="tag-row">{tags}</div>
                <p class="ingredients">{ingredients}</p>
                <div class="dish-footer">
                    <strong>{price}</strong>
                    <button class="add-button" data-add-cart="{dish_id}" type="button">Add</button>
                </div>
                <button class="text-action" data-view-dish="{dish_id}" type="button">View details</button>
            </div>
        </article>
        "#,
        image_block(dish, ""),
        dish_id = escape_attr(&dish.dish_id),
        category_lower = escape_attr(&dish.category.to_lowercase()),
        search_text = escape_attr(&search_text),
        name = escape_html(&dish.name),
        category = escape_html(&dish.category),
        badge = if dish.recommended {
            r#"<span class="badge">Recommended</span>"#
        } else {
            ""
        },
        tags = tags,
        ingredients = escape_html(&ingredients),
        price = escape_html(&dish.price)
    )
}

fn image_block(dish: &DishView, extra_class: &str) -> String {
    match &dish.image_url {
        Some(url) => format!(
            r#"<div class="dish-art {extra_class}"><img src="{}" alt="{}"></div>"#,
            escape_attr(url),
            escape_attr(&dish.name)
        ),
        None => format!(
            r#"<div class="dish-art placeholder {extra_class}" aria-label="No image for {}">🍽</div>"#,
            escape_attr(&dish.name)
        ),
    }
}

fn dish_detail_modal() -> &'static str {
    r#"
    <dialog class="dish-modal" id="dish-detail-modal">
        <div class="modal-content">
            <button class="modal-close" type="button" data-close-dish-modal>×</button>
            <div id="dish-detail-content"></div>
        </div>
    </dialog>
    "#
}

fn admin_dashboard(admin: &AdminView) -> String {
    let frequent = frequency_list(&admin.frequent_dishes);
    let pairs = frequency_list(&admin.co_order_pairs);
    format!(
        r#"
        <section class="admin-grid">
            <div class="metric-card"><span>Total dishes</span><strong>{}</strong></div>
            <div class="metric-card"><span>Available</span><strong>{}</strong></div>
            <div class="metric-card"><span>Unavailable</span><strong>{}</strong></div>
            <div class="metric-card"><span>Historical orders</span><strong>{}</strong></div>
            <div class="metric-card"><span>Live orders</span><strong>{}</strong></div>
        </section>
        <section class="admin-two-column">
            <div class="admin-card"><h2>Most Frequent Dishes</h2>{frequent}</div>
            <div class="admin-card"><h2>Common Co-order Pairs</h2>{pairs}</div>
        </section>
        "#,
        admin.total_dishes,
        admin.available_dishes,
        admin.unavailable_dishes,
        admin.historical_order_count,
        admin.live_order_count
    )
}

fn frequency_list(values: &[crate::web::state::FrequencyView]) -> String {
    if values.is_empty() {
        return r#"<p class="muted">No order data yet.</p>"#.to_string();
    }

    let items = values
        .iter()
        .map(|value| {
            format!(
                r#"<li><span>{}</span><strong>{}</strong></li>"#,
                escape_html(&value.label),
                value.count
            )
        })
        .collect::<String>();
    format!(r#"<ul class="frequency-list">{items}</ul>"#)
}

fn live_orders_table(live_orders: &[LiveOrder]) -> String {
    let rows = if live_orders.is_empty() {
        r#"<tr><td colspan="5">No live customer orders yet.</td></tr>"#.to_string()
    } else {
        live_orders
            .iter()
            .rev()
            .map(|order| {
                format!(
                    r#"
                    <tr data-live-order-row="{order_id}">
                        <td>{order_id}</td>
                        <td>{session}</td>
                        <td>{dishes}</td>
                        <td>{timestamp}</td>
                        <td>
                            <select data-order-status="{order_id}">
                                {status_options}
                            </select>
                        </td>
                    </tr>
                    "#,
                    order_id = escape_attr(&order.order_id),
                    session = escape_html(&order.session_user_id),
                    dishes = escape_html(&order.ordered_dishes.join(", ")),
                    timestamp = escape_html(&order.timestamp),
                    status_options = status_options(order.status)
                )
            })
            .collect::<String>()
    };

    format!(
        r#"
        <section class="admin-card">
            <div class="section-heading"><h2>Live Orders</h2><p>Orders created from customer checkout in this server session.</p></div>
            <div class="table-wrap">
                <table>
                    <thead><tr><th>Order ID</th><th>Session</th><th>Dishes</th><th>Time</th><th>Status</th></tr></thead>
                    <tbody>{rows}</tbody>
                </table>
            </div>
        </section>
        "#
    )
}

fn historical_orders_table(orders: &[Order]) -> String {
    let rows = orders
        .iter()
        .map(|order| {
            format!(
                r#"<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>"#,
                escape_html(&order.order_id),
                escape_html(&order.session_user_id),
                escape_html(&order.ordered_dishes.join(", ")),
                escape_html(&order.timestamp)
            )
        })
        .collect::<String>();

    format!(
        r#"
        <section class="admin-card">
            <div class="section-heading"><h2>Historical Orders</h2><p>CSV order logs used for co-order patterns.</p></div>
            <div class="table-wrap">
                <table>
                    <thead><tr><th>Order ID</th><th>Session</th><th>Dishes</th><th>Timestamp</th></tr></thead>
                    <tbody>{rows}</tbody>
                </table>
            </div>
        </section>
        "#
    )
}

fn status_options(selected: OrderStatus) -> String {
    [
        OrderStatus::Pending,
        OrderStatus::Preparing,
        OrderStatus::Ready,
        OrderStatus::Completed,
        OrderStatus::Cancelled,
    ]
    .iter()
    .map(|status| {
        format!(
            r#"<option value="{}"{}>{}</option>"#,
            status.label(),
            if *status == selected { " selected" } else { "" },
            status.label()
        )
    })
    .collect()
}

fn dish_management_panel(dishes: &[DishView]) -> String {
    let rows = dishes
        .iter()
        .map(|dish| {
            format!(
                r#"
                <tr data-admin-dish-row="{dish_id}">
                    <td>{image}</td>
                    <td><strong>{name}</strong><span>{id_label}</span></td>
                    <td>{category}</td>
                    <td>{ingredients}</td>
                    <td>{availability}</td>
                    <td>
                        <button class="ghost-action" data-toggle-dish="{dish_id}" data-available="{next_available}" type="button">{toggle_label}</button>
                        <button class="danger-action" data-delete-dish="{dish_id}" type="button">Delete</button>
                    </td>
                </tr>
                "#,
                image = image_block(dish, "thumb"),
                name = escape_html(&dish.name),
                id_label = escape_html(&dish.dish_id),
                category = escape_html(&dish.category),
                ingredients = escape_html(&dish.ingredients.join(", ")),
                availability = if dish.available { "Available" } else { "Unavailable" },
                dish_id = escape_attr(&dish.dish_id),
                next_available = if dish.available { "false" } else { "true" },
                toggle_label = if dish.available {
                    "Mark unavailable"
                } else {
                    "Mark available"
                }
            )
        })
        .collect::<String>();

    format!(
        r#"
        <section class="admin-card">
            <div class="section-heading"><h2>Dish Management</h2><p>In-memory management for FYP demo; CSV persistence can be added later.</p></div>
            <form class="admin-form" id="dish-form">
                <input name="dish_id" placeholder="Dish ID (optional)">
                <input name="name" placeholder="Dish name" required>
                <input name="category" placeholder="Category" required>
                <input name="ingredients" placeholder="Ingredients: chicken, rice" required>
                <input name="tags" placeholder="Tags: spicy, signature">
                <input name="image_path" placeholder="assets/dishes/D31.jpg">
                <button class="primary-action" type="submit">Add / Update Dish</button>
            </form>
            <p class="status-message" id="dish-management-status"></p>
            <div class="table-wrap">
                <table>
                    <thead><tr><th>Image</th><th>Dish</th><th>Category</th><th>Ingredients</th><th>Status</th><th>Actions</th></tr></thead>
                    <tbody>{rows}</tbody>
                </table>
            </div>
        </section>
        "#
    )
}

fn csv_tools_panel() -> &'static str {
    r#"
    <section class="admin-card">
        <div class="section-heading"><h2>CSV Tools</h2><p>Import validates CSV text and replaces the matching in-memory dataset for this demo session.</p></div>
        <h3>Dishes CSV</h3>
        <textarea id="dish-csv-import" rows="6" placeholder="Paste dishes.csv content here"></textarea>
        <div class="form-actions">
            <button class="primary-action" id="import-dishes-button" type="button">Import Dishes CSV</button>
            <a class="ghost-link" href="/admin/export/dishes.csv">Export Dishes CSV</a>
        </div>
        <h3>Historical Orders CSV</h3>
        <textarea id="order-csv-import" rows="5" placeholder="Paste orders.csv content here"></textarea>
        <div class="form-actions">
            <button class="primary-action" id="import-orders-button" type="button">Import Orders CSV</button>
            <a class="ghost-link" href="/admin/export/orders.csv">Export Orders CSV</a>
        </div>
        <p class="status-message" id="csv-import-status"></p>
    </section>
    "#
}

fn recommendation_tester(options: &PreferenceOptions, dishes: &[DishView]) -> String {
    let dish_options = dishes
        .iter()
        .map(|dish| {
            format!(
                r#"<option value="{}">{} ({})</option>"#,
                escape_attr(&dish.dish_id),
                escape_html(&dish.name),
                escape_html(&dish.dish_id)
            )
        })
        .collect::<String>();

    format!(
        r#"
        <section class="admin-card">
            <div class="section-heading"><h2>Recommendation Testing</h2><p>Run content, collaborative, and hybrid scoring with explainable output.</p></div>
            {}
            <label class="field-label">Selected dish/order context</label>
            <select id="admin-selected-dishes" multiple>{dish_options}</select>
            <button class="primary-action" type="button" id="run-admin-recommendations">Run Recommendation Test</button>
            <div class="table-wrap recommendation-results-wrap">
                <table>
                    <thead><tr><th>Dish</th><th>Content</th><th>Co-order</th><th>Hybrid</th><th>Reason</th></tr></thead>
                    <tbody id="admin-recommendation-results"></tbody>
                </table>
            </div>
        </section>
        "#,
        preference_panel(options, "admin")
    )
}

fn bottom_nav(active: &str) -> String {
    let items = [
        ("home", "Home", "/", "⌂"),
        ("orders", "Orders", "/orders", "▤"),
        ("cart", "Cart", "/cart", "🛒"),
        ("admin", "Admin", "/admin", "◈"),
    ];

    let links = items
        .iter()
        .map(|(id, label, href, icon)| {
            let cart_count = if *id == "cart" {
                r#"<span class="cart-count" data-cart-count>0</span>"#
            } else {
                ""
            };
            format!(
                r#"<a class="nav-item{}" href="{}"><span>{}</span><strong>{}</strong>{}</a>"#,
                if *id == active { " active" } else { "" },
                href,
                icon,
                label,
                cart_count
            )
        })
        .collect::<String>();

    format!(r#"<nav class="bottom-nav">{links}</nav>"#)
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn escape_attr(value: &str) -> String {
    escape_html(value)
}
