# Architecture

## Scope

The system is a single-restaurant FYP prototype. One Rust process serves
server-rendered pages, JSON endpoints, static assets, and local persistence. The
architecture favours explainability and small operational cost over distributed
scalability.

## Context

```mermaid
flowchart LR
    Customer[Customer phone browser]
    Staff[Staff/admin browser]
    App[Preference-Driven Restaurant System]
    Data[(Local CSV and JSONL)]
    Images[(Local dish images)]

    Customer -->|register, browse, order, track| App
    Staff -->|manage dishes/orders, inspect recommender| App
    App <--> Data
    App --> Images
```

No runtime request is sent to an LLM, external recommender, analytics service, or
image host.

## Containers and Modules

```mermaid
flowchart TB
    subgraph Browser
        HTML[Server-rendered HTML]
        JS[static/app.js]
        CSS[static/app.css]
    end
    subgraph Rust
        Routes[web/routes.rs]
        Handlers[web/handlers]
        Session[web/session.rs]
        Validation[web/validation.rs]
        State[web/state.rs]
        Search[search.rs and agent]
        Rec[recommender modules]
        Persistence[persistence modules]
        Loader[data_loader.rs]
        Templates[web/templates.rs]
    end
    subgraph Files
        Dishes[data/dishes.csv]
        Orders[data/orders.csv]
        Details[data/order_details.csv - runtime]
        Timeline[data/recommendation_learning_events.jsonl - runtime]
        Assets[assets/dishes]
    end

    HTML --> JS
    HTML --> CSS
    JS <--> Routes
    Routes --> Handlers
    Handlers --> Session
    Handlers --> Validation
    Handlers --> State
    State --> Search
    State --> Rec
    State --> Persistence
    Handlers --> Templates
    Loader <--> Dishes
    Loader <--> Orders
    Persistence <--> Details
    Persistence <--> Timeline
    Routes --> Assets
```

## Responsibility Boundaries

| Area | Owner | Does not own |
|---|---|---|
| HTTP paths and methods | `web/routes.rs` | Business formulas |
| Request/response wiring | `web/handlers/*` | CSV parsing |
| Cookies and session IDs | `web/session.rs` | Authentication UI |
| Repeated input rules | `web/validation.rs` | Dish/order persistence |
| Application orchestration | `web/state.rs` | Browser rendering |
| Recommendation formulas | `recommender/*` | HTTP or image loading |
| File integrity | `persistence/*`, `data_loader.rs` | UI decisions |
| Presentation | `web/templates.rs`, `static/*` | Scoring |

`web/state.rs` and `web/templates.rs` remain large. They are known concentration
points, but their boundaries are now narrower after session, validation, and
atomic persistence extraction. Further splitting should be behavior-led rather
than a line-count-only rewrite.

## Customer Request Flow

```mermaid
sequenceDiagram
    participant B as Browser
    participant H as Handler
    participant S as Session service
    participant A as WebState
    participant T as Template

    B->>H: GET /
    H->>S: Read customer cookie
    alt not registered
        H-->>B: Redirect /start
    else registered
        H->>A: Build menu and recommendations
        A-->>H: View model
        H->>T: Render page
        T-->>B: HTML
    end
```

Customer and admin cookies use different names and separate in-memory maps.
Customer order endpoints verify that an order belongs to the session phone,
rather than trusting a phone number supplied in a query string.

## Recommendation Flow

```mermaid
sequenceDiagram
    participant H as Recommendation handler
    participant S as WebState
    participant C as Scoring context
    participant R as Candidate scorer
    participant D as Diversity reranker

    H->>S: Validated preferences, context, Top-K, mode
    S->>C: Build once per request
    Note over C: Co-order matrix, popularity counts,<br/>candidate context counts, maxima,<br/>evidence profile and adaptive weights
    C->>R: Score every eligible dish
    Note over R: Disliked ingredients are excluded first
    R->>D: Base-ranked candidates
    D-->>S: Deterministic reranked list
    S-->>H: Scores, evidence, associations, explanations
```

All shared historical indexes are computed once for a request. Candidate scoring
does not rebuild the order matrix or rescan all orders for each dish.

## Checkout and Completion Flow

```mermaid
sequenceDiagram
    participant C as Customer
    participant A as WebState
    participant Staff as Admin
    participant CSV as data/orders.csv
    participant TL as Learning timeline

    C->>A: Checkout cart
    A->>A: Validate session and dish availability
    A->>A: Create live in-memory order
    Staff->>A: PATCH status
    alt status becomes Completed for first time
        A->>CSV: Append Oxxx,Uxx,"Dxx,...",timestamp
        A->>A: Add historical basket immediately
        A->>TL: Append explanatory learning event
        A->>A: Recalculate insights on next request
    else repeated Completed request
        A->>A: Idempotency guard prevents duplicate append
    end
```

Completed order baskets are durable in `orders.csv`. Live order detail is
process-local plus a runtime detail file and is not a database transaction.

## Learning Events

The timeline is a derived explanation log. A real completed order can append a
JSONL event showing popularity and pair-count deltas. Clearing the timeline
removes only these explanatory records. It does not remove historical orders or
recommendation evidence. Rebuild deterministically reconstructs events from
historical order baskets.

## Simulation Isolation

```mermaid
flowchart LR
    History[Clone of historical orders] --> Sim[Add synthetic baskets in memory]
    Sim --> Compare[Recalculate temporary result]
    Real[(data/orders.csv)] -. never written .-> Sim
    Compare --> Response[Before/after response]
```

Co-order simulation and counterfactual requests operate on cloned data. Tests
assert that historical order count and timeline state remain unchanged.

## Persistence Boundaries

- `data/dishes.csv`: startup fixture and admin export source; dish management is
  in memory for the current process.
- `data/orders.csv`: append-only completed historical baskets.
- `data/order_details.csv`: ignored runtime details for the current installation.
- `data/recommendation_learning_events.jsonl`: ignored derived timeline.
- Replacement files use `persistence/atomic_file.rs`: write temporary content,
  flush/sync, preserve a backup, and replace.
- Append-only files flush and sync before reporting success.

## Security Boundaries

- Admin credentials default to `admin` / `admin` for an immediate FYP demo.
  `ADMIN_USERNAME` and `ADMIN_PASSWORD` override those defaults.
- Admin sessions use OS-random 192-bit identifiers.
- Customer sessions also use OS randomness and a separate cookie.
- Cookies are `HttpOnly`, `SameSite=Lax`, path `/`, and can be marked `Secure`
  with `APP_COOKIE_SECURE`.
- Admin mutation routes require an admin session.
- Customer order lookup is scoped to the customer session.
- Passwords, cookies, and full phone numbers are not logged.
- This prototype does not yet implement CSRF tokens, password hashing/account
  storage, rate limiting, or TLS termination.

## Determinism and Data Safety

Recommendation ordering has explicit tie-breakers. Hash-map iteration order is
not used as a final ranking rule. Normalised values are clamped and denominator
checks prevent NaN or infinity. Duplicate IDs and malformed timestamps fail
loading with context rather than being silently discarded.
