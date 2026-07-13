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
            <div class="search-suggestions" id="search-suggestions" hidden></div>
        </section>

        <section class="assistant-card" aria-labelledby="assistant-title">
            <div class="section-heading compact-heading">
                <div>
                    <h2 id="assistant-title">Smart Menu Assistant</h2>
                    <p>Tell us what you feel like eating. Example: spicy chicken but no beef.</p>
                </div>
            </div>
            <div class="assistant-input-row">
                <input id="assistant-prompt" type="text" placeholder="Tell us what you feel like eating, e.g. spicy chicken but no beef">
                <button class="primary-action" id="assistant-run" type="button">Ask</button>
            </div>
            <p class="assistant-understood" id="assistant-understood"></p>
            <div class="assistant-upsells" id="assistant-upsells"></div>
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
                <div class="carousel-controls" aria-label="Recommended dish carousel controls">
                    <button class="carousel-arrow" type="button" data-carousel-scroll="recommended-row" data-direction="-1" aria-label="Scroll recommendations left">‹</button>
                    <button class="carousel-arrow" type="button" data-carousel-scroll="recommended-row" data-direction="1" aria-label="Scroll recommendations right">›</button>
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
pub fn orders_page(view: &MenuView, orders: &[LiveOrder]) -> String {
    let order_cards = if orders.is_empty() {
        r#"<div class="info-card"><strong>No current orders yet.</strong><span>Place an order from the cart to track it here.</span></div>"#.to_string()
    } else {
        orders.iter().map(order_card).collect::<String>()
    };
    let content = format!(
        r#"
        <section class="plain-page">
            <h1>Orders</h1>
            <p>Track checkout orders from this server session.</p>
            <section class="order-filter-row" aria-label="Order status filters">
                <button class="chip active" type="button" data-order-filter="all">All</button>
                <button class="chip" type="button" data-order-filter="pending">Pending</button>
                <button class="chip" type="button" data-order-filter="preparing">Preparing</button>
                <button class="chip" type="button" data-order-filter="ready">Ready</button>
                <button class="chip" type="button" data-order-filter="completed">Completed</button>
                <button class="chip" type="button" data-order-filter="cancelled">Cancelled</button>
            </section>
            <div class="order-card-list" id="orders-list">{order_cards}</div>
            <div class="info-card slim">
                <strong>Historical order records loaded</strong>
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

fn order_card(order: &LiveOrder) -> String {
    format!(
        r#"
        <article class="order-card" data-order-status-card="{status_lower}">
            <div class="order-card-main">
                <div>
                    <p class="eyebrow">{order_id}</p>
                    <h2>{status_badge} {total}</h2>
                </div>
                <span class="status-badge {status_lower}">{status}</span>
            </div>
            <p><strong>Dishes:</strong> {dish_names}</p>
            <p><strong>Dish IDs:</strong> {dish_ids}</p>
            <p><strong>Time:</strong> {timestamp}</p>
        </article>
        "#,
        order_id = escape_html(&order.order_id),
        status = order.status.label(),
        status_lower = order.status.label().to_lowercase(),
        status_badge = escape_html(order.status.label()),
        total = escape_html(&order.total_price),
        dish_names = escape_html(&order.dish_names.join(", ")),
        dish_ids = escape_html(&order.ordered_dishes.join(", ")),
        timestamp = escape_html(&order.timestamp)
    )
}

/// Renders the staff/admin dashboard and management page.
pub fn admin_page(view: &MenuView, admin: &AdminView) -> String {
    let dashboard = admin_dashboard(admin);

    let content = format!(
        r#"
        <section class="plain-page admin-page">
            <h1>Admin</h1>
            <p>Dashboard overview for the QR ordering and recommendation prototype.</p>
            {}
            {dashboard}
        </section>
        "#,
        admin_section_nav("dashboard")
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

pub fn admin_orders_page(view: &MenuView, admin: &AdminView) -> String {
    let content = format!(
        r#"
        <section class="plain-page admin-page">
            <h1>Admin Orders</h1>
            <p>Manage live checkout orders and review saved historical order baskets.</p>
            {}
            {}
            {}
            {}
        </section>
        "#,
        admin_section_nav("orders"),
        live_orders_table(&admin.live_orders),
        completed_orders_table(&admin.completed_session_orders),
        historical_orders_table(&admin.historical_orders)
    );

    page_shell(
        "Admin Orders",
        "admin",
        &content,
        &view.dishes_json,
        &view.recommendations_json,
        &view.preference_options_json,
    )
}

pub fn admin_dishes_page(view: &MenuView, admin: &AdminView) -> String {
    let content = format!(
        r#"
        <section class="plain-page admin-page">
            <h1>Dish Management</h1>
            <p>Add, edit, hide, and review dishes used by the customer menu.</p>
            {}
            {}
        </section>
        "#,
        admin_section_nav("dishes"),
        dish_management_panel(&admin.dishes)
    );

    page_shell(
        "Admin Dishes",
        "admin",
        &content,
        &view.dishes_json,
        &view.recommendations_json,
        &view.preference_options_json,
    )
}

pub fn admin_recommendations_page(view: &MenuView, admin: &AdminView) -> String {
    let content = format!(
        r#"
        <section class="plain-page admin-page">
            <h1>Recommendation Tester</h1>
            <p>Academic scoring view for content, co-ordering, popularity, time boost, and association metrics.</p>
            {}
            {}
        </section>
        "#,
        admin_section_nav("recommendations"),
        recommendation_tester(&admin.preference_options, &admin.dishes)
    );

    page_shell(
        "Admin Recommendations",
        "admin",
        &content,
        &view.dishes_json,
        &view.recommendations_json,
        &view.preference_options_json,
    )
}

pub fn admin_data_page(view: &MenuView) -> String {
    let content = format!(
        r#"
        <section class="plain-page admin-page">
            <h1>CSV / Data Tools</h1>
            <p>Reload, import, preview, and export the CSV files used by this prototype.</p>
            {}
            {}
        </section>
        "#,
        admin_section_nav("data"),
        csv_data_tools_panel()
    );

    page_shell(
        "Admin Data",
        "admin",
        &content,
        &view.dishes_json,
        &view.recommendations_json,
        &view.preference_options_json,
    )
}

pub fn admin_insights_page(view: &MenuView, admin: &AdminView) -> String {
    let content = format!(
        r#"
        <section class="plain-page admin-page">
            <h1>Insights</h1>
            <p>Popularity, co-order patterns, and rule-based Smart Menu insights.</p>
            {}
            {}
            <section class="admin-two-column">
                <div class="admin-card"><h2>Most Frequent Dishes</h2>{}</div>
                <div class="admin-card"><h2>Common Co-order Pairs</h2>{}</div>
            </section>
        </section>
        "#,
        admin_section_nav("insights"),
        admin_insight_panel(),
        frequency_list(&admin.frequent_dishes),
        frequency_list(&admin.co_order_pairs)
    );

    page_shell(
        "Admin Insights",
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
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
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

fn admin_section_nav(active: &str) -> String {
    let links = [
        ("dashboard", "Dashboard", "/admin"),
        ("orders", "Orders", "/admin/orders"),
        ("dishes", "Dishes", "/admin/dishes"),
        (
            "recommendations",
            "Recommendation Tester",
            "/admin/recommendations",
        ),
        ("data", "CSV / Data", "/admin/data"),
        ("insights", "Insights", "/admin/insights"),
    ];
    let items = links
        .iter()
        .map(|(id, label, href)| {
            format!(
                r#"<a class="admin-nav-link{}" href="{}">{}</a>"#,
                if *id == active { " active" } else { "" },
                escape_attr(href),
                escape_html(label)
            )
        })
        .collect::<String>();

    format!(r#"<nav class="admin-section-nav">{items}</nav>"#)
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
        reason = escape_html(&short_recommendation_reason(recommendation)),
        price = escape_html(&dish.price)
    )
}

fn short_recommendation_reason(recommendation: &RecommendationView) -> String {
    let mut parts = Vec::new();
    let mut preference_matches = recommendation.matched_liked_ingredients.clone();
    preference_matches.extend(recommendation.matched_preferred_tags.clone());

    if !preference_matches.is_empty() {
        parts.push(format!(
            "Matches {}",
            preference_matches
                .into_iter()
                .take(3)
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if let Some(related) = recommendation.related_selected_dishes.first() {
        parts.push(format!("Often ordered with {related}"));
    }
    if recommendation.popularity_score > 0.0 && parts.is_empty() {
        parts.push("Popular from order history".to_string());
    }
    if recommendation.business_rule_score > 0.0 && parts.len() < 2 {
        parts.push("Fits the menu context".to_string());
    }

    if parts.is_empty() {
        "Based on ingredients and order patterns".to_string()
    } else {
        parts.join(" · ")
    }
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
            <div class="metric-card"><span>Completed session orders</span><strong>{}</strong></div>
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
        admin.live_order_count,
        admin.completed_session_order_count
    )
}

fn admin_insight_panel() -> String {
    r#"
        <section class="admin-card assistant-card">
            <div class="section-heading">
                <div>
                    <h2>Smart Menu Insights</h2>
                    <p>Rule-based summary calculated from historical order baskets.</p>
                </div>
                <button class="ghost-action" id="refresh-admin-insights" type="button">Refresh</button>
            </div>
            <p class="assistant-understood" id="admin-insight-summary">Loading insights...</p>
            <div class="insight-grid" id="admin-insight-grid"></div>
        </section>
    "#
    .to_string()
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
        r#"<tr><td colspan="7">No active live customer orders yet.</td></tr>"#.to_string()
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
                        <td>{names}</td>
                        <td>{timestamp}</td>
                        <td>{total}</td>
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
                    names = escape_html(&order.dish_names.join(", ")),
                    timestamp = escape_html(&order.timestamp),
                    total = escape_html(&order.total_price),
                    status_options = status_options(order.status)
                )
            })
            .collect::<String>()
    };

    format!(
        r#"
        <section class="admin-card">
            <div class="section-heading"><h2>Live Orders</h2><p>Pending, preparing, and ready checkout orders from this server session.</p></div>
            <p class="status-message" id="admin-order-status"></p>
            <div class="table-wrap">
                <table>
                    <thead><tr><th>Order ID</th><th>Session</th><th>Dish IDs</th><th>Dish Names</th><th>Time</th><th>Total</th><th>Status</th></tr></thead>
                    <tbody>{rows}</tbody>
                </table>
            </div>
        </section>
        "#
    )
}

fn completed_orders_table(orders: &[LiveOrder]) -> String {
    let rows = if orders.is_empty() {
        r#"<tr><td colspan="6">No checkout order has been completed in this server session yet.</td></tr>"#.to_string()
    } else {
        orders
            .iter()
            .rev()
            .map(|order| {
                format!(
                    r#"<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>"#,
                    escape_html(&order.order_id),
                    escape_html(&order.session_user_id),
                    escape_html(&order.ordered_dishes.join(", ")),
                    escape_html(&order.dish_names.join(", ")),
                    escape_html(&order.timestamp),
                    escape_html(&order.total_price)
                )
            })
            .collect::<String>()
    };

    format!(
        r#"
        <section class="admin-card">
            <div class="section-heading"><h2>Completed Orders This Session</h2><p>Checkout orders marked Completed during this server session. They are also saved into data/orders.csv for future recommendation calculations.</p></div>
            <div class="form-actions">
                <a class="ghost-link" href="/admin/export/completed-session-orders.csv">Export Completed Session Orders</a>
            </div>
            <div class="table-wrap">
                <table>
                    <thead><tr><th>Order ID</th><th>Session</th><th>Dish IDs</th><th>Dish Names</th><th>Timestamp</th><th>Total</th></tr></thead>
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
            <div class="section-heading"><h2>Historical Orders</h2><p>Order logs loaded from data/orders.csv, including completed checkout orders saved by the admin flow.</p></div>
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
            <div class="admin-form compact-form">
                <label class="field-label">Time Context
                    <select id="admin-time-context">
                        <option value="Any">Any</option>
                        <option value="Breakfast">Breakfast</option>
                        <option value="Lunch">Lunch</option>
                        <option value="Dinner">Dinner</option>
                        <option value="Snack">Dessert / Snack</option>
                    </select>
                </label>
                <label class="field-label">Ranking Method
                    <select id="admin-ranking-method">
                        <option value="hybrid">Hybrid</option>
                        <option value="content-based">Content-based</option>
                        <option value="co-ordering">Co-ordering</option>
                    </select>
                </label>
            </div>
            <button class="primary-action" type="button" id="run-admin-recommendations">Run Recommendation Test</button>
            <div class="reason-box">
                <strong>Scoring formula</strong>
                <p>Hybrid score normally uses 0.45 content + 0.25 co-order + 0.20 popularity + 0.10 time/business. When liked preferences are selected, content match is weighted more strongly so matching dishes rise clearly. Disliked ingredients are excluded before ranking.</p>
            </div>
            <div class="evaluation-strip admin-evaluation-strip" id="admin-recommendation-stats"></div>
            <div class="table-wrap recommendation-results-wrap">
                <table>
                    <thead><tr><th>Dish</th><th>Category</th><th>Content</th><th>Co-order</th><th>Popularity</th><th>Time</th><th>Hybrid</th><th>Support</th><th>Confidence</th><th>Lift</th><th>Reason</th></tr></thead>
                    <tbody id="admin-recommendation-results"></tbody>
                </table>
            </div>
        </section>
        "#,
        preference_panel(options, "admin")
    )
}

fn csv_data_tools_panel() -> String {
    r#"
        <section class="admin-card">
            <div class="section-heading">
                <div>
                    <h2>CSV Import / Reload / Export</h2>
                    <p>Use compatible dishes.csv and orders.csv files. Preview validates required columns before import.</p>
                </div>
            </div>
            <p class="status-message" id="csv-import-status"></p>

            <div class="admin-two-column">
                <div class="data-tool-card">
                    <h3>Dishes CSV</h3>
                    <p class="muted">Required columns: dish_id, name, ingredients, category, tags.</p>
                    <input id="dish-csv-file" type="file" accept=".csv,text/csv">
                    <textarea id="dish-csv-import" rows="7" placeholder="Upload a CSV file or paste dishes.csv content here"></textarea>
                    <div class="csv-mode-row">
                        <label><input type="radio" name="dish-import-mode" value="replace" checked> Replace current dishes</label>
                        <label><input type="radio" name="dish-import-mode" value="merge"> Merge by dish ID</label>
                    </div>
                    <div class="form-actions">
                        <button class="primary-action" id="import-dishes-button" type="button">Import Dishes</button>
                        <button class="ghost-action" id="reload-dishes-button" type="button">Reload data/dishes.csv</button>
                        <a class="ghost-link" href="/admin/export/dishes.csv">Export Dishes</a>
                    </div>
                    <div class="csv-preview" id="dish-csv-preview"></div>
                </div>

                <div class="data-tool-card">
                    <h3>Orders CSV</h3>
                    <p class="muted">Required columns: order_id, session_user_id, ordered_dishes, timestamp.</p>
                    <input id="order-csv-file" type="file" accept=".csv,text/csv">
                    <textarea id="order-csv-import" rows="7" placeholder="Upload a CSV file or paste orders.csv content here"></textarea>
                    <div class="form-actions">
                        <button class="primary-action" id="import-orders-button" type="button">Import Orders</button>
                        <button class="ghost-action" id="reload-orders-button" type="button">Reload data/orders.csv</button>
                        <a class="ghost-link" href="/admin/export/orders.csv">Export Historical Orders</a>
                        <a class="ghost-link" href="/admin/export/completed-session-orders.csv">Export Completed Session Orders</a>
                    </div>
                    <div class="csv-preview" id="order-csv-preview"></div>
                </div>
            </div>
        </section>
    "#
    .to_string()
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
