# Phase 77: Update UI /templates — Research

**Researched:** 2026-06-14
**Domain:** React frontend (Template Library + Create/Edit form)
**Confidence:** HIGH

## Summary

This phase redesigns the Templates section at `/templates` with filter tabs (Featured/Yours/All), sort controls, category filter, enriched card info (version, last updated, usage count), table view toggle, delete confirmation modal, and form styling refinements. Two detailed plans (77-01 and 77-02) already exist covering the frontend work completely.

**CRITICAL FINDING:** The existing plans assume `template.version` and `template.usage_count` fields exist in the API response (`GET /api/v1/templates`). **These fields do not currently exist** in either the `api/` or `migration/` Rust backend. The `TemplateResponse` DTO lacks both fields. These must be added to the backend before (or as part of) the frontend work, or the frontend must use alternative data sources.

**Primary recommendation:** Execute the existing plans as written (77-01 for library page, 77-02 for form styling), but add the required backend field additions as a Wave 0 prerequisite. The backend changes are small (add 2 fields to the Template model, DTO, and queries).

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Replace stacked sections (Featured + Your Templates) with filter tabs: "Featured" / "Yours" / "All".
- **D-02:** Cards show additional info: version tag, last updated date, usage count (servers using this template).
- **D-03:** Add sorting (name, updated, popular) and category filter alongside existing search + game type filter.
- **D-04:** Filter/sort preferences not persisted (session-only).
- **D-05:** Keep existing 2-section layout (Basic Info + Configuration).
- **D-06:** Refine styling and form field presentation (cosmic theme consistency).
- **D-07:** No new form sections or fields.
- **D-08:** No changes to Mod Browser page. Scope limited to Library and Create/Edit form.

### The agent's Discretion
- Exact tab design for Featured/Yours/All (pill buttons or tab bar)
- Sort options and category filter UI placement
- Version/updated/count layout within cards
- Form styling details

### Deferred Ideas (OUT OF SCOPE)
- Quick-install from template library (one-click deploy) — future phase
- Mod Browser UI improvements — future phase
- Template icon/screenshot upload — future phase
</user_constraints>

## ⚠️ Critical Backend Gaps

### Gap 1: `version` field missing from Template domain model and response DTO

The existing plans at `77-01-PLAN.md` reference `template.version` in both the enriched card (TemplateCard.jsx) and the table view, but this field **does not exist** anywhere in the Rust backend:

- **File:** `api/src/domain/server/template/model.rs` — `Template` struct has no `version` field [VERIFIED: file read]
- **File:** `api/src/application/dto/template_dtos.rs` — `TemplateResponse` struct has no `version` field [VERIFIED: file read]
- **File:** `migration/src/application/dto/template_dtos.rs` — Same, no `version` field [VERIFIED: file read]

**Fix options:**
| Option | Effort | Recommended? |
|--------|--------|-------------|
| A. Add `version: Option<String>` to Template model + TemplateResponse | Small (add field to struct, DB migration, DTO mapping) | ✅ **Recommended** — matches plan assumption exactly |
| B. Use `config.docker_image` tag as version (e.g., `latest` from `itzg/minecraft-server:latest`) | Zero backend change, but semantically wrong | ❌ Docker tag != template version |
| C. Omit version tag from cards entirely | Zero backend change, but violates D-02 | ❌ User locked in D-02 |

### Gap 2: `usage_count` field missing from TemplateResponse

The plans reference `template.usage_count` in cards and table view, but this field **does not exist**:

- **File:** `api/src/application/dto/template_dtos.rs` — `TemplateResponse` has no `usage_count` field [VERIFIED: file read]
- **File:** `api/src/domain/server/template/model.rs` — `Template` has no `usage_count` field [VERIFIED: file read]
- **No SQL query** in `api/src/domain/server/template/repository.rs` counts servers per template [VERIFIED: file read lines 230-276 — list_public_templates does not LEFT JOIN or subquery server count]
- **Database hint:** Servers table has `template_id` column (`Option<Uuid>`) in the domain model at `model.rs:119`. Usage count can be derived via `LEFT JOIN (SELECT template_id, COUNT(*) FROM servers GROUP BY template_id) AS usage ON usage.template_id = t.id` in the `list_public_templates` query.

**Fix options:**
| Option | Effort | Recommended? |
|--------|--------|-------------|
| A. Add `usage_count: i64` with default `0` to TemplateResponse; modify `list_public_templates` SQL to LEFT JOIN the servers count | Medium (add field, modify query in repository.rs, add DTO mapping) | ✅ **Recommended** — matches plan assumption |
| B. Compute usage count in a separate query per template | Higher (N+1 queries) | ❌ Inefficient |
| C. Compute usage count client-side — not feasible without fetching all servers | Impossible | ❌ |

### Gap 3: Double codebase (api/ vs migration/)

Both `api/src/` and `migration/src/` have identical `template_dtos.rs` and `template/model.rs`. If backend changes are made to `api/` but `migration/` is the deployed service (or vice versa), the new fields won't be available.

- `docker-compose.yml` at root was checked — no `backend` service section found (empty output). Multiple compose files exist.
- **Recommendation:** Confirm with user which codebase is active before making backend changes. Apply changes in both directories if uncertain, as the existing plans for Phase 78 recommend.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| React | ^19.2.4 | UI framework | Project standard — confirmed in `app/package.json` |
| react-router-dom | ^7.13.0 | Routing | `/templates`, `/templates/create`, `/templates/:id/edit` routes in `App.jsx` |
| zustand | ^5.0.12 | State management | `useAuthStore`, `useUIStore` established patterns |
| lucide-react | ^1.18.0 | Icons | `Calendar`, `Users`, `LayoutGrid`, `List`, `X` needed |
| Tailwind CSS v4 | ^4.2.0 | Styling | Cosmic theme via `@theme` in `index.css` |
| Vite | ^7.3.1 | Build tooling | Project standard |

**Installation:**
```bash
# All dependencies already installed — no new packages needed
```

### Already Available APIs
| API | Method | Endpoint | Status |
|-----|--------|----------|--------|
| `templatesApi.list` | `GET /api/v1/templates` | ✅ Exists — returns `TemplateResponse[]` |
| `templatesApi.get` | `GET /api/v1/templates/:id` | ✅ Exists — returns single `TemplateResponse` |
| `templatesApi.create` | `POST /api/v1/templates` | ✅ Exists — accepts `CreateTemplateRequest` |
| `templatesApi.update` | `PUT /api/v1/templates/:id` | ✅ Exists — accepts `UpdateTemplateRequest` |
| `templatesApi.delete` | `DELETE /api/v1/templates/:id` | ✅ Exists — returns `{ status: "deleted" }` |
| `templatesApi.createServer` | `POST /api/v1/templates/:id/create-server` | ✅ Exists |

### Current TemplateResponse Fields
```json
{
  "id": "uuid",
  "game_type": "string",
  "category": "string",
  "display_name": "string",
  "description": "string | null",
  "config": { "docker_image": "...", "default_port": 25565, "env": {} },
  "visibility": "public|private",
  "user_id": "uuid | null",
  "is_builtin": true,
  "is_active": true,
  "created_at": "2026-06-01T12:00:00",
  "updated_at": "2026-06-10T12:00:00"
  // MISSING: "version": "string | null"
  // MISSING: "usage_count": 5
}
```

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│  Browser (React)                                                │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  TemplateLibraryPage.jsx (/templates)                     │  │
│  │  ┌──────────┐  ┌───────────┐  ┌──────────┐  ┌─────────┐ │  │
│  │  │  Filter  │  │  Sort     │  │  Search  │  │  View   │ │  │
│  │  │  Tabs    │  │  Select   │  │  Input   │  │  Toggle │ │  │
│  │  │ Feat/Yrs │  │ name/upd/ │  │          │  │ card/tbl│ │  │
│  │  │ /All     │  │ popular   │  │          │  │         │ │  │
│  │  └──────────┘  └───────────┘  └──────────┘  └─────────┘ │  │
│  │                                                           │  │
│  │  ┌─────────────────┐  ┌──────────────────┐               │  │
│  │  │  TemplateCard    │  │  Table view      │               │  │
│  │  │  (enriched card) │  │  (7 columns)     │               │  │
│  │  └─────────────────┘  └──────────────────┘               │  │
│  │                                                           │  │
│  │  ┌────────────────────────────────────────────┐          │  │
│  │  │  Delete Confirmation Modal                  │          │  │
│  │  │  "Delete Template?" → confirmDelete()       │          │  │
│  │  └────────────────────────────────────────────┘          │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  TemplateCreatePage.jsx (/templates/create, /:id/edit)   │  │
│  │  ┌──────────────────────┐  ┌──────────────────────────┐  │  │
│  │  │  Basic Info Section  │  │  Configuration Section   │  │  │
│  │  │  (name, game, cat,   │  │  (Docker image, port,    │  │  │
│  │  │   desc, visibility)  │  │   env JSON)              │  │  │
│  │  └──────────────────────┘  └──────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                 │
│  ◄—— templatesApi (api.get / api.post / api.put / api.delete) ► │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  zustand Stores                                           │  │
│  │  ┌──────────────┐  ┌──────────────┐                      │  │
│  │  │ useAuthStore │  │ useUIStore   │                      │  │
│  │  │ (user, role) │  │ (addToast)   │                      │  │
│  │  └──────────────┘  └──────────────┘                      │  │
│  └───────────────────────────────────────────────────────────┘  │
└──────────────────────────────┬──────────────────────────────────┘
                               │ HTTP
                               ▼
┌──────────────────────────────────────────────────────────────────┐
│  Rust Backend                                                    │
│  ┌─────────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │ GET /templates  │  │ POST/PUT/    │  │ POST /templates/ │   │
│  │ (list all)      │  │ DELETE       │  │ :id/create-server│   │
│  │                 │  │ /templates   │  │ (create server)  │   │
│  └────────┬────────┘  └──────┬───────┘  └────────┬─────────┘   │
│           │                  │                    │              │
│           ▼                  ▼                    ▼              │
│   ┌────────────────────────────────────────────────────────┐    │
│   │  PostgreSQL                                             │    │
│   │  • templates table (id, game_type, category, ...)       │    │
│   │  • servers table (has template_id FK → templates.id)    │    │
│   └────────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────────┘
```

**Primary use case trace (Filter → Create Server):**
```
User clicks "Featured" tab → setActiveTab('featured') → 
filtered = templates.filter(t => t.is_builtin) →
client-side sort + render TemplateCard grid →
User clicks "Create Server" on a card →
navigates to /templates/:id → ServerManagerPage (create flow)
```

### Recommended Project Structure (no new files)

```
api/src/
├── application/dto/template_dtos.rs    # ADD version, usage_count fields
├── domain/server/template/model.rs     # ADD version field to Template
├── domain/server/template/repository.rs # MODIFY SQL queries to include usage_count
│
app/src/
├── pages/templates/
│   ├── TemplateLibraryPage.jsx         # MAJOR rewrite (77-01) — filters, tabs, sort, table view, delete modal
│   └── TemplateCreatePage.jsx          # Minor changes (77-02) — focus rings, checkbox accent, vars
└── components/
    └── TemplateCard.jsx                # Enriched (77-01) — version tag, last updated, usage count
```

### Pattern 1: Filter Tab Pattern (D-01)
**What:** Three pill-shaped filter tabs (Featured/Yours/All) below page header, above filter bar. Active tab uses cyan highlight.
**Source:** 77-UI-SPEC.md lines 146-155 [VERIFIED: UI design contract]

```jsx
<div className="flex gap-2 mb-4" role="tablist">
  {['featured', 'yours', 'all'].map(tab => (
    <button key={tab} onClick={() => setActiveTab(tab)}
            role="tab" aria-selected={activeTab === tab}
            className={`px-4 py-2 text-sm rounded-lg transition-colors ${
              activeTab === tab
                ? 'bg-[var(--color-cosmic-cyan)]/10 text-[var(--color-cosmic-cyan)] border border-[var(--color-cosmic-cyan)]/30'
                : 'text-[var(--color-text-muted)] hover:text-white hover:bg-[var(--color-cosmic-card)]'
            }`}>
      {tab.charAt(0).toUpperCase() + tab.slice(1)}
    </button>
  ))}
</div>
```

### Pattern 2: Sort/Filter Dropdown (D-03)
**What:** All filter selects use the same cosmic-theme styling pattern.
**Source:** 77-UI-SPEC.md lines 157-166 [VERIFIED: UI design contract]

```jsx
<select value={sortMode} onChange={e => setSortMode(e.target.value)}
        className="px-4 py-2.5 rounded-lg text-sm bg-[var(--color-cosmic-card)]/60
                   border border-[var(--color-cosmic-border)] text-[var(--color-text-main)]">
  <option value="name">Name A-Z</option>
  <option value="updated">Last Updated</option>
  <option value="popular">Most Used</option>
</select>
```

### Pattern 3: Delete Confirmation Modal (replaces `confirm()`)
**What:** Inline modal overlay with cosmic theme styling, backdrop dismiss, Cancel/Delete buttons.
**Source:** 77-UI-SPEC.md lines 205-212 [VERIFIED: UI design contract]

```jsx
{deleteConfirm && (
  <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
       onClick={() => setDeleteConfirm(null)}>
    <div className="bg-[var(--color-cosmic-card)] border border-[var(--color-cosmic-border)]
                    rounded-lg p-6 max-w-md w-full mx-4"
         onClick={e => e.stopPropagation()}>
      <h3 className="text-lg font-semibold text-[var(--color-text-main)] mb-2">Delete Template?</h3>
      <p className="text-sm text-[var(--color-text-muted)] mb-6">
        Are you sure you want to delete <strong>{deleteConfirm.display_name}</strong>?
        Servers already created from this template will not be affected.
      </p>
      <div className="flex gap-2 justify-end">
        <button onClick={() => setDeleteConfirm(null)}
                className="px-4 py-2 rounded-lg text-sm border border-[var(--color-cosmic-border)]
                           text-[var(--color-text-muted)] hover:text-[var(--color-text-main)]">
          Cancel
        </button>
        <button onClick={confirmDelete}
                className="px-4 py-2 rounded-lg text-sm font-semibold bg-[var(--color-cosmic-red)] text-white">
          Delete
        </button>
      </div>
    </div>
  </div>
)}
```

### Pattern 4: Relative Time Formatting (no library needed)
**What:** `Intl.RelativeTimeFormat` or hand-rolled function — no date library in project. Plans use a hand-rolled `formatRelativeTime` helper.
**Note:** No `date-fns`, `dayjs`, `moment`, or `timeago.js` in `package.json`. Hand-rolled function is consistent with project style.

### Pattern 5: Toast Notification Pattern
**What:** Success/error feedback via zustand store. [VERIFIED: uiStore.js lines 21-27]
```javascript
const { addToast } = useUIStore()
addToast({ type: 'success', message: 'Template deleted' })
addToast({ type: 'error', message: 'Failed to delete template: ...' })
```
Auto-dismisses after 5000ms. Bottom-right positioning via `ToastContainer.jsx`.

### Pattern 6: Cosmic Theme Form Input
**What:** All form inputs follow a consistent class pattern. [VERIFIED: TemplateCreatePage.jsx lines 114-117]
```jsx
<input type="text" value={form.display_name} onChange={...}
       className="w-full px-4 py-2.5 rounded-lg text-sm bg-[var(--color-cosmic-card)]/60
                  border border-[var(--color-cosmic-border)] text-white
                  focus:outline-none focus:ring-2 focus:ring-[var(--color-cosmic-cyan)]/50
                  focus:border-[var(--color-cosmic-cyan)]/50" />
```

### Anti-Patterns to Avoid
- **`confirm()` dialog for delete:** Currently on `TemplateLibraryPage.jsx` line 33. Must be replaced with inline modal + toast.
- **Non-cosmic class names on back link:** `text-gray-400 hover:text-white` on line 99 of TemplateCreatePage — must be `text-[var(--color-text-muted)] hover:text-[var(--color-text-main)]`.
- **Generic accent color on checkbox:** `accent-cyan-500` on line 174 of TemplateCreatePage — must be `accent-[var(--color-cosmic-cyan)]`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| State management | Custom state container | zustand | Already in project — `useAuthStore`, `useUIStore` established |
| Icons | SVG icons | lucide-react | Already in project — `Calendar`, `Users`, `LayoutGrid`, `List` available |
| HTTP client | Raw fetch with auth | `api` client from `lib/api.js` | Handles token refresh, error parsing, base URL |
| Toast notifications | Custom toast component | `useUIStore().addToast()` | Already implemented in `uiStore.js` + `ToastContainer.jsx` |
| Loading spinners | Custom spinner | `EscluseSpinner` from SkeletonLoader | Already in project |

**Key insight:** This phase needs NO new npm packages. All required libraries (zustand, lucide-react, react-router-dom) are already in `package.json`. The only "new" things are backend DTO fields and SQL query modifications.

## Common Pitfalls

### Pitfall 1: Backend DTO fields not updated (CRITICAL)
**What goes wrong:** Frontend tries to access `template.version` and `template.usage_count` but they're `undefined` because the Rust backend DTOs don't include these fields.
**Root cause:** The `TemplateResponse` struct in `api/src/application/dto/template_dtos.rs` and the `Template` domain model lack both fields.
**How to avoid:** Add backend field changes as Wave 0/Task 0 before any frontend work.
**Warning signs:** Card shows no version tag, usage count shows `0 servers` even for popular templates.

### Pitfall 2: Double codebase (api/ vs migration/)
**What goes wrong:** Backend changes made to `api/src/` but `migration/src/` is the actively deployed service, or vice versa.
**Root cause:** Two near-identical Rust codebases; unclear which is production.
**How to avoid:** Confirm with user before making backend changes. Apply changes in both directories if both are active.
**Warning signs:** Backend changes deploy but have no effect.

### Pitfall 3: Category filter parameter sent to API but no backend endpoint expects it
**What goes wrong:** Plans assume category filter is client-side (D-04 says no persistence, client-side filtering only), but if accidentally sent to API, the API doesn't support `?category=` filtering.
**Root cause:** D-04 explicitly says session-only client-side filtering. Plans correctly handle this via `useMemo` client-side.
**How to avoid:** Keep category filtering purely client-side as the plans do — do not add `category` to `templatesApi.list()` params.
**Warning signs:** N/A — plans correctly implement client-side filtering.

### Pitfall 4: Table view toggle accidentally persisted to localStorage
**What goes wrong:** View preference ('card' vs 'table') is session-only per D-04, but if someone adds localStorage persistence it would violate the decision.
**Root cause:** D-04 says no persistence. Phase 75 persisted view preference (D-03 in 75-CONTEXT), but Phase 77 explicitly does NOT.
**How to avoid:** Use `useState` only, not `useLocalStorage` or similar.
**Warning signs:** Page reload restores table view — means it's persisting.

### Pitfall 5: Focus rings added inconsistently
**What goes wrong:** Some form elements get focus rings, others don't.
**Root cause:** The 8 inputs/selects/textarea in TemplateCreatePage.jsx have different class patterns — easy to miss one.
**How to avoid:** Use a systematic approach: find-replace each `text-white"` (end of input/select/textarea className) with focus ring suffix.
**Warning signs:** Tab through form — some fields show focus ring, some don't.

## Code Examples

### Verify API Response Shape
```javascript
// Check what fields actually come back from the API
const data = await templatesApi.list({})
const t = data[0]
console.log(Object.keys(t))
// Should include: id, game_type, category, display_name, description, config,
// visibility, user_id, is_builtin, is_active, created_at, updated_at
// Should NOT include: version, usage_count (until backend changes added)
```

### Filter by Tab (D-01)
```javascript
if (activeTab === 'featured') result = result.filter(t => t.is_builtin)
else if (activeTab === 'yours') result = result.filter(t => !t.is_builtin)
```

### Sort (D-03)
```javascript
switch (sortMode) {
  case 'name':    result.sort((a, b) => (a.display_name || '').localeCompare(b.display_name || '')); break
  case 'updated': result.sort((a, b) => new Date(b.updated_at || b.created_at) - new Date(a.updated_at || a.created_at)); break
  case 'popular': result.sort((a, b) => (b.usage_count || 0) - (a.usage_count || 0)); break
}
```

### Derive Categories from Data
```javascript
const categories = useMemo(() =>
  [...new Set(templates.map(t => t.category).filter(Boolean))],
  [templates]
)
```

### formatRelativeTime (no deps needed)
```javascript
function formatRelativeTime(isoString) {
  if (!isoString) return '—'
  const now = Date.now()
  const then = new Date(isoString).getTime()
  const diffSec = Math.floor((now - then) / 1000)
  if (diffSec < 60) return 'just now'
  const diffMin = Math.floor(diffSec / 60)
  if (diffMin < 60) return `${diffMin}m ago`
  const diffHr = Math.floor(diffMin / 60)
  if (diffHr < 24) return `${diffHr}h ago`
  const diffDay = Math.floor(diffHr / 24)
  if (diffDay === 1) return 'Yesterday'
  if (diffDay < 30) return `${diffDay}d ago`
  return `${Math.floor(diffDay / 30)}mo ago`
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Stacked sections (Featured + Your Templates) | Filter tabs (Featured/Yours/All) | This phase | Better exploration UX |
| Basic card (name, game, description, badges) | Enriched card (+ version, last updated, usage) | This phase | Users can evaluate templates at a glance |
| Game type filter only | Game type + category + sort + search | This phase | Multi-dimensional filtering |
| `confirm()` dialog for delete | Inline modal with Cancel/Delete + toast | This phase | Better UX, undo-friendly pattern |
| Plain `text-white` / `text-gray-400` | Cosmic CSS variables throughout | This phase | Theme consistency |
| Missing focus rings on form fields | Cosmic-cyan focus rings on all inputs | This phase | Accessibility + consistency |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Backend `api/src/` is the active deployed codebase, not `migration/src/` | Critical Backend Gaps | Backend changes in wrong directory won't deploy |
| A2 | The `version` field should be added to the Template model as `Option<String>` | Critical Backend Gaps | If `version` is meant to be derived differently (e.g., from Docker image tag), the field addition approach is wrong |
| A3 | Usage count can be derived by LEFT JOIN on `servers.template_id` | Critical Backend Gaps | If servers don't consistently set `template_id`, usage count will be 0 |
| A4 | User wants both `api/` and `migration/` directories kept in sync | Critical Backend Gaps | If `migration/` is deprecated, only `api/` needs changes |
| A5 | `npm run build` from `app/` is sufficient build verification | Validation Architecture | No test framework exists — build passing doesn't verify logic correctness |

## Open Questions (RESOLVED)

1. **What should `version` represent on a template?** (RESOLVED)
   - What we know: The CONTEXT.md says "Version tag could be from template.properties.version or template.created_at". Plans assume a `version` field on the Template model.
   - What's unclear: Does the database already have a version column? Is it meant to be a free-text field (like "v1.0"), a game version (like "1.21"), or a template schema version?
   - Plan resolution: `77-01-PLAN.md` adds `template.version` as `Option<String>` — a free-text nullable field. Backend change is needed.
   - RESOLVED: Version is an optional free-text metadata field (e.g. "1.0", "v2.1"). No DB column exists yet — the plan adds it.

2. **Is `migration/src/` active or deprecated?** (RESOLVED)
   - What we know: Both `api/` and `migration/` have near-identical `template_dtos.rs` files.
   - Recommendation: Apply backend changes to both directories for safety, or confirm with user.
   - RESOLVED: Plan 77-01 applies changes to **both** `api/` and `migration/` codebases — the safe dual-codebase approach. No user confirmation needed.

3. **Does the servers table consistently store `template_id`?** (RESOLVED)
   - What we know: Domain model has `template_id: Option<Uuid>` on Server entity. The `apply_template_to_server` handler sets it at line 210.
   - What's unclear: Are servers created outside the template flow missing `template_id`? This would make usage_count under-report.
   - Recommendation: Check if older servers/template-free servers have NULL template_id before relying on the count.
   - RESOLVED: Plan 77-01 uses `#[sqlx(default)]` on `usage_count` + `COALESCE(usage_stats.usage_count, 0)` in the LEFT JOIN + `WHERE template_id IS NOT NULL` filter. These three mitigations ensure correct behavior even if some servers have NULL template_id.

## Environment Availability

> Step 2.6: SKIPPED (no external dependencies beyond existing project infrastructure — all tooling already established for Phases 75-80)

## Validation Architecture

Since `workflow.nyquist_validation` is absent from `.planning/config.json`, treat as enabled.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | None detected in `app/package.json` — no testing dependencies |
| Config file | None detected |
| Quick run command | `npm run build` from `app/` (build verification only) |
| Full suite command | `npm run build` from `app/` |

### Phase Requirements → Test Map
No testing framework is configured. The only automated validation is `npm run build` exits with code 0.

Verification is done via grep-based checks defined in the plans:
- Plan 77-01: 12 automated grep checks (filter tabs, sort, category filter, view toggle, delete modal, card enrichment, error state, empty states)
- Plan 77-02: 5 automated grep checks (focus rings, checkbox accent, back link, page title, brightness)

### Wave 0 Gaps
- [ ] No test framework configured — `npm run build` is the only validation available
- [ ] Backend compilation check: `cargo build` from `api/` directory — verify backend compiles after DTO changes

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | Auth handled by existing JWT middleware — no changes in this phase |
| V3 Session Management | No | No session changes |
| V4 Access Control | No | Template edit/delete gated by backend ownership check — no frontend authorization changes |
| V5 Input Validation | Yes | Filter/sort/search is client-side only — no new API params introduced |
| V6 Cryptography | No | No cryptographic operations |

### Known Threat Patterns for React + Rust

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Double-click delete submission | DoS | Button disabled via `setDeleteConfirm(null)` after first click — already handled in plan |
| Template ID tampering | Tampering | Backend validates ownership before delete — frontend confirmation is UX-only |
| Search query XSS | Spoofing | React's JSX auto-escapes — no `dangerouslySetInnerHTML` used in cards or table |

## Sources

### Primary (HIGH confidence)
- `api/src/application/dto/template_dtos.rs` — TemplateResponse current fields [VERIFIED: file read]
- `api/src/domain/server/template/model.rs` — Template domain model [VERIFIED: file read]
- `api/src/domain/server/template/repository.rs` — Template SQL queries [VERIFIED: file read]
- `api/src/application/use_cases/template_use_cases.rs` — Template business logic [VERIFIED: file read]
- `api/src/presentation/handlers/template_handlers.rs` — Template API handlers [VERIFIED: file read]
- `migration/src/application/dto/template_dtos.rs` — Mirror codebase DTO [VERIFIED: file read]
- `app/src/pages/templates/TemplateLibraryPage.jsx` — target page current state [VERIFIED: file read]
- `app/src/pages/templates/TemplateCreatePage.jsx` — target form current state [VERIFIED: file read]
- `app/src/components/TemplateCard.jsx` — card component current state [VERIFIED: file read]
- `app/src/api/templatesApi.js` — API client [VERIFIED: file read]
- `app/src/lib/api.js` — API client base [VERIFIED: file read]
- `app/src/store/uiStore.js` — toast pattern [VERIFIED: file read]
- `app/src/store/authStore.js` — user/role pattern [VERIFIED: file read]
- `app/src/components/SkeletonLoader.jsx` — spinner component [VERIFIED: file read]
- `app/src/components/InviteFriendsModal.jsx` — modal overlay pattern [VERIFIED: file read]
- `app/src/index.css` — cosmic theme variables [VERIFIED: file read]
- `app/package.json` — dependency list [VERIFIED: file read]
- `77-CONTEXT.md` — phase locked decisions [VERIFIED: file read]
- `77-UI-SPEC.md` — UI design contract [VERIFIED: file read]
- `77-01-PLAN.md` — existing plan for frontend Wave 1 [VERIFIED: file read]
- `77-02-PLAN.md` — existing plan for frontend Wave 2 [VERIFIED: file read]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all confirmed via file reads
- Architecture: HIGH — confirmed via file reads and pattern analysis
- Backend gaps: HIGH — confirmed via DTO/struct file reads; plans assume fields that don't exist
- Pitfalls: HIGH — directly derived from codebase analysis
- Security: HIGH — no auth/access control changes

**Research date:** 2026-06-14
**Valid until:** 2026-07-14 (stable project — 30-day validity)
