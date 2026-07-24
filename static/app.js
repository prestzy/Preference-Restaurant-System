const CART_KEY = "fyp_web_cart_v1";
const LAST_ORDER_KEY = "fyp_last_order_id_v1";
const CUSTOMER_KEY = "fyp_customer_identity_v1";

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

function readCustomerIdentity() {
  return window.CUSTOMER_SESSION || {};
}

function writeCustomerIdentity(identity) {
  // Kept only as a harmless browser mirror for older pages. The server-side
  // customer cookie/session is now the authority for checkout and profile.
  localStorage.setItem(CUSTOMER_KEY, JSON.stringify(identity));
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

function shortRecommendationReason(recommendation) {
  const parts = [];
  const liked = [...(recommendation.matched_liked_ingredients || [])];
  const tags = [...(recommendation.matched_preferred_tags || [])];
  if (liked.length || tags.length) {
    parts.push(`Matches ${formatList([...liked, ...tags].slice(0, 3))}`);
  }
  if ((recommendation.related_selected_dishes || []).length) {
    parts.push(`Often ordered with ${recommendation.related_selected_dishes[0]}`);
  }
  if (recommendation.popularity_score > 0 && !parts.length) {
    parts.push("Popular from order history");
  }
  if (recommendation.business_rule_score > 0 && parts.length < 2) {
    parts.push("Fits the menu context");
  }
  return parts.length ? parts.join(" · ") : "Based on ingredients and order patterns";
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
    .map(normalizeSearchTerm)
    .filter(Boolean);
}

function normalizeSearchTerm(term) {
  const cleaned = String(term || "").trim().toLowerCase().replace(/\s+/g, " ");
  if (cleaned === "noodles") return "noodle";
  if (cleaned === "fruits") return "fruit";
  if (cleaned === "chillies" || cleaned === "chilli") return "chili";
  if (cleaned.endsWith("s") && cleaned.length > 4) return cleaned.slice(0, -1);
  return cleaned;
}

function expandedSearchGroups(terms) {
  const vocab = window.SEARCH_VOCABULARY || { aliases: {}, concepts: {} };
  return terms.map((rawTerm) => {
    const term = normalizeSearchTerm(rawTerm);
    const values = new Set([term]);
    const reasons = new Map();
    Object.entries(vocab.aliases || {}).forEach(([canonical, aliases]) => {
      const normalizedCanonical = normalizeSearchTerm(canonical);
      const normalizedAliases = (aliases || []).map(normalizeSearchTerm);
      if (term === normalizedCanonical || normalizedAliases.includes(term)) {
        values.add(normalizedCanonical);
        reasons.set(normalizedCanonical, term === normalizedCanonical ? `exact term: ${term}` : `${term} interpreted as ${normalizedCanonical}`);
        normalizedAliases.forEach((alias) => values.add(alias));
      }
    });
    Object.entries(vocab.concepts || {}).forEach(([concept, members]) => {
      const normalizedConcept = normalizeSearchTerm(concept);
      const normalizedMembers = (members || []).map(normalizeSearchTerm);
      if (term === normalizedConcept || normalizedMembers.includes(term)) {
        values.add(normalizedConcept);
        normalizedMembers.forEach((member) => {
          values.add(member);
          if (term !== member) {
            reasons.set(member, `${member} belongs to ${normalizedConcept} concept`);
          }
        });
      }
    });
    return { raw: term, values: Array.from(values), reasons };
  });
}

function smartSearchResult(dish, groups, mode = "all") {
  if (!groups.length) {
    return { matched: true, score: 0, reasons: [] };
  }
  const name = normalizeSearchTerm(dish.name || "");
  const id = normalizeSearchTerm(dish.dish_id || "");
  const category = normalizeSearchTerm(dish.category || "");
  const ingredients = (dish.ingredients || []).map(normalizeSearchTerm);
  const tags = (dish.tags || []).map(normalizeSearchTerm);
  const groupResults = groups.map((group) => {
    let best = null;
    for (const value of group.values) {
      let candidate = null;
      if (name === value || id === value) candidate = { score: 100, reason: `exact dish match: ${dish.name}` };
      else if (name.startsWith(value)) candidate = { score: 80, reason: `dish name starts with ${value}` };
      else if (name.includes(value)) candidate = { score: 65, reason: `dish name contains ${value}` };
      else if (ingredients.includes(value)) candidate = { score: value === group.raw ? 55 : 48, reason: group.reasons.get(value) || `ingredient: ${value}` };
      else if (tags.includes(value)) candidate = { score: value === group.raw ? 45 : 38, reason: group.reasons.get(value) || `tag: ${value}` };
      else if (ingredients.some((ingredient) => ingredient.includes(value))) candidate = { score: 38, reason: group.reasons.get(value) || `ingredient concept: ${value}` };
      else if (tags.some((tag) => tag.includes(value))) candidate = { score: 38, reason: group.reasons.get(value) || `tag concept: ${value}` };
      else if (category.includes(value)) candidate = { score: 30, reason: `category: ${dish.category}` };
      if (candidate && (!best || candidate.score > best.score)) {
        best = candidate;
      }
    }
    return best;
  });
  const matched = mode === "any" ? groupResults.some(Boolean) : groupResults.every(Boolean);
  const score = groupResults.reduce((sum, item) => sum + (item?.score || 0), 0) + Math.min(10, Number(dish.price_amount || 0) % 10);
  const reasons = groupResults.filter(Boolean).map((item) => item.reason);
  return { matched, score, reasons };
}

function setupDishLocator() {
  // Search is a locator only. It calls the Rust search API to populate the
  // suggestion dropdown, then suggestion clicks scroll to the existing static
  // Menu card. This function intentionally never changes Menu card visibility,
  // Menu card order, or the server-rendered Menu count.
  const searchInput = document.getElementById("search-input");
  const suggestions = document.getElementById("search-suggestions");
  let latestSearch = { query: "", results: [] };
  let searchSequence = 0;

  if (!searchInput || !suggestions) {
    return;
  }

  const updateSuggestions = async () => {
    const query = searchInput.value || "";
    const hasQuery = parseSearchTerms(query).length > 0;
    const sequence = ++searchSequence;

    if (!hasQuery) {
      latestSearch = { query: "", results: [] };
      renderSearchSuggestions(latestSearch, suggestions, false);
      return;
    }

    try {
      const response = await fetch(`/api/search?q=${encodeURIComponent(query)}&mode=all`);
      const payload = await response.json();
      if (sequence !== searchSequence || !payload.ok || !payload.data) {
        return;
      }
      latestSearch = payload.data;
    } catch (error) {
      console.error("Menu search failed", error);
      return;
    }
    renderSearchSuggestions(latestSearch, suggestions, hasQuery);
  };

  const debouncedUpdateSuggestions = debounce(updateSuggestions, 160);
  searchInput?.addEventListener("input", debouncedUpdateSuggestions);

  renderSearchSuggestions(latestSearch, suggestions, false);
}

function debounce(callback, delay) {
  let timer = null;
  return (...args) => {
    window.clearTimeout(timer);
    timer = window.setTimeout(() => callback(...args), delay);
  };
}

function setupCarouselControls() {
  document.querySelectorAll("[data-carousel-scroll]").forEach((button) => {
    const row = document.getElementById(button.dataset.carouselScroll);
    if (!row || button.dataset.carouselBound === "true") {
      return;
    }
    button.dataset.carouselBound = "true";

    const updateDisabledState = () => {
      const controls = document.querySelectorAll(
        `[data-carousel-scroll="${CSS.escape(button.dataset.carouselScroll)}"]`
      );
      const maxScroll = row.scrollWidth - row.clientWidth - 2;
      controls.forEach((control) => {
        const direction = Number(control.dataset.direction || 1);
        control.disabled =
          maxScroll <= 0 ||
          (direction < 0 && row.scrollLeft <= 2) ||
          (direction > 0 && row.scrollLeft >= maxScroll);
      });
    };

    button.addEventListener("click", () => {
      const direction = Number(button.dataset.direction || 1);
      row.scrollBy({
        left: direction * Math.max(220, Math.floor(row.clientWidth * 0.8)),
        behavior: "smooth",
      });
      window.setTimeout(updateDisabledState, 280);
    });
    row.addEventListener("scroll", updateDisabledState, { passive: true });
    window.addEventListener("resize", updateDisabledState);
    updateDisabledState();
  });
}

function setupOrderFilters() {
  document.querySelectorAll("[data-order-filter]").forEach((button) => {
    if (button.dataset.orderFilterBound === "true") return;
    button.dataset.orderFilterBound = "true";
    button.addEventListener("click", () => {
      document
        .querySelectorAll("[data-order-filter]")
        .forEach((item) => item.classList.remove("active"));
      button.classList.add("active");
      const filter = button.dataset.orderFilter;
      document.querySelectorAll("[data-order-status-card]").forEach((card) => {
        card.hidden = filter !== "all" && card.dataset.orderStatusCard !== filter;
      });
    });
  });
}

function renderSearchSuggestions(searchResult, container, hasQuery) {
  if (!container) {
    return;
  }

  if (!hasQuery) {
    container.hidden = true;
    container.innerHTML = "";
    return;
  }

  const matches = (searchResult.results || []).slice(0, 6);

  container.hidden = false;
  if (!matches.length) {
    container.innerHTML = `<div class="suggestion-empty">No matching dishes found</div>`;
    return;
  }

  container.innerHTML = matches
    .map((item) => {
      const dish = item.dish;
      return `
        <button class="suggestion-item" type="button" data-suggestion-dish="${escapeHtml(dish.dish_id)}">
          ${imageHtml(dish, "suggestion-thumb")}
          <span>
            <strong>${escapeHtml(dish.name)}</strong>
            <small>${escapeHtml(dish.category)} · ${escapeHtml(dish.price)} · ${escapeHtml((item.match_reasons || []).slice(0, 2).join("; ") || "menu match")}</small>
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
  const card =
    document.getElementById(`dish-${dishId}`) ||
    document.querySelector(`.dish-card[data-dish-id="${CSS.escape(dishId)}"]`);
  if (card) {
    card.scrollIntoView({ behavior: "smooth", block: "center" });
    card.classList.add("dish-locator-highlight");
    window.setTimeout(() => card.classList.remove("dish-locator-highlight"), 2000);
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
      document.querySelectorAll("[data-admin-context-dish]:checked")
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
    window.dispatchEvent(new Event("resize"));
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
        <span class="reason">${escapeHtml(shortRecommendationReason(recommendation))}</span>
        <strong>${escapeHtml(dish.price)}</strong>
        <div class="card-actions">
          <button class="add-button" data-add-cart="${escapeHtml(dish.dish_id)}" type="button">Add</button>
          <button class="ghost-action" data-view-dish="${escapeHtml(dish.dish_id)}" type="button">Why this?</button>
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

  const matched = Number(stats.matched_preferences || 0);
  const shown = Number(stats.recommended_shown || (window.RECOMMENDATIONS || []).length);
  const fallback =
    matched === 0 && shown > 0
      ? `<span>No exact preference match found. Showing popular alternatives.</span>`
      : "";
  target.innerHTML = `
    <span>Eligible dishes: <strong>${stats.eligible_dishes ?? stats.filtered_dishes}</strong></span>
    <span>Matched preferences: <strong>${matched}</strong></span>
    <span>Excluded disliked: <strong>${stats.excluded_due_to_disliked}</strong></span>
    <span>Skipped cart dishes: <strong>${stats.skipped_selected_dishes}</strong></span>
    <span>Recommended shown: <strong>${shown}</strong></span>
    ${fallback}
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
                <strong>Why recommended?</strong>
                <p>${escapeHtml(recommendation.explanation)}</p>
                <details>
                  <summary>Technical score breakdown</summary>
                  <p>Content ${recommendation.content_score.toFixed(2)} · Co-order ${recommendation.co_order_score.toFixed(2)} · Popularity ${recommendation.popularity_score.toFixed(2)} · Time ${recommendation.business_rule_score.toFixed(2)} · Hybrid ${recommendation.hybrid_score.toFixed(2)}</p>
                  <p>Support ${recommendation.association_support.toFixed(2)} · Confidence ${recommendation.association_confidence.toFixed(2)} · Lift ${recommendation.association_lift.toFixed(2)}</p>
                </details>
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
    const identity = readCustomerIdentity();
    if (!identity.session_id) {
      status.textContent = "Your customer session expired. Please register again.";
      return;
    }
    const note = document.getElementById("customer-note")?.value.trim() || "";

    button.disabled = true;
    status.textContent = "Placing order...";

    try {
      const response = await fetch("/api/orders", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ dish_ids: dishIds, note }),
      });
      const result = await response.json().catch(() => ({
        ok: false,
        message: "Checkout failed because the server returned an unreadable response.",
      }));
      status.textContent = result.message || "Checkout failed. Please try again.";

      if (result.ok) {
        if (result.order_id) {
          localStorage.setItem(LAST_ORDER_KEY, result.order_id);
        }
        writeCart({});
        renderCartPage();
        window.setTimeout(() => {
          window.location.href = "/profile";
        }, 700);
      }
    } catch (error) {
      console.error("Checkout request failed", error);
      status.textContent = "We could not reach the order server. Please try again.";
    } finally {
      button.disabled = false;
    }
  });
}

function setupSmartMenuAssistant() {
  const prompt = document.getElementById("search-input");
  const button = document.getElementById("assistant-run");
  const understood = document.getElementById("assistant-understood");
  const upsells = document.getElementById("assistant-upsells");
  const row = document.getElementById("recommended-row");
  if (!prompt || !button || !understood || !row) {
    // The Home search bar is now a dish locator only. When the optional
    // assistant button is not rendered, typing should only show search
    // suggestions and must not update recommendations or the static Menu grid.
    return;
  }

  const runAssistant = async () => {
    const text = prompt.value.trim();
    if (!text) {
      understood.textContent = "Type a dish, ingredient, or preference first, for example: spicy chicken but no beef.";
      return;
    }

    button.disabled = true;
    understood.textContent = "Understanding your request...";
    try {
      const response = await fetch("/api/assistant/recommendations", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          prompt: text,
          selected_dish_ids: Object.keys(readCart()),
        }),
      });
      const result = await response.json();
      understood.textContent = result.understood || "Assistant could not match menu terms.";
      window.RECOMMENDATIONS = result.recommendations || [];
      row.innerHTML = window.RECOMMENDATIONS.length
        ? window.RECOMMENDATIONS.map(renderRecommendationCard).join("")
        : `<div class="info-card slim"><strong>No assistant recommendations</strong><span>Try menu words such as chicken, rice, spicy, dessert, or no beef.</span></div>`;
      if (upsells) {
        upsells.innerHTML = (result.upsells || []).length
          ? (result.upsells || [])
              .map((item) => `<span class="assistant-pill">${escapeHtml(item)}</span>`)
              .join("")
          : "";
      }
      renderRecommendationStats(result.stats);
      applyRecommendationBadges();
    setupCartButtons();
    setupDetailButtons();
      window.dispatchEvent(new Event("resize"));
    } catch {
      understood.textContent = "Assistant request failed. Please try again.";
    } finally {
      button.disabled = false;
    }
  };

  button.addEventListener("click", runAssistant);
  prompt.addEventListener("keydown", (event) => {
    if (event.key === "Enter") {
      event.preventDefault();
      runAssistant();
    }
  });
}

function setupAdminTools() {
  setupAdminOrderStatus();
  setupAdminOrderPolling();
  setupDishManagement();
  setupCsvTools();
  setupAdminRecommendationTester();
  setupAdminInsights();
  setupSimulationTester();
}

function setupAdminOrderStatus() {
  const statusMessage = document.getElementById("admin-order-status");
  document.querySelectorAll("[data-order-status]").forEach((select) => {
    select.addEventListener("change", async () => {
      const orderId = select.dataset.orderStatus;
      const response = await fetch(`/api/admin/orders/${encodeURIComponent(orderId)}/status`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ status: select.value }),
      });
      const result = await response.json();
      if (statusMessage) {
        statusMessage.textContent = result.message || "";
      }
      if (result.ok && (select.value === "Completed" || select.value === "Cancelled")) {
        // Completed orders are now written into data/orders.csv before this
        // success path returns. Reloading refreshes both the active table and
        // the historical table so staff see the persisted order immediately.
        window.setTimeout(() => window.location.reload(), 900);
      }
    });
  });
}

function setupAdminOrderPolling() {
  const body = document.getElementById("admin-live-orders-body");
  const statusMessage = document.getElementById("admin-order-status");
  if (!body || body.dataset.pollingBound === "true") return;
  body.dataset.pollingBound = "true";
  let lastVersion = null;
  const load = async () => {
    const response = await fetch("/api/admin/orders");
    const result = await response.json();
    if (!result.ok || !result.data) {
      if (statusMessage) statusMessage.textContent = result.message || "Unable to refresh orders.";
      return;
    }
    if (lastVersion === result.data.version) return;
    const hadVersion = lastVersion !== null;
    lastVersion = result.data.version;
    const active = (result.data.orders || []).filter((order) => !["Completed", "Cancelled"].includes(String(order.status)));
    body.innerHTML = active.length
      ? active.map(renderAdminOrderRow).join("")
      : `<tr><td colspan="11">No active live customer orders yet.</td></tr>`;
    setupAdminOrderStatus();
    if (hadVersion && statusMessage) statusMessage.textContent = `Order list updated: ${result.data.updated_at}`;
  };
  load();
  window.setInterval(() => {
    if (!document.hidden) load();
  }, 4000);
}

function renderAdminOrderRow(order) {
  const statuses = ["Pending", "Preparing", "Ready", "Completed", "Cancelled"];
  return `
    <tr class="admin-live-order-row">
      <td data-label="Order ID">${escapeHtml(order.order_id)}</td>
      <td data-label="Session">${escapeHtml(order.session_user_id)}</td>
      <td data-label="Customer">${escapeHtml(order.customer_name || "-")}</td>
      <td data-label="Phone">${escapeHtml(order.customer_phone || "-")}</td>
      <td data-label="Table">${escapeHtml(order.table_number || "-")}</td>
      <td data-label="Dish IDs">${escapeHtml((order.ordered_dishes || []).join(", "))}</td>
      <td data-label="Dish Names">${escapeHtml((order.dish_names || []).join(", "))}</td>
      <td data-label="Note">${escapeHtml(order.note || "-")}</td>
      <td data-label="Time">${escapeHtml(order.timestamp || "-")}</td>
      <td data-label="Total">${escapeHtml(order.total_price || "-")}</td>
      <td data-label="Status"><select data-order-status="${escapeHtml(order.order_id)}">${statuses.map((status) => `<option value="${status}" ${status === order.status ? "selected" : ""}>${status}</option>`).join("")}</select></td>
    </tr>
  `;
}

function setupDishManagement() {
  const form = document.getElementById("dish-form");
  const status = document.getElementById("dish-management-status");
  const modal = document.getElementById("dish-form-modal");
  const title = document.getElementById("dish-form-title");
  const search = document.getElementById("admin-dish-search");
  const availability = document.getElementById("admin-availability-filter");
  const tableBody = document.getElementById("admin-dish-table-body");
  const emptyState = document.getElementById("admin-dish-empty");
  if (!form || !modal || !tableBody) return;

  const setStatus = (message, isError = false) => {
    if (!status) return;
    status.textContent = message || "";
    status.classList.toggle("error", isError);
  };

  const openForm = (mode, dish = null) => {
    form.reset();
    form.dataset.mode = mode;
    if (title) title.textContent = mode === "edit" ? "Edit Dish" : "Add Dish";
    if (form.elements.dish_id) {
      // Dish IDs connect menu rows with historical order logs. Edit mode keeps
      // the ID visible but read-only so staff can change dish details without
      // accidentally breaking existing co-order evidence.
      form.elements.dish_id.readOnly = mode === "edit";
    }
    const submitButton = form.querySelector('button[type="submit"]');
    if (submitButton) {
      submitButton.textContent = mode === "edit" ? "Save Changes" : "Add Dish";
    }
    form.elements.available.checked = true;
    if (dish) {
      form.elements.dish_id.value = dish.dish_id || "";
      form.elements.name.value = dish.name || "";
      form.elements.price.value = dish.price_amount || "";
      form.elements.category.value = dish.category || "";
      form.elements.ingredients.value = (dish.ingredients || []).join(", ");
      form.elements.tags.value = (dish.tags || []).join(", ");
      form.elements.image_path.value = dish.image_path || "";
      form.elements.available.checked = dish.available !== false;
    }
    modal.hidden = false;
  };

  document.getElementById("open-dish-form")?.addEventListener("click", () => openForm("add"));
  document.getElementById("cancel-dish-form")?.addEventListener("click", () => {
    modal.hidden = true;
  });

  const filterRows = () => {
    const term = (search?.value || "").trim().toLowerCase();
    const mode = availability?.value || "all";
    let visible = 0;
    document.querySelectorAll("[data-admin-dish-row]").forEach((row) => {
      const haystack = [
        row.dataset.adminDishRow,
        row.dataset.dishName,
        row.dataset.dishCategory,
        row.dataset.dishIngredients,
        row.dataset.dishTags,
      ].join(" ").toLowerCase();
      const availableText = row.dataset.dishAvailable === "true" ? "available" : "unavailable";
      row.hidden = (term && !haystack.includes(term)) || (mode !== "all" && mode !== availableText);
      if (!row.hidden) visible += 1;
    });
    if (emptyState) emptyState.hidden = visible !== 0;
  };
  search?.addEventListener("input", filterRows);
  availability?.addEventListener("change", filterRows);

  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    const formData = new FormData(form);
    const payload = Object.fromEntries(formData.entries());
    payload.available = formData.has("available");
    const isEdit = form.dataset.mode === "edit";
    const dishId = String(payload.dish_id || "").trim();
    const endpoint = isEdit
      ? `/api/admin/dishes/${encodeURIComponent(dishId)}`
      : "/api/admin/dishes";
    const submitButton = form.querySelector('button[type="submit"]');
    submitButton.disabled = true;
    try {
      const response = await fetch(endpoint, {
        method: isEdit ? "PUT" : "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });
      const result = await response.json();
      setStatus(result.message, !result.ok);
      if (result.ok) {
        modal.hidden = true;
        // The server is the source of truth for image lookup and display
        // pricing. Reloading after a successful mutation keeps the row markup
        // consistent while all interaction bindings remain delegated.
        window.setTimeout(() => window.location.reload(), 450);
      }
    } catch {
      setStatus("Dish request failed. Please try again.", true);
    } finally {
      submitButton.disabled = false;
    }
  });

  // Event delegation keeps actions working even when rows are replaced after
  // a future in-place refresh or import.
  tableBody.addEventListener("click", async (event) => {
    const button = event.target.closest("button");
    if (!button) return;

    if (button.matches("[data-edit-dish]")) {
      setStatus("Loading dish...");
      try {
        const response = await fetch(
          `/api/admin/dishes/${encodeURIComponent(button.dataset.editDish)}`,
          { cache: "no-store" }
        );
        const result = await response.json();
        if (!result.ok || !result.data) {
          setStatus(result.message || "Dish could not be loaded.", true);
          return;
        }
        setStatus("");
        openForm("edit", result.data);
      } catch {
        setStatus("Dish could not be loaded.", true);
      }
      return;
    }

    if (button.matches("[data-toggle-dish]")) {
      button.disabled = true;
      const response = await fetch(
        `/api/admin/dishes/${encodeURIComponent(button.dataset.toggleDish)}/availability`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ available: button.dataset.available === "true" }),
        }
      );
      const result = await response.json();
      setStatus(result.message, !result.ok);
      if (result.ok) {
        window.setTimeout(() => window.location.reload(), 500);
      }
      button.disabled = false;
      return;
    }

    if (button.matches("[data-delete-dish]")) {
      if (!window.confirm("Permanently remove this dish from the in-memory menu? Dishes referenced by historical orders cannot be deleted.")) {
        return;
      }
      button.disabled = true;
      const response = await fetch(
        `/api/admin/dishes/${encodeURIComponent(button.dataset.deleteDish)}/delete`,
        { method: "POST" }
      );
      const result = await response.json();
      setStatus(result.message, !result.ok);
      if (result.ok) {
        button.closest("[data-admin-dish-row]")?.remove();
        filterRows();
      }
      button.disabled = false;
    }
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
  const target = document.getElementById("orders-list");
  if (!target) {
    return;
  }

  let lastVersion = null;
  let timer = null;
  const indicator = document.getElementById("order-sync-status");
  const loadOrders = async () => {
    let result = null;
    try {
      const response = await fetch("/api/customer/orders", { cache: "no-store" });
      result = await response.json();
    } catch (error) {
      console.error("Customer order polling failed", error);
      if (indicator) indicator.textContent = "Reconnecting to order status...";
      return;
    }

    if (!result.ok || !result.data) {
      if (!lastVersion) {
        target.innerHTML = `<div class="info-card"><strong>No current orders yet.</strong><span>${escapeHtml(result.message)}</span></div>`;
      }
      if (indicator) indicator.textContent = result.message || "Unable to refresh order status.";
      return;
    }
    if (!(result.data.orders || []).length) {
      target.innerHTML = `<div class="info-card"><strong>No current orders yet.</strong><span>Place an order from the cart to track it here.</span></div>`;
      return;
    }
    if (lastVersion === result.data.version) return;
    lastVersion = result.data.version;

    target.innerHTML = result.data.orders.map(renderOrderCard).join("");
    if (indicator) indicator.textContent = `Last updated: ${escapeHtml(result.data.updated_at)}`;
    setupOrderFilters();
  };

  const schedule = () => {
    window.clearTimeout(timer);
    timer = window.setTimeout(async () => {
      if (!document.hidden) await loadOrders();
      schedule();
    }, document.hidden ? 12000 : 3000);
  };

  await loadOrders();
  schedule();
  document.addEventListener("visibilitychange", () => {
    if (!document.hidden) {
      loadOrders();
    }
    schedule();
  });
}

function renderOrderCard(order) {
  const status = String(order.status || "Pending");
  const statusLower = status.toLowerCase();
  const statusMessages = {
    Pending: "Your order has been received.",
    Preparing: "The kitchen is preparing your food.",
    Ready: "Your food is ready.",
    Completed: "Your order is completed.",
    Cancelled: "Your order was cancelled. Please contact staff.",
  };
  return `
    <article class="order-card" data-order-status-card="${escapeHtml(statusLower)}">
      <div class="order-card-main">
        <div>
          <p class="eyebrow">${escapeHtml(order.order_id)}</p>
          <h2>${escapeHtml(status)} · ${escapeHtml(order.total_price)}</h2>
        </div>
        <span class="status-badge ${escapeHtml(statusLower)}">${escapeHtml(status)}</span>
      </div>
      <div class="order-progress" aria-label="Order progress">${renderOrderSteps(status)}</div>
      <p class="order-status-explanation">${escapeHtml(statusMessages[status] || "Order status updated.")}</p>
      <p><strong>Customer:</strong> ${escapeHtml(order.customer_name || "-")}</p>
      <p><strong>Dishes:</strong> ${escapeHtml((order.dish_names || []).join(", "))}</p>
      <p><strong>Time:</strong> ${escapeHtml(order.timestamp || "-")}</p>
      ${order.table_number ? `<p><strong>Table:</strong> ${escapeHtml(order.table_number)}</p>` : ""}
    </article>
  `;
}

function renderOrderSteps(status) {
  const steps = ["Pending", "Preparing", "Ready", "Completed"];
  if (status === "Cancelled") {
    return `<span class="progress-step active cancelled">Cancelled</span>`;
  }
  const activeIndex = Math.max(0, steps.indexOf(status));
  return steps
    .map((step, index) => `<span class="progress-step ${index <= activeIndex ? "active" : ""}">${escapeHtml(step)}</span>`)
    .join("");
}

function setupAdminRecommendationTester() {
  const tabs = Array.from(document.querySelectorAll("[data-experiment-tab]"));
  if (!tabs.length) return;

  const activateTab = (name, focus = false) => {
    tabs.forEach((tab) => {
      const active = tab.dataset.experimentTab === name;
      tab.classList.toggle("active", active);
      tab.setAttribute("aria-selected", String(active));
      tab.tabIndex = active ? 0 : -1;
      if (active && focus) tab.focus();
    });
    document.querySelectorAll("[data-experiment-panel]").forEach((panel) => {
      panel.hidden = panel.dataset.experimentPanel !== name;
    });
  };

  tabs.forEach((tab, index) => {
    tab.addEventListener("click", () => activateTab(tab.dataset.experimentTab));
    tab.addEventListener("keydown", (event) => {
      if (!["ArrowLeft", "ArrowRight"].includes(event.key)) return;
      event.preventDefault();
      const offset = event.key === "ArrowRight" ? 1 : -1;
      const next = tabs[(index + offset + tabs.length) % tabs.length];
      activateTab(next.dataset.experimentTab, true);
    });
  });
  activateTab("ingredient");

  const selectedValues = (kind) =>
    Array.from(document.querySelectorAll(`[data-experiment-option="${kind}"]:checked`)).map(
      (item) => item.value
    );
  const clearChecks = (prefix) => {
    document.querySelectorAll(`[data-experiment-option^="${prefix}"]`).forEach((item) => {
      item.checked = false;
    });
  };

  document.querySelectorAll("[data-experiment-option-search]").forEach((search) => {
    search.addEventListener("input", () => {
      const kind = search.dataset.experimentOptionSearch;
      const term = search.value.trim().toLowerCase();
      document
        .querySelectorAll(`[data-experiment-option-list="${kind}"] .ingredient-option`)
        .forEach((option) => {
          option.hidden = Boolean(term) && !String(option.dataset.optionLabel || "").includes(term);
        });
    });
  });

  // Liked and disliked selections are mutually exclusive in each experiment.
  document.querySelectorAll("[data-experiment-option]").forEach((input) => {
    input.addEventListener("change", () => {
      if (!input.checked) return;
      const [scope, side] = input.dataset.experimentOption.split("-");
      const opposite = side === "liked" ? "disliked" : "liked";
      document
        .querySelector(
          `[data-experiment-option="${scope}-${opposite}"][value="${CSS.escape(input.value)}"]`
        )
        ?.removeAttribute("checked");
      const oppositeInput = document.querySelector(
        `[data-experiment-option="${scope}-${opposite}"][value="${CSS.escape(input.value)}"]`
      );
      if (oppositeInput) oppositeInput.checked = false;
    });
  });

  document.querySelectorAll("[data-ingredient-preset]").forEach((button) => {
    button.addEventListener("click", () => {
      clearChecks("ingredient-");
      if (button.dataset.ingredientPreset === "all") {
        document.querySelectorAll('[data-experiment-option="ingredient-liked"]').forEach((item) => {
          item.checked = true;
        });
      }
      if (button.dataset.ingredientPreset === "example") {
        ["chicken", "rice"].forEach((value) => {
          const input = document.querySelector(
            `[data-experiment-option="ingredient-liked"][value="${CSS.escape(value)}"]`
          );
          if (input) input.checked = true;
        });
        ["beef", "anchovies"].forEach((value) => {
          const input = document.querySelector(
            `[data-experiment-option="ingredient-disliked"][value="${CSS.escape(value)}"]`
          );
          if (input) input.checked = true;
        });
      }
    });
  });

  const historicalOrder = document.getElementById("method-historical-order");
  const hiddenDish = document.getElementById("method-hidden-dish");
  const populateHiddenTargets = () => {
    if (!historicalOrder || !hiddenDish) return;
    const selected = historicalOrder.selectedOptions[0];
    const ids = String(selected?.dataset.dishIds || "")
      .split(",")
      .map((value) => value.trim())
      .filter(Boolean);
    hiddenDish.innerHTML = ids.length
      ? `<option value="">Select hidden target</option>${ids
          .map((id) => {
            const dish = (window.MENU_DISHES || []).find((item) => item.dish_id === id);
            return `<option value="${escapeHtml(id)}">${escapeHtml(dish?.name || id)} (${escapeHtml(id)})</option>`;
          })
          .join("")}`
      : `<option value="">Select an order first</option>`;
    hiddenDish.disabled = !ids.length;
  };
  historicalOrder?.addEventListener("change", populateHiddenTargets);

  const payloadFor = (type) => {
    if (type === "ingredient") {
      return {
        experiment_type: type,
        liked_ingredients: selectedValues("ingredient-liked"),
        disliked_ingredients: selectedValues("ingredient-disliked"),
        top_k: Number(document.getElementById("ingredient-top-k")?.value || 5),
      };
    }
    if (type === "coorder") {
      const anchor = document.getElementById("coorder-anchor-dish")?.value || "";
      const candidate = document.getElementById("coorder-candidate-dish")?.value || "";
      if (!anchor) throw new Error("Select an anchor dish.");
      if (!candidate) throw new Error("Select a candidate dish.");
      if (anchor === candidate) throw new Error("Anchor and candidate must be different.");
      return {
        experiment_type: type,
        anchor_dish_id: anchor,
        candidate_dish_id: candidate,
        additional_coorders: Number(document.getElementById("coorder-additional")?.value || 0),
        top_k: Number(document.getElementById("coorder-top-k")?.value || 5),
      };
    }
    const orderId = historicalOrder?.value || "";
    const target = hiddenDish?.value || "";
    if (!orderId) throw new Error("Select a historical order.");
    if (!target) throw new Error("Select a hidden target dish.");
    return {
      experiment_type: type,
      historical_order_id: orderId,
      hidden_dish_id: target,
      liked_ingredients: selectedValues("method-liked"),
      disliked_ingredients: selectedValues("method-disliked"),
      top_k: Number(document.getElementById("method-top-k")?.value || 5),
    };
  };

  document.querySelectorAll("[data-run-experiment]").forEach((button) => {
    button.addEventListener("click", async () => {
      const type = button.dataset.runExperiment;
      const target = document.getElementById(`experiment-result-${type}`);
      if (!target) return;
      let payload;
      try {
        payload = payloadFor(type);
      } catch (error) {
        target.innerHTML = experimentMessage(error.message, true);
        return;
      }
      button.disabled = true;
      const original = button.textContent;
      button.textContent = "Running...";
      target.innerHTML = experimentMessage("Running controlled experiment...");
      try {
        const response = await fetch("/api/admin/experiment-lab", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(payload),
        });
        const result = await response.json();
        if (!result.ok || !result.data) {
          target.innerHTML = experimentMessage(
            `The experiment request failed: ${result.message || "Unknown error."}`,
            true
          );
          return;
        }
        target.innerHTML = renderExperimentResult(type, result.data);
      } catch {
        target.innerHTML = experimentMessage("The experiment request failed: unable to reach the server.", true);
      } finally {
        button.disabled = false;
        button.textContent = original;
      }
    });
  });

  document.querySelectorAll("[data-clear-experiment]").forEach((button) => {
    button.addEventListener("click", () => {
      const target = document.getElementById(`experiment-result-${button.dataset.clearExperiment}`);
      if (target) target.innerHTML = "";
    });
  });

  document.querySelectorAll("[data-reset-experiment]").forEach((button) => {
    button.addEventListener("click", () => {
      const type = button.dataset.resetExperiment;
      if (type === "ingredient") {
        clearChecks("ingredient-");
        document.getElementById("ingredient-top-k").value = "5";
      } else if (type === "coorder") {
        document.getElementById("coorder-anchor-dish").value = "";
        document.getElementById("coorder-candidate-dish").value = "";
        document.getElementById("coorder-additional").value = "10";
        document.getElementById("coorder-top-k").value = "5";
      } else {
        clearChecks("method-");
        historicalOrder.value = "";
        document.getElementById("method-top-k").value = "5";
        populateHiddenTargets();
      }
      if (type === "ingredient" || type === "method") {
        document
          .querySelectorAll(`[data-experiment-option-search^="${type}-"]`)
          .forEach((search) => {
            search.value = "";
            search.dispatchEvent(new Event("input"));
          });
      }
      const target = document.getElementById(`experiment-result-${type}`);
      if (target) target.innerHTML = "";
    });
  });
}

function experimentMessage(message, isError = false) {
  return `<div class="info-card slim ${isError ? "error" : ""}"><strong>${escapeHtml(message)}</strong></div>`;
}

function renderExperimentResult(type, data) {
  const rows = data.rows || [];
  let table = "";
  if (type === "ingredient") {
    const dishes = new Map();
    rows.forEach((row) => {
      const item = dishes.get(row.dish_id) || { dish_id: row.dish_id, dish_name: row.dish_name };
      if (String(row.method).startsWith("Before")) item.before = row;
      else item.after = row;
      dishes.set(row.dish_id, item);
    });
    const body = Array.from(dishes.values())
      .map((item) => {
        const before = item.before?.rank ?? "-";
        const after = item.after?.rank ?? "-";
        const change =
          before === "-" ? "New in Top-K" :
          after === "-" ? "Excluded" :
          before > after ? `↑ ${before - after} position(s)` :
          before < after ? `↓ ${after - before} position(s)` : "No change";
        return `<tr><td>${escapeHtml(item.dish_name)} (${escapeHtml(item.dish_id)})</td><td>${before}</td><td>${after}</td><td>${item.after ? Number(item.after.ingredient_score).toFixed(2) : "-"}</td><td>${escapeHtml(change)}</td><td>${escapeHtml(item.after?.matched || item.after?.excluded || "Disliked ingredient")}</td></tr>`;
      })
      .join("");
    table = experimentTable(
      ["Dish", "Before rank", "After rank", "Ingredient score", "Change", "Matched / exclusion"],
      body
    );
  } else if (type === "coorder") {
    const before = rows[0] || {};
    const after = rows[1] || {};
    const beforeMetrics = parseCoorderMetrics(before.matched);
    const afterMetrics = parseCoorderMetrics(after.matched);
    const metrics = [
      ["Pair count", beforeMetrics.pairCount, afterMetrics.pairCount],
      ["Co-order score", Number(before.co_order_score || 0), Number(after.co_order_score || 0)],
      ["Support", beforeMetrics.support, afterMetrics.support],
      ["Confidence", beforeMetrics.confidence, afterMetrics.confidence],
      ["Lift", beforeMetrics.lift, afterMetrics.lift],
      ["Candidate rank", before.rank ?? "-", after.rank ?? "-"],
    ];
    const body = metrics
      .map(([label, first, second]) => {
        const numeric = typeof first === "number" && typeof second === "number";
        const change = numeric ? second - first : "-";
        const format = (value) =>
          typeof value === "number" ? value.toFixed(label === "Pair count" || label === "Candidate rank" ? 0 : 2) : value;
        return `<tr><td>${escapeHtml(label)}</td><td>${format(first)}</td><td>${format(second)}</td><td>${format(change)}</td></tr>`;
      })
      .join("");
    table = experimentTable(["Metric", "Before", "After", "Change"], body);
  } else {
    const methods = [...new Set(rows.map((row) => row.method))];
    const methodTables = methods
      .map((method) => {
        const body = rows
          .filter((row) => row.method === method)
          .map((row) => `<tr class="${row.hidden_match ? "highlight-row" : ""}"><td>${row.rank ?? "-"}</td><td>${escapeHtml(row.dish_id)}</td><td>${escapeHtml(row.dish_name)}</td><td>${Number(row.final_score).toFixed(2)}</td><td>${row.hidden_match ? "Hidden target" : "-"}</td></tr>`)
          .join("");
        return `<h4>${escapeHtml(method)}</h4>${experimentTable(["Rank", "Dish ID", "Dish", "Score", "Target"], body)}`;
      })
      .join("");
    const summaryRows = methods
      .map((method) => {
        const methodRows = rows.filter((row) => row.method === method);
        const target = methodRows.find((row) => row.hidden_match);
        const details = String(methodRows[0]?.matched || "");
        const matchRate = details.match(/preference match rate:\s*([0-9.]+)/i)?.[1] || "-";
        const violations = details.match(/violations:\s*(\d+)/i)?.[1] || "-";
        return `<tr><td>${escapeHtml(method)}</td><td>${target ? "Yes" : "No"}</td><td>${target?.rank ?? "-"}</td><td>${escapeHtml(matchRate)}</td><td>${escapeHtml(violations)}</td></tr>`;
      })
      .join("");
    table = `${methodTables}<h4>Method summary</h4>${experimentTable(
      ["Method", "Hit@K", "Hidden rank", "Preference match rate", "Violations"],
      summaryRows
    )}`;
  }
  return `${table}<div class="reason-box"><strong>Conclusion</strong><p>${escapeHtml(data.conclusion || "-")}</p><p>Production data and weights were not changed.</p></div>`;
}

function parseCoorderMetrics(text) {
  const value = String(text || "");
  const number = (pattern) => Number(value.match(pattern)?.[1] || 0);
  return {
    pairCount: number(/Pair count:\s*(\d+)/i),
    support: number(/support\s*([0-9.]+)/i),
    confidence: number(/confidence\s*([0-9.]+)/i),
    lift: number(/lift\s*([0-9.]+)/i),
  };
}

function experimentTable(headers, body) {
  return `<div class="table-wrap"><table><thead><tr>${headers
    .map((header) => `<th>${escapeHtml(header)}</th>`)
    .join("")}</tr></thead><tbody>${body || `<tr><td colspan="${headers.length}">No result rows.</td></tr>`}</tbody></table></div>`;
}

function setupSimulationTester() {
  const runButton = document.getElementById("run-simulation");
  const resetButton = document.getElementById("reset-simulation");
  const target = document.getElementById("simulation-results");
  const forcedA = document.getElementById("simulation-forced-a");
  const forcedB = document.getElementById("simulation-forced-b");
  if (!runButton || !target) {
    return;
  }

  const forcedOptions = [`<option value="">No forced pair</option>`]
    .concat(
      (window.MENU_DISHES || []).map(
        (dish) => `<option value="${escapeHtml(dish.dish_id)}">${escapeHtml(dish.name)} (${escapeHtml(dish.dish_id)})</option>`
      )
    )
    .join("");
  if (forcedA) forcedA.innerHTML = forcedOptions;
  if (forcedB) forcedB.innerHTML = forcedOptions;

  runButton.addEventListener("click", async () => {
    target.innerHTML = `<div class="info-card slim"><strong>Running simulation...</strong><span>Generated orders are in memory only.</span></div>`;
    const preferences = collectPreferences("admin");
    const payload = {
      ...preferences,
      order_count: Number(document.getElementById("simulation-order-count")?.value || 20),
      min_dishes: Number(document.getElementById("simulation-min-dishes")?.value || 2),
      max_dishes: Number(document.getElementById("simulation-max-dishes")?.value || 4),
      seed: Number(document.getElementById("simulation-seed")?.value || 42),
      popularity_skew: document.getElementById("simulation-skew")?.value || "uniform",
      forced_dish_a: forcedA?.value || null,
      forced_dish_b: forcedB?.value || null,
      pair_probability: Number(document.getElementById("simulation-pair-probability")?.value || 35),
    };
    try {
      const response = await fetch("/api/admin/simulation", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });
      const result = await response.json();
      if (!result.ok) {
        target.innerHTML = `<div class="info-card slim"><strong>Simulation failed</strong><span>${escapeHtml(result.message)}</span></div>`;
        return;
      }
      renderSimulationResults(result.data, target);
    } catch (error) {
      console.error("Simulation failed", error);
      target.innerHTML = `<div class="info-card slim"><strong>Simulation failed</strong><span>Please try again.</span></div>`;
    }
  });

  resetButton?.addEventListener("click", () => {
    target.innerHTML = `<div class="info-card slim"><strong>Simulation reset</strong><span>The tester is using the real historical order dataset only.</span></div>`;
  });
}

function renderSimulationResults(data, target) {
  const preview = (data.preview || [])
    .map((order) => `<li><strong>${escapeHtml(order.order_id)}</strong>: ${escapeHtml((order.dish_names || order.dish_ids || []).join(", "))}</li>`)
    .join("");
  const pairs = (data.changed_pairs || [])
    .map(
      (pair) => `<tr><td>${escapeHtml(pair.label)}</td><td>${pair.before_count}</td><td>${pair.after_count}</td><td>${Number(pair.support_before).toFixed(2)}</td><td>${Number(pair.support_after).toFixed(2)}</td></tr>`
    )
    .join("");
  const ranks = (data.rank_changes || [])
    .map(
      (item) => `<tr><td>${escapeHtml(item.dish_name)} (${escapeHtml(item.dish_id)})</td><td>${item.before_rank ?? "-"}</td><td>${item.after_rank ?? "-"}</td><td>${Number(item.before_score).toFixed(2)}</td><td>${Number(item.after_score).toFixed(2)}</td><td>${escapeHtml(item.explanation)}</td></tr>`
    )
    .join("");

  target.innerHTML = `
    <div class="info-card slim"><strong>${escapeHtml(data.note || "Simulation completed.")}</strong><span>${data.generated_order_count || 0} generated basket(s).</span></div>
    <details open><summary>Generated order preview</summary><ul class="simulation-preview">${preview || "<li>No generated orders.</li>"}</ul></details>
    <details open><summary>Top changed co-order pairs</summary><div class="table-wrap"><table><thead><tr><th>Pair</th><th>Before</th><th>After</th><th>Support before</th><th>Support after</th></tr></thead><tbody>${pairs || "<tr><td colspan='5'>No pair changes found.</td></tr>"}</tbody></table></div></details>
    <details open><summary>Recommendation rank changes</summary><div class="table-wrap"><table><thead><tr><th>Dish</th><th>Before rank</th><th>After rank</th><th>Before score</th><th>After score</th><th>Explanation</th></tr></thead><tbody>${ranks || "<tr><td colspan='6'>No rank changes found.</td></tr>"}</tbody></table></div></details>
  `;
}

function setupAdminInsights() {
  const summary = document.getElementById("admin-insight-summary");
  const grid = document.getElementById("admin-insight-grid");
  const refresh = document.getElementById("refresh-admin-insights");
  if (!summary || !grid) {
    return;
  }

  const renderList = (title, values) => `
    <div class="insight-card">
      <strong>${escapeHtml(title)}</strong>
      ${
        values && values.length
          ? `<ul>${values.map((value) => `<li>${escapeHtml(value)}</li>`).join("")}</ul>`
          : `<p class="muted">No data yet.</p>`
      }
    </div>
  `;

  const loadInsights = async () => {
    summary.textContent = "Loading insights...";
    try {
      const response = await fetch("/api/admin/insights");
      const result = await response.json();
      summary.textContent = result.summary || "";
      grid.innerHTML =
        renderList("Popular dishes", result.popular || []) +
        renderList("Co-order patterns", result.co_order_pairs || []) +
        renderList("Low exposure dishes", result.low_exposure || []);
    } catch {
      summary.textContent = "Unable to load insight summary.";
    }
  };

  refresh?.addEventListener("click", loadInsights);
  loadInsights();
}

document.addEventListener("DOMContentLoaded", () => {
  updateCartCount();
  setupDishLocator();
  setupCarouselControls();
  setupOrderFilters();
  setupPreferencePanels();
  setupCartButtons();
  setupDetailButtons();
  renderCartPage();
  setupCheckout();
  setupSmartMenuAssistant();
  setupAdminTools();
  setupCustomerOrderStatus();
  refreshCustomerRecommendations();
});
