# Phase 79: Update UI https://app.esluce.com/billing — Research

**Researched:** 2026-06-15
**Domain:** Frontend UI restyle + backend route mount
**Confidence:** HIGH

## Summary

Phase 79 restyles the Billing page (`/billing`) from flat `bg-gray-800 rounded-lg` to cosmic `glass-panel` theme, adds usage progress bars (servers, RAM, CPU cores, disk) with green/yellow/red thresholds, and mounts the existing `UsageHandlers` route in the backend.

**Key findings:**
1. **Backend is ready** — `UsageHandlers` is imported in `api_routes.rs` line 17, just needs one `.nest()` call. The `get_quotas` handler and `QuotaService::get_user_quota()` are fully implemented. `CurrentQuotaUsage` has `servers`, `ram_mb`, `cpu_cores`, `disk_gb` (no nodes count, matching spec).
2. **ApiResponse double-wrapping** — Backend wraps in `ApiResponse { success, data, error }`, and the frontend `api.js` line 58 does `data?.data ?? data`, so the frontend receives the unwrapped `{ plan, limits, current_usage }` object.
3. **No existing tests** — Zero test files for billing or usage. Validation architecture will be manual/smoke-test based.
4. **CSS patterns are consistent** — `glass-panel`, `var(--color-cosmic-*)`, progress bar pattern from `ProfilingPanel.jsx` lines 11-14 all exist.
5. **Existing bug opportunity** — Current BillingPage has three subscription state sections (lines 173, 219, 251) where sections 2 and 3 render identical content. Can be collapsed in the refactor.

**Primary recommendation:** Three plan files: (1) Backend route mount (single line), (2) Frontend API client + BillingPage restyle, (3) Usage bars component extraction (or inline). Heavy JSX work concentrated in file 2.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Lift all containers: `bg-gray-800 rounded-lg` → `glass-panel p-6` with `border border-[var(--color-cosmic-border)]` cosmic border accents
- Use `hover:border-[var(--color-cosmic-cyan)]/40` on plan cards (matching PluginManager pattern)
- Keep spinner loading state as-is (consistent with other pages)
- DO NOT change font sizes, heading hierarchy, or button styling patterns — they already match
- Keep vertical stacked layout, DO NOT introduce tabs or sidebars
- Section order: Subscription Status → Plan Cards (Hobby/Pro) → Payment History (invoices) → Refund sections
- Add a subtle visual separator between sections (`border-b border-[var(--color-cosmic-border)]`)
- Progress bars in subscription card for: servers, nodes, RAM, CPU, disk
  - Each bar shows: `current / max` numeric label + fill bar
  - Max = -1 means "Unlimited" — full bar with "Unlimited" label, no fill
- Fetch from existing `QuotaService::get_user_quota()` via new route
- Backend: add `.nest("/api/v1/usage", UsageHandlers::router(state.clone()))` to `api_routes.rs`
- Frontend API: new `usageApi.getQuotas()` → `api.get('/usage/quotas')` — returns `{ plan, limits, current_usage }`
- Call alongside billing data in `loadData()` via `Promise.all()`
- `current_usage` includes: `servers`, `ram_mb`, `cpu_cores`, `disk_gb` — NO node count. Nodes = text-only display.
- Bar colors: green (< 60%), yellow (60-85%), red (> 85%)
- Refund section: keep emoji indicators (🟢🟡🔴), keep "Berhenti Berlangganan" button (Indonesian locale), restyle only
- Plan cards: 2-column grid, `glass-panel`, hover glow, keep all content
- Payment History: keep table format, restyle container to glass-panel
- No tabs/sidebar, no new sections, no backend refactoring beyond route mount, no locale changes, no mobile-specific layout changes
- Out of scope (deferred): usage history charts, bandwidth tracking, payment method management, invoice download, annual pricing toggle

### the agent's Discretion
[NONE — all design decisions locked]

### Deferred Ideas (OUT OF SCOPE)
- Usage history charts / monthly breakdown
- Bandwidth usage tracking display
- Payment method management UI
- Invoice download
- Annual pricing toggle
</user_constraints>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Usage quota data | **API / Backend** | — | QuotaService queries PostgreSQL for plan limits + live server resources |
| Progress bar display | **Browser / Client** | — | Pure JSX calculation (percentage = current/max), no server round-trip |
| Subscription status | **API / Backend** | — | BillingHandlers return subscription + plan data |
| Plan card rendering | **Browser / Client** | — | Static JSX with hard-coded fallback; API data enriches but doesn't block render |
| Payment history | **API / Backend** | — | Invoices fetched from billing service via BillingHandlers |
| Refund eligibility | **API / Backend** | — | BillingHandlers compute from subscription metadata |
| Route mounting | **API / Backend** | — | Single-line addition to existing axum Router |

## Phase Requirements

> No requirement IDs provided in ROADMAP.md for Phase 79. The UI-SPEC.md interaction contracts (D-01 through D-10) serve as the behavioral specification. The planner should reference these as D-01, D-02, etc.

| Reference | Description | Research Support |
|-----------|-------------|-----------------|
| D-01 | Section layout: Subscription → Plan Cards → Payment History → Refund | Layout pattern confirmed: current BillingPage follows same order. CONTEXT.md locks section order. |
| D-02 | Subscription Status card restyle to glass-panel + accent border | `glass-panel` class defined in `index.css:75`. Status badge pattern from Nodes.jsx: `bg-[var(--color-cosmic-{color})]/10 text-[var(--color-cosmic-{color})]`. |
| D-03 | Usage bars: 4 resources, green/yellow/red thresholds, Unlimited case | `ProfilingPanel.jsx:4-18` has existing progress bar pattern. `QuotaService::get_user_quota()` fully implemented. |
| D-04 | Plan cards: glass-panel, hover glow, 2-column grid | `TemplateCard.jsx:8-9` has exact `hover:border-[var(--color-cosmic-cyan)]/50 transition-all` pattern. |
| D-05 | Berhenti Berlangganan button: window.confirm, restyle only | Current code at lines 62-75 uses `window.confirm()`. CONTEXT.md locks this behavior. |
| D-06 | Request Refund button: conditional on eligibility | Current code at lines 303-310. No confirmation dialog per spec. |
| D-07 | Payment History table glass-panel restyle | Current table at lines 405-433. Status badge pattern from existing code + CSS vars. |
| D-08 | Refund Eligibility glass-panel + emoji indicators | Current code at lines 283-312. Locked per CONTEXT.md. |
| D-09 | Loading state: EscluseSpinner centered | Current code at lines 91-97. Existing component at `SkeletonLoader.jsx:158-164`. |
| D-10 | Data loading: parallel billing + usage, waterfall refund | Current `loadData()` pattern at lines 19-49. Usage API joins `Promise.all()`. |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| React | ^18.x | UI framework | Project-wide standard. BillingPage is a React component. |
| Tailwind CSS | v4 | Utility CSS | Project-wide. Cosmic theme defined via `@theme` in `index.css:4-19`. |
| Axum | 0.7 | Rust HTTP framework | Backend standard. `api_routes.rs` mounts all handlers. |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| lucide-react | ^1.18.0 | Icons | Per UI-SPEC.md. Not currently used on billing page (emoji only), but available if needed. |
| zustand | ^4.x | State management | `uiStore.js` provides `addToast()` pattern used by billing page. |
| sqlx | 0.7 | PostgreSQL driver | `QuotaService` uses sqlx for all DB queries. |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand-built progress bar (`<div>`) | shadcn/ui Progress | Project has no shadcn dependency. Hand-built matches cosmic theme and avoids 200KB+ dependency. |
| Inline JSX for usage bars | Extracted `UsageBar` component | Inline keeps BillingPage single-file pattern. Extract if bars need reuse elsewhere (deferred decision — no indication of reuse). |

**Installation:**
```bash
# No new packages needed — all libraries already in the project.
```

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        Browser (React SPA)                      │
│                                                                 │
│  BillingPage.jsx                                                │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  loadData()  ──┐                                        │    │
│  │                ▼                                        │    │
│  │  Promise.all([                                          │    │
│  │    billingApi.getPlans(),          ──┐                   │    │
│  │    billingApi.getCurrentSubscription(),                 │    │
│  │    billingApi.getInvoices(),                            │    │
│  │    usageApi.getQuotas()           <── NEW                │    │
│  │  ])                                                     │    │
│  │                │                                        │    │
│  │     ┌──────────┼─────────────┬──────────────┐           │    │
│  │     ▼          ▼             ▼              ▼           │    │
│  │  Plans    Subscription   Invoices     Quotas            │    │
│  │  (cards)  (status +      (table)     (usage bars)      │    │
│  │             usage bars)              NEW                │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
│  Data flow (after Promise.all resolves):                        │
│  quotas.current_usage.servers → UsageBar % calculation          │
│  quotas.limits.max_servers    → UsageBar denominator            │
│  ──────────────────────────────────────────────                 │
│  Bar color = f(percentage):                                     │
│    < 60%  → green  (var(--color-cosmic-green))                  │
│    60-85% → yellow (var(--color-cosmic-orange))                 │
│    > 85%  → red    (var(--color-cosmic-red))                    │
└─────────────────────────────────────────────────────────────────┘
                            │ HTTP
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                    API Server (Axum/Rust)                        │
│                                                                 │
│  api_routes.rs                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  .nest("/api/v1/usage", UsageHandlers::router(...))     │    │
│  │                    │                                     │    │
│  │                    ▼                                     │    │
│  │  usage_handlers.rs                                       │    │
│  │  ┌──────────────────────────────────────────────────┐    │    │
│  │  │  GET /quotas → get_quotas()                      │    │    │
│  │  │    → QuotaService::get_user_quota(user_id)        │    │    │
│  │  │    → ApiResponse::success(json!({                 │    │    │
│  │  │        plan, limits, current_usage                │    │    │
│  │  │      }))                                          │    │    │
│  │  └──────────────────────────────────────────────────┘    │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
│  QuotaService (usage/service.rs)                                │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  get_user_quota(user_id)                                │    │
│  │    1. Find subscription → get plan                       │    │
│  │    2. Parse PlanLimits from plan.limits JSON              │    │
│  │    3. Sum server resources (ram, cpu, disk)               │    │
│  │    4. Return UserQuota { plan_name, limits,               │    │
│  │       current_usage: { servers, ram_mb,                   │    │
│  │         cpu_cores, disk_gb } }                            │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
                            │ SQL
                            ▼
              ┌─────────────────────────┐
              │     PostgreSQL           │
              │  • servers table         │
              │  • plans table           │
              │  • subscriptions table   │
              └─────────────────────────┘
```

### Recommended Project Structure

> No structural changes needed. Only three files modified:
```
app/src/
├── lib/
│   └── api.js                              ← Add usageApi export (+3 lines)
└── pages/
    └── billing/
        └── BillingPage.jsx                 ← Heavy restyle (+usage bars)

api/src/presentation/routes/
    └── api_routes.rs                       ← Add .nest() call (+1 line)
```

### Pattern 1: Loading State with EscluseSpinner

**What:** Full-page centered spinner during initial data load.

**When to use:** Page-level async data fetching with `useEffect`.

**Source:** `BillingPage.jsx:91-97` (current), `SkeletonLoader.jsx:158-164` (component definition)

```jsx
// Current pattern (kept as-is per CONTEXT.md):
if (isLoading) {
  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-900">
      <EscluseSpinner size={100} color="#06b6d4" />
    </div>
  )
}
```

### Pattern 2: API Client Extension

**What:** Add a new `usageApi` export following existing API client pattern.

**When to use:** Any new backend endpoint consumed by the frontend.

**Source:** `api.js:124-138` (billingApi pattern), `api.js:56-58` (response unwrapping)

```js
// Add to app/src/lib/api.js (following billingApi pattern at line 124):
export const usageApi = {
  getQuotas: () => api.get('/usage/quotas'),
}
```

### Pattern 3: Glass-Panel Container

**What:** Replace `bg-gray-800 rounded-lg` with `glass-panel p-6` and optional cosmic border.

**When to use:** Every section wrapper in the billing page.

**Source:** `index.css:75-80` (glass-panel definition), `TemplateCard.jsx:8-9` (implementation example)

```jsx
<div className="glass-panel p-6 mb-8 border border-[var(--color-cosmic-border)]">
  {/* section content */}
</div>

<!-- Accent border variant (subscription active): -->
<div className="glass-panel p-6 mb-8 border border-[var(--color-cosmic-cyan)]">
```

### Pattern 4: Progress Bar

**What:** Custom div-based progress bar with dynamic color thresholding.

**When to use:** Usage display, resource monitoring.

**Source:** `ProfilingPanel.jsx:4-18` (existing pattern), UI-SPEC.md D-03 (specification)

```jsx
function getBarColor(percentage) {
  if (percentage >= 85) return 'var(--color-cosmic-red)'    // red > 85%
  if (percentage >= 60) return 'var(--color-cosmic-orange)' // yellow 60-85%
  return 'var(--color-cosmic-green)'                         // green < 60%
}

// Per-bar structure:
<div className="mb-4">
  <div className="flex justify-between text-sm mb-1">
    <span className="text-[var(--color-text-muted)]">Servers</span>
    <span className="text-white font-medium">5 / 10</span>
  </div>
  <div className="h-2 rounded-full bg-gray-700 overflow-hidden">
    <div
      className="h-full rounded-full transition-all duration-300"
      style={{
        width: `${percentage}%`,
        backgroundColor: getBarColor(percentage),
      }}
    />
  </div>
</div>

// Unlimited case (max === -1):
// Show full bar at 100% with bg-gray-600 (muted, not threshold color)
// Label shows "Unlimited" instead of "current / max"
<div className="h-2 rounded-full bg-gray-700 overflow-hidden">
  <div className="h-full rounded-full bg-gray-600" style={{ width: '100%' }} />
</div>

// Overage case (current > max):
// Cap at 100%, use red color, show actual numbers "6 / 5"
```

### Pattern 5: Status Badge with CSS Variables

**What:** Colored badge using cosmic theme CSS variables.

**When to use:** Invoice status, subscription status, refund status.

**Source:** CONTEXT.md line 69, Nodes.jsx pattern

```jsx
<span className="bg-[var(--color-cosmic-green)]/20 text-[var(--color-cosmic-green)] px-2 py-1 rounded text-sm">
  Active
</span>

<!-- Orange variant: -->
<span className="bg-[var(--color-cosmic-orange)]/20 text-[var(--color-cosmic-orange)] px-2 py-1 rounded text-sm">
  Pending
</span>

<!-- Red variant: -->
<span className="bg-[var(--color-cosmic-red)]/20 text-[var(--color-cosmic-red)] px-2 py-1 rounded text-sm">
  Failed
</span>
```

### Pattern 6: Plan Card with Hover Glow

**What:** Interactive card with cosmic hover glow effect.

**When to use:** Plan cards, any interactive card in cosmic theme.

**Source:** `TemplateCard.jsx:8-9`, CONTEXT.md line 67

```jsx
<div className="glass-panel p-6 border border-[var(--color-cosmic-border)]
                hover:border-[var(--color-cosmic-cyan)]/40 transition-all
                flex flex-col h-full">
  {/* card content */}
</div>
```

### Anti-Patterns to Avoid
- **Importing a progress bar library:** The `<div>`-based pattern already exists in the codebase (`ProfilingPanel.jsx:11-14`). Adding a library like `recharts` or `@radix-ui/progress` for 4 bars is unnecessary weight.
- **Using inline `bg-green-500` instead of CSS variables:** The cosmic theme defines `var(--color-cosmic-green)` etc. All new code should use CSS vars for consistency and theming support.
- **Nesting `loading` state for usage separately:** CONTEXT.md specifies usage is fetched alongside billing in `Promise.all()`. No separate loading spinner needed.
- **Creating a new component file for usage bars:** UI-SPEC.md doesn't indicate reuse. Inline in BillingPage.jsx is fine. Extract only if the page exceeds ~700 lines or bars are needed elsewhere.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Progress bar UI | Custom `<div>` with width/height/border-radius | Hand-built (current pattern) | Already exists in ProfilingPanel.jsx. 4 lines of JSX. No library needed. |
| CSS animations | Custom keyframes | Tailwind `transition-all duration-300` | Already available from Tailwind v4. Bar fill animation is a single class. |
| Toast notifications | Custom toast system | `useUIStore().addToast()` | Already exists. BillingPage already imports and uses it (lines 55-74). |
| HTTP client | Custom fetch wrapper | `api` class from `api.js` | Already exists with auth token refresh, error handling, JSON parsing. |

**Key insight:** This phase requires zero new dependencies. Everything needed (glass panels, progress bars, toasts, API client, CSS variables) already exists in the codebase.

## Common Pitfalls

### Pitfall 1: ApiResponse Double-Wrapping Confusion
**What goes wrong:** The backyard `get_quotas` handler returns `ApiResponse::success(json!({...}))`, which wraps the data in `{ success: true, data: { plan, limits, current_usage } }`. The frontend `api.js` line 58 does `data?.data ?? data`, so the consumer receives `{ plan, limits, current_usage }`. If someone adds `.data` access in BillingPage, it will be `undefined`.
**Why it happens:** The `api.js` already unwraps `ApiResponse.data`. Calling `api.get('/usage/quotas')` returns the already-unwrapped payload.
**How to avoid:** Do NOT add `.data` access to usage API response. The response shape is `{ plan: String, limits: PlanLimits, current_usage: { servers, ram_mb, cpu_cores, disk_gb } }`.
**Warning signs:** If `usage.plan` is `undefined →` check for double unwrapping.

### Pitfall 2: Missing `.with_state()` on UsageHandlers Router
**What goes wrong:** The usage router `UsageHandlers::router(state)` already calls `.with_state(state)` inside the impl block (`usage_handlers.rs:24-30`). But if the `.nest()` call in `api_routes.rs` passes `state.clone()`, the inner state might conflict if the outer Router also calls `.with_state()`.
**Why it happens:** Axum nested routers inherit state from parent, but if they also call `.with_state()`, the explicit state takes precedence.
**How to avoid:** The pattern in `api_routes.rs` is `.nest("/api/v1/billing", BillingHandlers::router(state.clone()))` — UsageHandlers follows the identical pattern. The inner `.with_state(state)` is fine because `UsageHandlers::router()` accepts `ApiState` and calls `.with_state()` on its own router.
**Verification:** BillingHandlers follows this pattern already. UsageHandlers impl at line 23-31 matches exactly.

### Pitfall 3: Percentage Calculation Division by Zero
**What goes wrong:** `Math.round((current / max) * 100)` when `max === 0` produces `Infinity`.
**Why it happens:** Some plans may have limit field `0` (parsed as `unwrap_or(0)` in `PlanLimits::from_json`).
**How to avoid:** Always guard: `max > 0 ? Math.round((current / max) * 100) : 0`. The Unlimited case (max === -1) should be handled separately before the percentage calculation.
**Warning signs:** Progress bar showing full width for 0 limit.

### Pitfall 4: Hard-Coded Color Strings Instead of CSS Variables
**What goes wrong:** Using `bg-green-500` instead of `bg-[var(--color-cosmic-green)]/20` breaks light theme support and creates visual inconsistency.
**Why it happens:** Old billing page uses hard-coded tailwind colors (`bg-red-600`, `text-green-400`, etc.). The new cosmic theme uses CSS variables for all semantic colors.
**How to avoid:** Replace ALL hard-coded color classes with CSS variable equivalents. Reference the mapping in UI-SPEC.md color section lines 71-87.
**Verify against:** Every color class in the current BillingPage.jsx — lines 179, 208, 210, 223, 284, 306, 329-331, 350, 377, 389, 395, 410, 413, 422-423.

### Pitfall 5: Unlimited Label Logic
**What goes wrong:** For max === -1, the bar should show "Unlimited" label and full muted bar. For max === 0 (unset/null), it might be confused with Unlimited.
**Why it happens:** Both -1 and 0 are falsy in some conditionals.
**How to avoid:** Check `max === -1` explicitly for the Unlimited case. Zero should still render a bar at 0%.
**Contract from CONTEXT.md line 34:** "Max = -1 means 'Unlimited' — show full bar with 'Unlimited' label, no fill."

## Code Examples

### Backend Route Mount (api_routes.rs)
```rust
// Add after .nest("/api/v1/billing", BillingHandlers::router(state.clone()))
// (line 29 in api_routes.rs)
.nest("/api/v1/usage", UsageHandlers::router(state.clone()))
// Note: UsageHandlers is already imported at line 17
```

### Frontend API Client Addition (api.js)
```javascript
// Add after billingApi block (after line 138):
export const usageApi = {
  getQuotas: () => api.get('/usage/quotas'),
}
// Response shape (already unwrapped by api.js):
// { plan: String, limits: PlanLimits, current_usage: { servers: i32, ram_mb: i32, cpu_cores: i32, disk_gb: i32 } }
```

### Usage Bar Color Threshold Function
```javascript
// Pure function, no dependencies:
function getBarColor(percentage) {
  if (percentage >= 85) return 'var(--color-cosmic-red)'
  if (percentage >= 60) return 'var(--color-cosmic-orange)'
  return 'var(--color-cosmic-green)'
}
```

### Progress Bar JSX (UsageBar)
```jsx
// Inline pattern for each metric:
const percentage = Math.min(Math.round((current / max) * 100), 100)

<div className="mb-4">
  <div className="flex justify-between text-sm mb-1">
    <span className="text-[var(--color-text-muted)]">Servers</span>
    <span className="text-white font-medium">{current} / {max === -1 ? 'Unlimited' : max}</span>
  </div>
  <div className="h-2 rounded-full bg-gray-700 overflow-hidden">
    <div
      className="h-full rounded-full transition-all duration-300"
      style={{
        width: max === -1 ? '100%' : `${percentage}%`,
        backgroundColor: max === -1 ? 'rgb(75, 85, 99)' : getBarColor(percentage), // gray-600 for unlimited
      }}
    />
  </div>
</div>
```

### Data Loading with Usage Quotas (BillingPage.jsx)
```javascript
// Updated loadData() — add usageApi to Promise.all:
import { billingApi, usageApi } from '../../lib/api'

const loadData = async () => {
  try {
    const [plansData, subscriptionData, invoicesData, usageData] = await Promise.all([
      billingApi.getPlans(),
      billingApi.getCurrentSubscription(),
      billingApi.getInvoices(),
      usageApi.getQuotas(),                    // NEW
    ])
    setPlans(plansData?.data || plansData || [])
    setCurrentSubscription(subscriptionData?.data || subscriptionData || null)
    setInvoices(invoicesData?.data || invoicesData || [])
    setUsage(usageData || null)                // NEW state variable

    // Refund waterfall (unchanged)...
  } catch (err) {
    console.error('Failed to load subscription:', err)
    setCurrentSubscription(null)
    setUsage(null)                             // Set null on error so bars hide gracefully
  } finally {
    setIsLoading(false)
  }
}
```

### UI-SPEC D-10: Error Isolation Pattern
```javascript
// Usage fetch errors should not block billing data rendering:
// The Promise.all will reject if usage fails. Solution:
// Option A (recommended): Separate the usage fetch into its own try/catch
const loadData = async () => {
  try {
    const [plansData, subscriptionData, invoicesData] = await Promise.all([
      billingApi.getPlans(),
      billingApi.getCurrentSubscription(),
      billingApi.getInvoices(),
    ])
    // ... set billing state ...

    // Fetch usage separately so failure doesn't block billing:
    try {
      const usageData = await usageApi.getQuotas()
      setUsage(usageData)
    } catch (e) {
      console.error('Failed to load usage:', e)
      setUsage(null) // bars hidden gracefully
    }

    // ... refund waterfall ...
  } catch (err) {
    // billing error handling (unchanged)
  }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `bg-gray-800 rounded-lg` | `glass-panel p-6` | Phase 79 | All containers get cosmic theme with backdrop blur and proper border colors |
| `border-2 border-gray-700` | `border border-[var(--color-cosmic-border)]` | Phase 79 | Thinner borders, consistent variable-based color |
| `bg-red-600 hover:bg-red-500` | `bg-[var(--color-cosmic-red)] hover:bg-[var(--color-cosmic-red)]/80` | Phase 79 | CSS variables enable light theme swapping |
| Raw limit numbers only | Usage bars with color thresholds | Phase 79 | New feature from QuotaService |
| `bg-green-500/20 text-green-400` | `bg-[var(--color-cosmic-green)]/20 text-[var(--color-cosmic-green)]` | Phase 79 | Already partially migrated; complete the migration |

**Deprecated/outdated:**
- `bg-gray-800` for card backgrounds — replaced by `glass-panel` (which uses `var(--color-cosmic-card)` = `rgba(255,255,255,0.03)`)
- `text-gray-400` for muted text — replaced by `text-[var(--color-text-muted)]`
- `text-gray-500` for labels — replaced by `text-[var(--color-text-muted)]`
- `border-2` for card borders — replaced by `border` (1px is the cosmic standard)
- `bg-gray-900` for page background — keep only in spinner loading state; page content uses `var(--color-deep-space)` via body background in index.css

## Assumptions Log

> All claims in this research were verified against the codebase (source files, CONTEXT.md, UI-SPEC.md). No user confirmation needed.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| — | No claims tagged `[ASSUMED]` | All | N/A |

## Open Questions (RESOLVED)

1. **[How should the Free tier display usage bars?]** — RESOLVED
   - Decision: Bars should show for all tiers. Unlimited resources show muted full bar with "Unlimited" label. If usage is null, section shows "Usage data unavailable" text.
   - Implemented in: 79-02 Plan Task 2 (Section 6A — usage bars rendered when usage is not null, muted full bar for max === -1)

2. **[Should the three subscription state sections be collapsed into two?]** — RESOLVED
   - Decision: Collapse into 2 states — "Active subscription" and "No active subscription or null" — eliminates duplicate code.
   - Implemented in: 79-02 Plan Task 2 (Sections 6A and 6B — active subscription vs everything else)

3. **[What happens if usage API returns error during loadData?]** — RESOLVED
   - Decision: Usage fetch moved OUT of main Promise.all into its own try/catch after billing data succeeds. Usage failure does not block billing data display.
   - Implemented in: 79-02 Plan Task 2 (Section 3 — separate try/catch for usageApi.getQuotas())

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Node.js | Frontend build | ✓ | (project standard) | — |
| Rust toolchain | Backend build | ✓ | (project standard) | — |
| PostgreSQL | QuotaService execution | ✓ | (project standard) | — |
| npm/yarn | Frontend dependencies | ✓ | (project standard) | — |
| Cargo | Backend dependencies | ✓ | (project standard) | — |

**Missing dependencies with no fallback:** None — all project dependencies are already established.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | No test framework detected for billing frontend; backend likely uses `cargo test` |
| Config file | None found for billing-specific tests |
| Quick run command | Manual: eyeball the page at `/billing` |
| Full suite command | Manual: verify all sections render correctly |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| D-01 | Section layout renders in correct order | Visual smoke | Manual | ❌ (no billing tests exist) |
| D-02 | Subscription card shows glass-panel + status badge | Visual smoke | Manual | ❌ |
| D-03 | Usage bars render with correct colors per threshold | Visual smoke | Manual | ❌ |
| D-04 | Plan cards show glass-panel + hover glow | Visual smoke | Manual | ❌ |
| D-05 | Cancel button triggers window.confirm | Manual | Manual | ❌ |
| D-06 | Refund button conditional on eligibility | Manual | Manual | ❌ |
| D-07 | Payment History table in glass-panel | Visual smoke | Manual | ❌ |
| D-08 | Refund eligibility with emoji indicators | Visual smoke | Manual | ❌ |
| D-09 | EscluseSpinner during load | Visual smoke | Manual | ❌ |
| D-10 | Parallel loading + waterfall refund | Manual | Manual | ❌ |
| Backend | GET /api/v1/usage/quotas returns 200 with valid JSON | Integration | `cargo test` (if usage test exists) | ❌ |

### Sampling Rate
- **Per task commit:** No automated tests available. Review the changed files visually.
- **Per wave merge:** Manual verification on staging environment at `/billing`.
- **Phase gate:** Visual sign-off on all D-01 through D-10 behaviors before `/gsd-verify-work`.

### Wave 0 Gaps
- [ ] No test infrastructure for frontend billing page exists. Adding tests would be out of scope for this phase (which is a restyle + route mount). Accept manual verification.
- [ ] Consider adding a backend integration test for `GET /usage/quotas` as part of the route mount plan, but not required — the handler already works; mounting is the only change.

*(No test gaps to fill — this phase is a visual restyle with a trivial backend change. Manual verification is appropriate.)*

## Security Domain

> `security_enforcement` key is absent from `.planning/config.json`. Treated as enabled by default.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes | JWT Bearer token via `Authorization` header — `api.js:28-30`. Middleware: `AuthUser` extractor in `usage_handlers.rs:36`. |
| V3 Session Management | yes | Token refresh in `api.js:43-49`. Session expiry → logout. |
| V4 Access Control | yes | `QuotaService::get_user_quota(auth_user.user_id)` — user can only see their own quota. Subscription lookup scoped to `auth_user.user_id`. |
| V5 Input Validation | yes | Monthly query params (`year`, `month`) validated as `Option<i32>` / `Option<u32>` in `usage_handlers.rs:15-18`. No user input on GET `/quotas`. |
| V6 Cryptography | no | No encryption/decryption in this phase. |

### Known Threat Patterns for Axum + React

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Unauthorized access to usage data | Spoofing | `AuthUser` middleware on `/usage/quotas` — validates JWT before handler executes. QuotaService scopes to `auth_user.user_id`. |
| IDOR (viewing another user's quota) | Information Disclosure | `QuotaService::get_user_quota()` takes `user_id: Uuid` from `AuthUser`, not from URL params. No user-controlled ID in the request path. |
| XSS via usage data display | Tampering | Usage data is numeric (i32). No user-controlled strings are displayed in usage bars. Plan names come from database, rendered as text content (not dangerouslySetInnerHTML). |

**Security assessment:** The usage endpoint follows the same security pattern as every other authenticated endpoint in the application (AuthUser middleware + user-scoped queries). No additional security controls needed for this phase.

## Sources

### Primary (HIGH confidence)
- [CODE: BillingPage.jsx] — Current 436-line implementation, all three subscription states, plan fallback, refund logic
- [CODE: api.js] — ApiClient with token refresh + response unwrapping, billingApi pattern
- [CODE: api_routes.rs] — UsageHandlers already imported at line 17, not yet mounted
- [CODE: usage_handlers.rs] — `get_quotas` handler returns `{ plan, limits, current_usage }` via ApiResponse
- [CODE: usage/service.rs] — `QuotaService::get_user_quota()` returns `UserQuota { plan_name, limits: PlanLimits, current_usage: CurrentQuotaUsage }`
- [CODE: plan/model.rs] — `PlanLimits` struct with all fields including `max_servers`, `max_nodes`, etc.
- [CODE: index.css] — `glass-panel` definition at line 75, cosmic theme variables at lines 4-19
- [CODE: SkeletonLoader.jsx] — `EscluseSpinner` component at lines 158-164
- [CODE: uiStore.js] — Toast pattern via `addToast()`
- [DOC: 79-CONTEXT.md] — All locked decisions, API reference, reusable patterns
- [DOC: 79-UI-SPEC.md] — Interaction contracts D-01 through D-10, copywriting contract, spacing/color/typography specifications
- [CODE: ProfilingPanel.jsx] — Existing progress bar pattern at lines 4-18 (`h-2 bg-[...] rounded-full overflow-hidden` + inner `h-full rounded-full transition-all`)
- [CODE: TemplateCard.jsx] — Glass-panel + hover glow pattern at lines 8-9

### Secondary (MEDIUM confidence)
- [CODE: App.jsx] — Page layout uses `bg-gray-900` at line 55; billing page restyle doesn't change page-level bg
- [CODE: usage/model.rs] — `CurrentUsage` struct confirms `servers`, `ram_mb`, `cpu_cores`, `disk_gb`, `bandwidth_gb` fields
- [CODE: api_response.rs] — `ApiResponse<T>` struct with `success: bool`, `data: Option<T>`, `error: Option<...>`

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — verified against project dependencies and source code
- Architecture: HIGH — all patterns confirmed in existing codebase
- Pitfalls: HIGH — based on direct code analysis of edge cases
- API response shape: HIGH — verified `ApiResponse` struct + `api.js` unwrapping logic
- Usage handler completeness: HIGH — verified handler exists and QuotaService has `get_user_quota()`

**Research date:** 2026-06-15
**Valid until:** 2026-07-15 (30 days — stable dependencies, no fast-moving libraries)
