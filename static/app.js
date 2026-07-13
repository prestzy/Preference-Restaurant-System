const CART_KEY = "fyp_web_cart_v1";
const LAST_ORDER_KEY = "fyp_last_order_id_v1";

function readCart() {
  try {
    return JSON.parse(localStorage.getItem(CART_KEY)) || {};
  } catch {
    return {};
  }
}

function writeCart(cart) {
  localStorage.setItem(CART_KEY, JSON.stringify(cart));
  updateCartCount();
  refreshCustomerRecommendations();
}

function cartCount(cart = readCart()) {
  return Object.values(cart).reduce((sum, quantity) => sum + Number(quantity || 0), 0);
}

function updateCartCount() {
  document.querySelectorAll("[data-cart-count]").forEach((element) => {
    element.textContent = cartCount().toString();
  });
}

function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

function formatList(values) {
  return values && values.length ? values.join(", ") : "-";
}

function dishById(dishId) {
  return (window.MENU_DISHES || []).find((dish) => dish.dish_id === dishId);
}

function recommendationByDishId(dishId) {
  return (window.RECOMMENDATIONS || []).find((item) => item.dish.dish_id === dishId);
}

function parsePriceAmount(dish) {
  return Number(dish?.price_amount || 0);
}

function imageHtml(dish, extraClass = "") {
  if (dish?.image_url) {
    return `<div class="dish-art ${extraClass}"><img src="${escapeHtml(dish.image_url)}" alt="${escapeHtml(dish.name)}"></div>`;
  }
  return `<div class="dish-art placeholder ${extraClass}" aria-label="No image">🍽</div>`;
}

function dishSearchHaystack(dish) {
  return [
    dish?.name,
    dish?.dish_id,
    dish?.category,
    ...(dish?.ingredients || []),
    ...(dish?.tags || []),
  ]
    .join(" ")
    .toLowerCase();
}

function dishMatchReason(dish, terms) {
  const lowerTerms = terms.map((term) => term.toLowerCase());
  const name = String(dish.name || "").toLowerCase();
  const category = String(dish.category || "").toLowerCase();

  for (const term of lowerTerms) {
    const ingredient = (dish.ingredients || []).find((value) =>
      String(value).toLowerCase().includes(term)
    );
    if (ingredient) {
      return `ingredient: ${ingredient}`;
    }

    const tag = (dish.tags || []).find((value) => String(value).toLowerCase().includes(term));
    if (tag) {
      return `tag: ${tag}`;
    }

    if (category.includes(term)) {
      return `category: ${dish.category}`;
    }

    if (name.includes(term)) {
      return "dish name match";
    }
  }

  return "menu match";
}

function addToCart(dishId) {
  const cart = readCart();
  cart[dishId] = (cart[dishId] || 0) + 1;
  writeCart(cart);
}

function changeCartQuantity(dishId, delta) {
  const cart = readCart();
  const nextQuantity = Number(cart[dishId] || 0) + delta;
  if (nextQuantity <= 0) {
    delete cart[dishId];
  } else {
    cart[dishId] = nextQuantity;
  }
  writeCart(cart);
  renderCartPage();
}

function removeFromCart(dishId) {
  const cart = readCart();
  delete cart[dishId];
  writeCart(cart);
  renderCartPage();
}

function parseSearchTerms(query) {
  return String(query || "")
    .toLowerCase()
    .split(/[,;\n|]+|\s{2,}/)
    .map((term) => term.trim())
    .filter(Boolean);
}

function setupMenuFiltering() {
  const searchInput = document.getElementById("search-input");
  const cards = Array.from(document.querySelectorAll(".dish-card"));
  const visibleCount = document.getElementById("visible-count");
  const suggestions = document.getElementById("search-suggestions");
  let activeCategory = "all";

  if (!cards.length) {
    return;
  }

  const applyFilters = () => {
    const terms = parseSearchTerms(searchInput?.value || "");
    let count = 0;

    cards.forEach((card) => {
      const searchText = card.dataset.search || "";
      const matchesSearch =
        !terms.length || terms.every((term) => searchText.includes(term));
      const matchesCategory =
        activeCategory === "all" || card.dataset.category === activeCategory;
      const visible = matchesSearch && matchesCategory;
      card.hidden = !visible;
      if (visible) {
        count += 1;
      }
    });

    if (visibleCount) {
      visibleCount.textContent = count.toString();
    }
    renderSearchSuggestions(terms, suggestions);
  };

  searchInput?.addEventListener("input", applyFilters);

  document.querySelectorAll("[data-category-chip]").forEach((chip) => {
    chip.addEventListener("click", () => {
      document
        .querySelectorAll("[data-category-chip]")
        .forEach((item) => item.classList.remove("active"));
      chip.classList.add("active");
      activeCategory = chip.dataset.categoryChip.toLowerCase();
      applyFilters();
    });
  });

  applyFilters();
}

function renderSearchSuggestions(terms, container) {
  if (!container) {
    return;
  }

  if (!terms.length) {
    container.hidden = true;
    container.innerHTML = "";
    return;
  }

  const matches = (window.MENU_DISHES || [])
    .filter((dish) => terms.every((term) => dishSearchHaystack(dish).includes(term)))
    .slice(0, 6);

  container.hidden = false;
  if (!matches.length) {
    container.innerHTML = `<div class="suggestion-empty">No matching dishes found</div>`;
    return;
  }

  container.innerHTML = matches
    .map((dish) => {
      return `
        <button class="suggestion-item" type="button" data-suggestion-dish="${escapeHtml(dish.dish_id)}">
          ${imageHtml(dish, "suggestion-thumb")}
          <span>
            <strong>${escapeHtml(dish.name)}</strong>
            <small>${escapeHtml(dish.category)} · ${escapeHtml(dish.price)} · ${escapeHtml(dishMatchReason(dish, terms))}</small>
          </span>
        </button>
      `;
    })
    .join("");

  container.querySelectorAll("[data-suggestion-dish]").forEach((button) => {
    button.addEventListener("click", () => {
      focusDishFromSuggestion(button.dataset.suggestionDish);
      container.hidden = true;
    });
  });
}

function focusDishFromSuggestion(dishId) {
  const card = document.querySelector(`.dish-card[data-dish-id="${CSS.escape(dishId)}"]`);
  if (card) {
    card.hidden = false;
    card.scrollIntoView({ behavior: "smooth", block: "center" });
    card.classList.add("focused");
    window.setTimeout(() => card.classList.remove("focused"), 1400);
    return;
  }

  showDishDetail(dishId);
}

function setupCartButtons() {
  document.querySelectorAll("[data-add-cart]").forEach((button) => {
    if (button.dataset.bound === "true") {
      return;
    }
    button.dataset.bound = "true";
    button.addEventListener("click", () => {
      addToCart(button.dataset.addCart);
      const original = button.textContent;
      button.textContent = "Added";
      window.setTimeout(() => {
        button.textContent = original || "Add";
      }, 800);
    });
  });
}

function collectPreferences(scope) {
  const root = document.querySelector(`[data-preference-scope="${scope}"]`);
  const preferences = {
    liked_ingredients: [],
    disliked_ingredients: [],
    preferred_tags: [],
    selected_dish_ids: [],
  };

  root?.querySelectorAll(".mini-chip.active").forEach((chip) => {
    const kind = chip.dataset.preferenceKind;
    if (preferences[kind]) {
      preferences[kind].push(chip.dataset.preferenceValue);
    }
  });

  if (scope === "customer") {
    preferences.selected_dish_ids = Object.keys(readCart());
  } else {
    preferences.selected_dish_ids = Array.from(
      document.getElementById("admin-selected-dishes")?.selectedOptions || []
    ).map((option) => option.value);
    preferences.time_context = document.getElementById("admin-time-context")?.value || "Any";
    preferences.ranking_method =
      document.getElementById("admin-ranking-method")?.value || "hybrid";
  }

  return preferences;
}

function setupPreferencePanels() {
  document.querySelectorAll("[data-preference-scope]").forEach((panel) => {
    const scope = panel.dataset.preferenceScope;

    panel.querySelectorAll("[data-preference-kind]").forEach((chip) => {
      chip.addEventListener("click", () => {
        const kind = chip.dataset.preferenceKind;
        const value = chip.dataset.preferenceValue;
        chip.classList.toggle("active");

        // An ingredient cannot be both liked and disliked. The UI resolves the
        // conflict immediately so the recommender receives a clean preference
        // object without contradictory signals.
        if (kind === "liked_ingredients" && chip.classList.contains("active")) {
          panel
            .querySelector(
              `[data-preference-kind="disliked_ingredients"][data-preference-value="${CSS.escape(value)}"]`
            )
            ?.classList.remove("active");
        }
        if (kind === "disliked_ingredients" && chip.classList.contains("active")) {
          panel
            .querySelector(
              `[data-preference-kind="liked_ingredients"][data-preference-value="${CSS.escape(value)}"]`
            )
            ?.classList.remove("active");
        }

        if (scope === "customer") {
          refreshCustomerRecommendations();
        }
      });
    });

    panel.querySelector("[data-clear-preferences]")?.addEventListener("click", () => {
      panel.querySelectorAll(".mini-chip.active").forEach((chip) => {
        chip.classList.remove("active");
      });
      if (scope === "customer") {
        refreshCustomerRecommendations();
      }
    });
  });
}

async function requestRecommendations(preferences) {
  const response = await fetch("/api/recommendations", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(preferences),
  });
  return response.json();
}

async function refreshCustomerRecommendations() {
  const row = document.getElementById("recommended-row");
  if (!row) {
    return;
  }

  try {
    const result = await requestRecommendations(collectPreferences("customer"));
    window.RECOMMENDATIONS = result.recommendations || [];
    row.innerHTML = window.RECOMMENDATIONS.length
      ? window.RECOMMENDATIONS.map(renderRecommendationCard).join("")
      : `<div class="info-card slim"><strong>No recommendations yet</strong><span>Select preferences or add dishes to cart.</span></div>`;
    renderRecommendationStats(result.stats);
    applyRecommendationBadges();
    setupCartButtons();
    setupDetailButtons();
  } catch {
    row.innerHTML = `<div class="info-card slim"><strong>Recommendation refresh failed</strong><span>Please try again.</span></div>`;
  }
}

function renderRecommendationCard(recommendation) {
  const dish = recommendation.dish;
  return `
    <article class="recommendation-card" data-dish-id="${escapeHtml(dish.dish_id)}">
      ${imageHtml(dish, "compact")}
      <div class="card-body">
        <h3>${escapeHtml(dish.name)}</h3>
        <p>${escapeHtml(dish.category)}</p>
        <span class="reason">${escapeHtml(recommendation.explanation)}</span>
        <strong>${escapeHtml(dish.price)}</strong>
        <div class="card-actions">
          <button class="add-button" data-add-cart="${escapeHtml(dish.dish_id)}" type="button">Add</button>
          <button class="ghost-action" data-view-dish="${escapeHtml(dish.dish_id)}" type="button">Details</button>
        </div>
      </div>
    </article>
  `;
}

function renderRecommendationStats(stats) {
  const target = document.getElementById("recommendation-stats");
  if (!target || !stats) {
    return;
  }

  target.innerHTML = `
    <span>Filtered: <strong>${stats.filtered_dishes}</strong></span>
    <span>Excluded disliked: <strong>${stats.excluded_due_to_disliked}</strong></span>
    <span>Skipped cart dishes: <strong>${stats.skipped_selected_dishes}</strong></span>
    <span>Top-5 category diversity: <strong>${stats.diversity_count_top_5}</strong></span>
  `;
}

function applyRecommendationBadges() {
  const recommendedIds = new Set((window.RECOMMENDATIONS || []).map((item) => item.dish.dish_id));
  document.querySelectorAll(".dish-card").forEach((card) => {
    const badge = card.querySelector(".badge");
    const shouldShow = recommendedIds.has(card.dataset.dishId);
    if (badge) {
      badge.hidden = !shouldShow;
    }
  });
}

function setupDetailButtons() {
  document.querySelectorAll("[data-view-dish]").forEach((button) => {
    if (button.dataset.detailBound === "true") {
      return;
    }
    button.dataset.detailBound = "true";
    button.addEventListener("click", () => showDishDetail(button.dataset.viewDish));
  });

  document
    .querySelector("[data-close-dish-modal]")
    ?.addEventListener("click", () => closeDishDetail());
}

function showDishDetail(dishId) {
  const dish = dishById(dishId) || recommendationByDishId(dishId)?.dish;
  const dialog = document.getElementById("dish-detail-modal");
  const content = document.getElementById("dish-detail-content");
  if (!dish || !dialog || !content) {
    return;
  }

  const recommendation = recommendationByDishId(dishId);
  content.innerHTML = `
    <div class="detail-layout">
      ${imageHtml(dish, "large")}
      <div>
        <p class="eyebrow">${escapeHtml(dish.dish_id)} · ${escapeHtml(dish.category)}</p>
        <h2>${escapeHtml(dish.name)}</h2>
        <p><strong>Ingredients:</strong> ${escapeHtml((dish.ingredients || []).join(", "))}</p>
        <p><strong>Tags:</strong> ${escapeHtml(formatList(dish.tags || []))}</p>
        <p><strong>Price:</strong> ${escapeHtml(dish.price)}</p>
        ${
          recommendation
            ? `<div class="reason-box">
                <strong>Recommendation reason</strong>
                <p>${escapeHtml(recommendation.explanation)}</p>
                <p>Content ${recommendation.content_score.toFixed(2)} · Co-order ${recommendation.co_order_score.toFixed(2)} · Popularity ${recommendation.popularity_score.toFixed(2)} · Time ${recommendation.business_rule_score.toFixed(2)} · Hybrid ${recommendation.hybrid_score.toFixed(2)}</p>
              </div>`
            : ""
        }
        <button class="primary-action" data-add-cart="${escapeHtml(dish.dish_id)}" type="button">Add to Cart</button>
      </div>
    </div>
  `;
  setupCartButtons();

  if (typeof dialog.showModal === "function") {
    dialog.showModal();
  } else {
    dialog.setAttribute("open", "open");
  }
}

function closeDishDetail() {
  const dialog = document.getElementById("dish-detail-modal");
  if (!dialog) {
    return;
  }
  if (typeof dialog.close === "function") {
    dialog.close();
  } else {
    dialog.removeAttribute("open");
  }
}

function renderCartPage() {
  const container = document.getElementById("cart-page-items");
  const totalElement = document.getElementById("cart-page-total");
  if (!container || !totalElement) {
    return;
  }

  const cart = readCart();
  const entries = Object.entries(cart);

  if (!entries.length) {
    container.innerHTML =
      '<div class="info-card"><strong>Your cart is empty</strong><span>Add dishes from Home.</span></div>';
    totalElement.textContent = "RM 0";
    return;
  }

  let total = 0;
  container.innerHTML = entries
    .map(([dishId, quantity]) => {
      const dish = dishById(dishId);
      if (!dish) {
        return "";
      }
      const lineTotal = parsePriceAmount(dish) * quantity;
      total += lineTotal;
      return `
        <div class="cart-row">
          <div class="cart-dish">
            ${imageHtml(dish, "cart-thumb")}
            <div>
              <strong>${escapeHtml(dish.name)}</strong>
              <span>${escapeHtml(dish.category)} · ${escapeHtml(dish.price)}</span>
            </div>
          </div>
          <div class="quantity-control">
            <button type="button" data-cart-decrease="${escapeHtml(dishId)}">−</button>
            <strong>${quantity}</strong>
            <button type="button" data-cart-increase="${escapeHtml(dishId)}">+</button>
          </div>
          <strong>RM ${lineTotal}</strong>
          <button class="ghost-action" type="button" data-remove-cart="${escapeHtml(dishId)}">Remove</button>
        </div>
      `;
    })
    .join("");

  totalElement.textContent = `RM ${total}`;

  document.querySelectorAll("[data-cart-decrease]").forEach((button) => {
    button.addEventListener("click", () => changeCartQuantity(button.dataset.cartDecrease, -1));
  });
  document.querySelectorAll("[data-cart-increase]").forEach((button) => {
    button.addEventListener("click", () => changeCartQuantity(button.dataset.cartIncrease, 1));
  });
  document.querySelectorAll("[data-remove-cart]").forEach((button) => {
    button.addEventListener("click", () => removeFromCart(button.dataset.removeCart));
  });
}

function setupCheckout() {
  const button = document.getElementById("checkout-button");
  const status = document.getElementById("checkout-status");
  if (!button || !status) {
    return;
  }

  button.addEventListener("click", async () => {
    const dishIds = Object.entries(readCart()).flatMap(([dishId, quantity]) =>
      Array.from({ length: Number(quantity || 0) }, () => dishId)
    );
    if (!dishIds.length) {
      status.textContent = "Add at least one dish before checkout.";
      return;
    }

    button.disabled = true;
    status.textContent = "Placing prototype order...";

    try {
      const response = await fetch("/api/orders", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ dish_ids: dishIds }),
      });
      const result = await response.json();
      status.textContent = result.message;

      if (result.ok) {
        if (result.order_id) {
          localStorage.setItem(LAST_ORDER_KEY, result.order_id);
        }
        writeCart({});
        renderCartPage();
      }
    } catch {
      status.textContent = "Checkout failed. Please try again.";
    } finally {
      button.disabled = false;
    }
  });
}

function setupAdminTools() {
  setupAdminOrderStatus();
  setupDishManagement();
  setupAdminRecommendationTester();
}

function setupAdminOrderStatus() {
  document.querySelectorAll("[data-order-status]").forEach((select) => {
    select.addEventListener("change", async () => {
      const orderId = select.dataset.orderStatus;
      const response = await fetch(`/api/admin/orders/${encodeURIComponent(orderId)}/status`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ status: select.value }),
      });
      const result = await response.json();
      if (result.ok && (select.value === "Completed" || select.value === "Cancelled")) {
        // Completed/cancelled orders leave the active live-order table. Reloading
        // keeps the admin view honest and makes the completed-session section
        // update immediately without pretending persistence exists.
        window.setTimeout(() => window.location.reload(), 400);
      }
    });
  });
}

function setupDishManagement() {
  const form = document.getElementById("dish-form");
  const status = document.getElementById("dish-management-status");

  form?.addEventListener("submit", async (event) => {
    event.preventDefault();
    const formData = new FormData(form);
    const payload = Object.fromEntries(formData.entries());
    const response = await fetch("/api/admin/dishes", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    const result = await response.json();
    if (status) {
      status.textContent = result.message;
    }
    if (result.ok) {
      window.setTimeout(() => window.location.reload(), 600);
    }
  });

  document.querySelectorAll("[data-toggle-dish]").forEach((button) => {
    button.addEventListener("click", async () => {
      const response = await fetch(
        `/api/admin/dishes/${encodeURIComponent(button.dataset.toggleDish)}/availability`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ available: button.dataset.available === "true" }),
        }
      );
      const result = await response.json();
      if (status) {
        status.textContent = result.message;
      }
      if (result.ok) {
        window.setTimeout(() => window.location.reload(), 500);
      }
    });
  });

  document.querySelectorAll("[data-delete-dish]").forEach((button) => {
    button.addEventListener("click", async () => {
      if (!window.confirm("Delete this dish from the in-memory menu?")) {
        return;
      }
      const response = await fetch(
        `/api/admin/dishes/${encodeURIComponent(button.dataset.deleteDish)}/delete`,
        { method: "POST" }
      );
      const result = await response.json();
      if (status) {
        status.textContent = result.message;
      }
      if (result.ok) {
        window.setTimeout(() => window.location.reload(), 500);
      }
    });
  });
}

function setupCsvTools() {
  const status = document.getElementById("csv-import-status");
  setupCsvFileInput("dish-csv-file", "dish-csv-import", "dish-csv-preview", [
    "dish_id",
    "name",
    "ingredients",
    "category",
    "tags",
  ]);
  setupCsvFileInput("order-csv-file", "order-csv-import", "order-csv-preview", [
    "order_id",
    "session_user_id",
    "ordered_dishes",
    "timestamp",
  ]);

  document.getElementById("import-dishes-button")?.addEventListener("click", async () => {
    const csv = document.getElementById("dish-csv-import")?.value || "";
    const mode =
      document.querySelector('input[name="dish-import-mode"]:checked')?.value || "replace";
    const result = await postCsvImport("/api/admin/import/dishes", csv, mode);
    if (status) {
      status.textContent = result.message;
    }
    if (result.ok) {
      window.setTimeout(() => window.location.reload(), 700);
    }
  });

  document.getElementById("import-orders-button")?.addEventListener("click", async () => {
    const csv = document.getElementById("order-csv-import")?.value || "";
    const result = await postCsvImport("/api/admin/import/orders", csv);
    if (status) {
      status.textContent = result.message;
    }
    if (result.ok) {
      window.setTimeout(() => window.location.reload(), 700);
    }
  });

  document.getElementById("reload-dishes-button")?.addEventListener("click", async () => {
    const result = await postJson("/api/admin/reload/dishes", {});
    if (status) {
      status.textContent = result.message;
    }
    if (result.ok) {
      window.setTimeout(() => window.location.reload(), 700);
    }
  });

  document.getElementById("reload-orders-button")?.addEventListener("click", async () => {
    const result = await postJson("/api/admin/reload/orders", {});
    if (status) {
      status.textContent = result.message;
    }
    if (result.ok) {
      window.setTimeout(() => window.location.reload(), 700);
    }
  });
}

async function postJson(url, payload) {
  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
  return response.json();
}

async function postCsvImport(url, csv, mode = "replace") {
  return postJson(url, { csv, mode });
}

function setupCsvFileInput(inputId, textareaId, previewId, requiredColumns) {
  const input = document.getElementById(inputId);
  const textarea = document.getElementById(textareaId);
  const preview = document.getElementById(previewId);
  if (!input || !textarea || !preview) {
    return;
  }

  input.addEventListener("change", async () => {
    const file = input.files?.[0];
    if (!file) {
      return;
    }
    const csv = await file.text();
    textarea.value = csv;
    previewCsv(csv, preview, requiredColumns);
  });

  textarea.addEventListener("input", () => previewCsv(textarea.value, preview, requiredColumns));
}

function previewCsv(csv, target, requiredColumns) {
  const rows = csv
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
  if (!rows.length) {
    target.innerHTML = "";
    return;
  }

  const headers = splitCsvPreviewRow(rows[0]).map((value) => value.trim());
  const normalizedHeaders = headers.map((value) => value.toLowerCase());
  const missing = requiredColumns.filter((column) => !normalizedHeaders.includes(column));
  const bodyRows = rows.slice(1, 6).map(splitCsvPreviewRow);

  const warning = missing.length
    ? `<p class="csv-error">Missing required column(s): ${escapeHtml(missing.join(", "))}</p>`
    : `<p class="csv-ok">Preview ready: ${rows.length - 1} record(s) detected.</p>`;

  const headerHtml = headers.map((header) => `<th>${escapeHtml(header)}</th>`).join("");
  const bodyHtml = bodyRows
    .map((row) => `<tr>${headers.map((_, index) => `<td>${escapeHtml(row[index] || "")}</td>`).join("")}</tr>`)
    .join("");

  target.innerHTML = `
    ${warning}
    <div class="table-wrap csv-preview-table">
      <table>
        <thead><tr>${headerHtml}</tr></thead>
        <tbody>${bodyHtml}</tbody>
      </table>
    </div>
  `;
}

function splitCsvPreviewRow(row) {
  const values = [];
  let current = "";
  let inQuotes = false;

  for (let index = 0; index < row.length; index += 1) {
    const character = row[index];
    if (character === '"') {
      inQuotes = !inQuotes;
      continue;
    }
    if (character === "," && !inQuotes) {
      values.push(current);
      current = "";
      continue;
    }
    current += character;
  }

  values.push(current);
  return values;
}

async function setupCustomerOrderStatus() {
  const target = document.getElementById("latest-order-status");
  if (!target) {
    return;
  }

  const orderId = localStorage.getItem(LAST_ORDER_KEY);
  if (!orderId) {
    return;
  }

  try {
    const response = await fetch(`/api/orders/${encodeURIComponent(orderId)}`);
    const result = await response.json();
    if (!result.ok || !result.order) {
      target.innerHTML = `<strong>${escapeHtml(orderId)}</strong><span>${escapeHtml(result.message)}</span>`;
      return;
    }

    const order = result.order;
    target.innerHTML = `
      <strong>${escapeHtml(order.order_id)} · ${escapeHtml(order.status)}</strong>
      <span>${escapeHtml((order.dish_names || []).join(", "))}</span>
      <span>${escapeHtml(order.total_price)} · ${escapeHtml(order.timestamp)}</span>
    `;
  } catch {
    target.innerHTML = `<strong>${escapeHtml(orderId)}</strong><span>Unable to fetch latest order status.</span>`;
  }
}

function setupAdminRecommendationTester() {
  const button = document.getElementById("run-admin-recommendations");
  const tableBody = document.getElementById("admin-recommendation-results");
  if (!button || !tableBody) {
    return;
  }

  button.addEventListener("click", async () => {
    tableBody.innerHTML = `<tr><td colspan="11">Running recommendation test...</td></tr>`;
    const result = await requestRecommendations(collectPreferences("admin"));
    const rows = (result.recommendations || []).slice(0, 12).map((item) => {
      return `
        <tr>
          <td><strong>${escapeHtml(item.dish.name)}</strong><span>${escapeHtml(item.dish.dish_id)}</span></td>
          <td>${escapeHtml(item.dish.category)}</td>
          <td>${item.content_score.toFixed(2)}</td>
          <td>${item.co_order_score.toFixed(2)}</td>
          <td>${item.popularity_score.toFixed(2)}</td>
          <td>${item.business_rule_score.toFixed(2)}</td>
          <td>${item.hybrid_score.toFixed(2)}</td>
          <td>${item.association_support.toFixed(2)}<span>${item.association_pair_count} pair(s)</span></td>
          <td>${item.association_confidence.toFixed(2)}</td>
          <td>${item.association_lift.toFixed(2)}<span>${escapeHtml(item.association_base_dish_id || "-")}</span></td>
          <td>
            ${escapeHtml(item.explanation)}
            <br><span class="muted">Liked: ${escapeHtml(formatList(item.matched_liked_ingredients))}</span>
            <br><span class="muted">Tags: ${escapeHtml(formatList(item.matched_preferred_tags))}</span>
            <br><span class="muted">Co-order: ${escapeHtml(formatList(item.related_selected_dishes))}</span>
          </td>
        </tr>
      `;
    });
    tableBody.innerHTML = rows.length
      ? rows.join("")
      : `<tr><td colspan="11">No recommendation evidence for this test input.</td></tr>`;
  });
}

document.addEventListener("DOMContentLoaded", () => {
  updateCartCount();
  setupMenuFiltering();
  setupPreferencePanels();
  setupCartButtons();
  setupDetailButtons();
  renderCartPage();
  setupCheckout();
  setupAdminTools();
  setupCustomerOrderStatus();
  refreshCustomerRecommendations();
});
