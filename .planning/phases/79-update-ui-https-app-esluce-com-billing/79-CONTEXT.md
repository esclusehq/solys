# Phase 79: Update UI https://app.esluce.com/billing — CONTEXT

## Source Files
- **Frontend:** `app/src/pages/billing/BillingPage.jsx` (436 lines, single-file page)
- **Backend:** `api/src/presentation/handlers/billing_handlers.rs` (existing endpoints)
- **Backend usage handler:** `api/src/presentation/handlers/usage_handlers.rs` (exists, not mounted)
- **Routes:** `api/src/presentation/routes/api_routes.rs`
- **Quota service:** `api/src/domain/usage/service.rs` (`QuotaService::get_user_quota()`)

## Current State
- Flat `bg-gray-800 rounded-lg` styling — inconsistent with cosmic `glass-panel` theme used everywhere else
- Vertical stacked layout: Subscription → Refund → Plans (2-column grid) → Invoices
- No usage/progress indicators, only raw limit numbers
- Refund section only shows when subscription + eligibility data exist
- Plan cards have hard-coded fallback data (Hobby $6.99, Pro $24.99) if API returns empty

## Decisions

### Visual Consistency (Cosmic Theme)
- Lift all containers: `bg-gray-800 rounded-lg` → `glass-panel p-6` with cosmic border accents
- Use `border border-[var(--color-cosmic-border)]` on panels
- Use `hover:border-[var(--color-cosmic-cyan)]/40` on plan cards (matching PluginManager pattern)
- Keep spinner loading state as-is (consistent with other pages)
- DO NOT change font sizes, heading hierarchy, or button styling patterns — they already match

### Page Organization
- Keep vertical stacked layout, DO NOT introduce tabs or sidebars
- Section order: Subscription Status → Plan Cards (Hobby/Pro) → Payment History (invoices) → Refund sections
- Add a subtle visual separator between sections (e.g., `border-b border-[var(--color-cosmic-border)]`)

### Usage Bars (New Feature)
- Add **progress bars** in the subscription card for: servers, nodes, RAM, CPU, disk
- Each bar shows: `current / max` numeric label + fill bar
- Max = -1 means "Unlimited" — show full bar with "Unlimited" label, no fill
- Fetch from existing `QuotaService::get_user_quota()` via a new route
- Backend change needed: add `.nest("/api/v1/usage", UsageHandlers::router(state.clone()))` to `api_routes.rs` (UsageHandlers already imported)
- Frontend API call: new `usageApi.getQuotas()` → `api.get('/usage/quotas')` — returns `{ plan, limits, current_usage }`
- Call alongside billing data in `loadData()`
- `current_usage` includes: `servers` (count), `ram_mb` (total), `cpu_cores` (total), `disk_gb` (total) — does NOT include node count. For nodes usage, use the `Node` resource query pattern or accept node count not displayed in bars. Decision: keep node count in limits display as text only, no bar.
- Bar colors: green (< 60%), yellow (60-85%), red (> 85%) thresholds

### Refund Section
- Keep as-is functionally — only restyle with cosmic theme
- Keep the emoji indicators (🟢🟡🔴) for refund eligibility status
- Keep "Berhenti Berlangganan" button (Indonesian locale intentional)

### Plan Cards
- Keep side-by-side 2-column grid, same layout
- Replace `bg-gray-800 rounded-lg border-2 border-gray-700` with `glass-panel`
- Add hover glow effect matching other cosmic cards
- Keep all current content: icon, name, price, description, limits, features, best-for, CTA

### Payment History (Invoices)
- Keep table format, restyle container to glass-panel
- Keep current columns: Date, Amount, Status (with color badges)

### What's NOT Changing
- No tabs/sidebar reorganization
- No new sections (usage tracking history, etc.)
- No backend refactoring beyond the single route mount line
- No API contract changes for existing billing endpoints
- No locale changes (Indonesian labels stay)
- No mobile-specific layout changes beyond existing responsive grid

## Reusable Patterns
- `glass-panel` class (defined in global CSS) for all containers
- `border border-[var(--color-cosmic-border)]` for default panel borders
- `hover:border-[var(--color-cosmic-cyan)]/40 transition-all` for interactive card hover (from PluginManager.jsx)
- `bg-[var(--color-cosmic-cyan)]/10 text-[var(--color-cosmic-cyan)]` for status badges (from Nodes.jsx)
- Progress bar: custom div-based (no library) — `<div className="h-2 rounded-full bg-gray-700"><div className="h-full rounded-full bg-green-500" style={{width: X%}} /></div>`

## API Reference
- `GET /api/v1/usage/quotas` — returns `{ plan, limits, current_usage }`
- Usage data loaded alongside billing data in `loadData()` via `Promise.all()`
- Current usage structure: `CurrentQuotaUsage { servers: i32, ram_mb: i32, cpu_cores: i32, disk_gb: i32 }`
- Limits structure: `PlanLimits { max_servers: i32, max_nodes: i32, max_ram_mb: i32, max_cpu_cores: i32, max_disk_gb: i32 }` (-1 = unlimited)

## Out of Scope (Deferred)
- Usage history charts / monthly breakdown
- Bandwidth usage tracking display
- Payment method management UI
- Invoice download
- Annual pricing toggle
