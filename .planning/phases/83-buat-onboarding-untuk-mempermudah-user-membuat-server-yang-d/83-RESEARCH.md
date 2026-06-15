# Phase 83: Buat Onboarding Server — Research

**Researched:** 2026-06-16
**Domain:** React frontend — multi-step wizard modal for server creation
**Confidence:** HIGH

## Summary

Phase 83 introduces a `ServerOnboardingWizard` component — a 4-step modal wizard that replaces the current `navigate('/servers')` behavior when first-time users click "Create your first server" on the dashboard empty state. The wizard reuses existing API logic (`serversApi.create()`, `MINECRAFT_VERSIONS`, template fetching) from `CreateServerModal.jsx` but presents it as a guided multi-step flow (Type → Resources → Config → Deploy) rather than a single form. The existing `CreateServerModal` remains untouched for experienced users accessing `/servers`.

**Primary recommendation:** Build a single `ServerOnboardingWizard.jsx` component in `app/src/features/server/` using component-local `useState` for step management (4 steps is too simple to justify a Zustand store or a wizard library). Use `useRef` to persist wizard state across steps, and reuse `serversApi.create()` from the existing API layer. No new dependencies needed.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Clicking "Create your first server" on dashboard empty state opens a new ServerOnboardingWizard modal (not navigate to /servers)
- **D-02:** Existing CreateServerModal stays unchanged for experienced users accessing from /servers page
- **D-03:** Wizard is a multi-step modal with progress bar — not side panel or tooltip overlay
- **D-04:** Step 1 (Type): 3 large cards — Java Edition, Bedrock, PocketMine — with icons, descriptions, and recommendations. User clicks one to select.
- **D-05:** Step 2 (Resources): Show available plan cards (Free/Hobby/Pro) with feature lists. User picks a plan, then can configure RAM/CPU/Disk within the plan's limits.
- **D-06:** Step 3 (Config): Full server configuration — name, Minecraft version (auto-select latest), port (with random suggest), template (optional).
- **D-07:** Step 4 (Deploy): Review summary of all choices + "Deploy Server" button. After successful creation, auto-redirect to the new server's detail page.
- **D-08:** New component: `ServerOnboardingWizard` — separate from `CreateServerModal`
- **D-09:** Reuse API create server logic (same fetchApi/serversApi.create call)
- **D-10:** CreateServerModal remains as-is for experienced users on /servers page

### the agent's Discretion
- Exact visual design of cards in Step 1 (icon size, layout density)
- Progress bar style and positioning
- Default values for port (random), version (latest), template (none)
- Plan card layout in Step 2
- Deploy animation/loading state after clicking "Deploy Server"
- Confetti or celebratory element on success

### Deferred Ideas (OUT OF SCOPE)
- None
</user_constraints>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Wizard modal render | Browser (Client) | — | Entire wizard runs client-side as a React modal overlay |
| Step navigation (back/next) | Browser (Client) | — | Component-local state, no server interaction |
| Game type selection | Browser (Client) | — | Static options (Java/Bedrock/PocketMine), no fetch needed |
| Plan card display | Browser (Client) | API (Backend) | Display comes from client-side plan definitions; real plan data (pricing, features) could optionally come from `/billing/plans` API |
| Version list | Browser (Client) | — | MINECRAFT_VERSIONS array is hardcoded in CreateServerModal.jsx — reuse same client-side data |
| Template list | API (Backend) | Browser (Client) | Fetched via `api.templates()` — but fetched once and cached client-side during wizard session |
| Server creation | API (Backend) | — | `POST /api/v1/servers` via `serversApi.create()` — pure backend operation |
| Redirect to server detail | Browser (Client) | — | Client-side routing via `useNavigate()` after successful creation |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| React | 19.2.4 | UI framework | Project standard — all pages use React |
| react-router-dom | 7.13.0 | Client-side routing | Project standard — `useNavigate` for redirects |
| Zustand | 5.0.12 | State management | Already used for 3 stores (authStore, serverStore, uiStore) — NOT for wizard state |
| Tailwind CSS | 4.2.0 | Styling | Project standard utility classes |
| lucide-react | 1.18.0 | Icons | Project standard — used in 7+ components |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `useState` (React) | ^19.2.4 | Step management | Primary state management for wizard — 4 steps, no persistence needed |
| `useRef` (React) | ^19.2.4 | Wizard data persistence | Store selections across steps without re-rendering all consumers |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Component-local `useState` | Zustand wizard store | Overkill — wizard state is ephemeral, only lives during modal lifespan |
| Custom wizard | `react-stepper-horizontal` or similar | Minimal benefit for 4 steps; bundle size increase; may not match cosmic theme |
| Single component file | Split into `WizardStep1.jsx`, etc. | Over-fragmentation — 4 simple steps fit easily in one file (< 500 lines) |

**Installation:**
No new packages needed. All dependencies are already in the project.

**Version verification:** All versions confirmed via `app/package.json` — no new installations required.

## Architecture Patterns

### System Architecture Diagram

```
DashboardPage.jsx                    App.jsx (ProtectedRoute wrapper)
│                                        │
├── (totalServers === 0)                 │
│   └── "Create your first server"       │
│        button (line 237)               │
│            │                           │
│            ▼                           │
│        [onClick handler]               │
│            │                           │
│       ┌────┴─────────────────┐         │
│       │  setShowWizard(true)  │         │
│       └────┬─────────────────┘         │
│            │                           │
│            ▼                           │
│    ┌──────────────────────┐            │
│    │ ServerOnboardingWizard│           │
│    │  ───────────────────  │           │
│    │  Step 1: Type         │  ──► API: api.templates()
│    │  Step 2: Resources    │  ──► API: billingApi.getCurrentSubscription()
│    │  Step 3: Config       │
│    │  Step 4: Deploy       │  ──► API: serversApi.create(data)
│    └──────────┬───────────┘            │
│               │                        │
│               ▼                        │
│         [On success]                   │
│               │                        │
│               ▼                        │
│         navigate(`/servers/${id}`)     │
└────────────────────────────────────────┘

ServerManagerPage.jsx
├── "Create your first server" button
│   └── Opens CreateServerModal (UNCHANGED)
└── No change — existing route-based access preserved
```

### Recommended Project Structure
```
app/src/
├── features/
│   └── server/
│       ├── CreateServerModal.jsx       # [UNCHANGED] 815-line existing modal
│       └── ServerOnboardingWizard.jsx   # [NEW] Multi-step wizard (~400-500 lines)
├── pages/
│   └── dashboard/
│       └── DashboardPage.jsx            # [MINIMAL CHANGE] Line 237 only
```

### Pattern 1: Multi-Step Wizard with `useState`
**What:** Component-internal step management using a `currentStep` state variable (0-3), with a `wizardData` ref to persist selections across steps. Each step renders conditionally via switch/match.

**When to use:** Any multi-step form too simple to justify a wizard library (≤ 6 steps, no branching paths, no async step loading).

**Source:** Established React pattern — no library dependency needed.

### Pattern 2: Modal Overlay with glass-panel
**What:** Full-screen backdrop with centered modal container using `glass-panel` class and cosmic CSS variables.

**Source:** [VERIFIED: CreateServerModal.jsx line 393-394] — `fixed inset-0 bg-black/50 flex items-center justify-center z-50`, inner container uses `bg-[var(--color-bg-secondary)]` with `border border-[var(--color-border)] rounded-lg`.

**Example structure:**
```jsx
<div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
  <div className="glass-panel p-8 w-full max-w-lg max-h-[90vh] overflow-y-auto
                  bg-[var(--color-bg-secondary)] border border-[var(--color-border)] rounded-lg">
    {/* Progress bar */}
    {/* Step title */}
    {/* Step content */}
    {/* Navigation buttons (Back / Next / Deploy) */}
  </div>
</div>
```

### Pattern 3: Server Creation via API
**What:** Call `serversApi.create(data)` with the server configuration object. On success, `addServer(server)` to store, refresh server list, and redirect.

**Source:** [VERIFIED: CreateServerModal.jsx lines 296-328] — The exact payload shape:
```js
const serverData = {
  name: name.trim(),
  game_type: gameType,
  minecraft_version: mcVersion,
  ram_mb: parseInt(ram) * 1024,
  max_ram_mb: parseInt(maxRam) * 1024,
  max_players: parseInt(maxPlayers),
  port: parseInt(port),
  online_mode: onlineMode === 'true',
  world_seed: worldSeed || undefined,
  difficulty,
  op: op || undefined,
  server_type: serverType,
  jvm_opts: jvmOpts || undefined,
  image: selectedTemplate?.config?.docker_image || undefined,
  template_id: selectedTemplate?.id || undefined,
  node_id: nodeId || undefined,
  modpack_template_id: selectedModpack?.id || null,
}
const server = await serversApi.create(serverData)
```

### Anti-Patterns to Avoid
- **Zustand store for wizard state:** The wizard lives only as long as the modal is open. A Zustand store persists across component mounts and would leave stale state in memory. Use component-local state.
- **Duplicating MINECRAFT_VERSIONS:** The array in CreateServerModal.jsx lines 7-82 is the canonical source. Import it or co-locate it in a shared constants file rather than duplicating.
- **Server creation logic duplication:** The `handleSubmit` function in CreateServerModal.jsx (lines 274-335) contains the full API call, error handling, store updates, and toast notifications. Import and reuse `serversApi.create()` — do not copy the entire handler.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Stepper/wizard UI | Custom step progress indicator | Simple CSS dots/bar with Tailwind | 4 steps is too trivial to justify a library. Existing Onboarding.jsx already has a dot progress pattern (6 dots → 4 steps). |
| Form validation framework | Custom validation | Native HTML5 validation + simple `validateStep()` functions | Only 3 fields in Step 3 need validation (name required, port range). No complex cross-field validation. |
| Icons for game types | Hand-crafted SVG icons | lucide-react icons library | Project standard. Use Cpu (Java), Shield (Bedrock), Smartphone (PocketMine) from lucide-react. |
| Animation library | Fade/slide transitions | CSS transitions only | Simple step transitions; no need for Framer Motion. The project's `400ms ease` transition on all elements already handles state changes smoothly. |

**Key insight:** The wizard is an orchestration component (4 steps → 1 API call). The hard parts are already solved in `CreateServerModal.jsx` (API calls, version data, template fetching). This phase is primarily about step UI layout and data flow orchestration.

## Common Pitfalls

### Pitfall 1: Step State Loss on "Back" Navigation
**What goes wrong:** User fills Step 3 (name, version, port), clicks Back to change plan in Step 2, then clicks Next to Step 3 — all their Step 3 inputs are cleared.
**Why it happens:** Each step renders fresh when `currentStep` changes if step data isn't persisted in refs or lifted to parent state.
**How to avoid:** Store all wizard selections in a `useRef({})` object that persists across step renders. Only `currentStep` should be `useState`. Alternatively, keep all form values as parent-level `useState` and pass setters down.
**Warning signs:** Input values flash to defaults on step navigation.

### Pitfall 2: Version Dropdown Not Filtered by Game Type
**What goes wrong:** User selects PocketMine in Step 1, but Step 3's version dropdown still shows Java versions.
**Why it happens:** The version list is a static array. If game type ≠ Java, the version dropdown should either hide completely (PocketMine/Bedrock don't have version pickers in the current modal) or show a different set.
**How to avoid:** In CreateServerModal.jsx, the bedrock section (line 663) has no version dropdown and no RAM/max_ram fields — only max_players, online_mode, game_mode, difficulty, port. For PocketMine, likewise no version dropdown. The wizard Step 3 should conditionally render version dropdown only for Java Edition, and show relevant fields per game type.
**Warning signs:** Version dropdown visible for PocketMine or Bedrock selections.

### Pitfall 3: API Error Leaves Wizard in Half-Created State
**What goes wrong:** User clicks "Deploy Server", API returns an error (e.g., port conflict, server limit reached). Toast fires, but the wizard doesn't clean up — next deploy attempt may fail again without resolution.
**How to avoid:** Keep the Deploy button enabled after error so user can retry (don't close wizard on error). If the error is plan-related (server limit), show an inline upgrade CTA within the wizard rather than just a toast.
**Warning signs:** Wizard closes on error without giving user a chance to fix the issue.

### Pitfall 4: Random Port Generation on First Render
**What goes wrong:** Every time Step 3 renders (including on Back→Next navigation), the port regenerates a random value, overwriting user's previous input.
**How to avoid:** Generate the port suggestion only once when data ref initializes in Step 3, not on every render. Use `useEffect` with empty deps or check if port is already set.
**Warning signs:** Port value changes when navigating between steps.

### Pitfall 5: Resource Plan Limits Not Enforced
**What goes wrong:** User selects Free plan (1 server limit) in Step 2, but they already have a server. The API call fails with server limit error in Step 4.
**How to avoid:** Fetch the user's current subscription + existing server count in Step 2 (during resource plan display) and either (a) hide plans that exceed limits with a "locked" indicator, or (b) validate before allowing Next from Step 2. The `checkServerLimit(plan)` method exists in `useServerStore`.
**Warning signs:** User selects a plan and proceeds, only to hit an error at deploy time.

## Code Examples

### Step 1: Wizard Skeleton — Step Management Pattern
**Source:** [VERIFIED: Onboarding.jsx + standard React pattern]

```jsx
import { useState, useRef } from 'react'
import { useNavigate } from 'react-router-dom'
import { serversApi } from '../../lib/api'
import { useServerStore } from '../../store/serverStore'
import { useUIStore } from '../../store/uiStore'

export default function ServerOnboardingWizard({ isOpen, onClose }) {
  const [currentStep, setCurrentStep] = useState(0)
  const wizardData = useRef({
    gameType: 'minecraft',
    plan: null,
    name: '',
    mcVersion: '26.2',
    port: '',
    template: null,
  })

  const canProceed = () => {
    switch (currentStep) {
      case 0: return !!wizardData.current.gameType     // Type selected
      case 1: return !!wizardData.current.plan          // Plan selected
      case 2: return wizardData.current.name?.trim()    // Name provided
      case 3: return true                                // Review — always show deploy
      default: return false
    }
  }

  const handleNext = () => {
    if (currentStep < 3) setCurrentStep(s => s + 1)
  }

  const handleBack = () => {
    if (currentStep > 0) setCurrentStep(s => s - 1)
  }

  if (!isOpen) return null

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="glass-panel p-8 w-full max-w-lg max-h-[90vh] overflow-y-auto
                      bg-[var(--color-bg-secondary)] border border-[var(--color-border)] rounded-lg">
        {/* Progress bar */}
        <div className="h-2 bg-[var(--color-cosmic-border)] rounded-full mb-6">
          <div className="h-full bg-[var(--color-cosmic-cyan)] rounded-full transition-all duration-300"
               style={{ width: `${((currentStep + 1) / 4) * 100}%` }} />
        </div>

        {/* Step content */}
        {currentStep === 0 && <Step1Type wizardData={wizardData} />}
        {currentStep === 1 && <Step2Resources wizardData={wizardData} />}
        {currentStep === 2 && <Step3Config wizardData={wizardData} />}
        {currentStep === 3 && <Step4Deploy wizardData={wizardData} />}

        {/* Navigation */}
        <div className="flex justify-between mt-8">
          <div>
            {currentStep > 0 && (
              <button onClick={handleBack} className="px-4 py-2 text-[var(--color-text-secondary)]
                hover:text-[var(--color-text-primary)]">
                Back
              </button>
            )}
          </div>
          <div className="flex gap-3">
            <button onClick={onClose} className="px-4 py-2 text-[var(--color-text-secondary)]
              hover:text-[var(--color-text-primary)]">
              Skip
            </button>
            {currentStep < 3 ? (
              <button onClick={handleNext} disabled={!canProceed()}
                className="px-6 py-2 bg-[var(--color-cosmic-cyan)] text-white rounded-lg
                  hover:brightness-110 disabled:opacity-50">
                Next
              </button>
            ) : (
              <DeployButton wizardData={wizardData} onClose={onClose} />
            )}
          </div>
        </div>
      </div>
    </div>
  )
}
```

### Step 2: Server Creation API Call — Reuse Pattern
**Source:** [VERIFIED: CreateServerModal.jsx lines 296-328]

```jsx
async function handleDeploy(wizardData, onClose) {
  const data = wizardData.current
  setIsDeploying(true)
  try {
    const serverData = {
      name: data.name.trim(),
      game_type: data.gameType,
      minecraft_version: data.mcVersion,
      ram_mb: data.ram ? parseInt(data.ram) * 1024 : 4096,
      port: parseInt(data.port),
      server_type: data.serverType,
      template_id: data.template?.id || undefined,
      node_id: undefined, // auto-select
    }
    const server = await serversApi.create(serverData)
    addServer(server)
    addToast({ type: 'success', message: 'Server created successfully!' })
    onClose()
    navigate(`/servers/${server.id}`)
  } catch (err) {
    addToast({ type: 'error', message: err.message || 'Failed to create server' })
  } finally {
    setIsDeploying(false)
  }
}
```

### Step 3: Progress Dots Pattern (Existing)
**Source:** [VERIFIED: Onboarding.jsx lines 58-67]

```jsx
<div className="flex justify-center gap-2 mb-8">
  {[0, 1, 2, 3].map((i) => (
    <div key={i} className={`w-3 h-3 rounded-full transition-colors ${
      i === currentStep ? 'bg-[var(--color-cosmic-cyan)]' : 'bg-[var(--color-border)]'
    }`} />
  ))}
</div>
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `navigate('/servers')` on dashboard empty state | Open `ServerOnboardingWizard` modal | This phase | New users get guided server creation flow instead of redirecting to /servers |
| Single-form `CreateServerModal` (815 lines) | Preserved for experienced users | This phase | Dual entry points: wizard (dashboard empty state) + modal (/servers page) |
| Legacy `Onboarding.jsx` (86-line placeholder) | Unchanged — still renders in App.jsx | This phase | Coexistence — wizard replaces its trigger behavior, not the component itself |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | No wizard/stepper library is needed because 4 steps is simple enough | Standard Stack | If future phases add steps 5+, migrating to a library would require refactoring |
| A2 | The wizard should use component-local `useState` + `useRef` rather than Zustand | Architecture Patterns | If wizard state needs to persist across page navigations (e.g., user goes to another tab mid-wizard), ref-based state would be lost |
| A3 | Step 2 plan cards can show plan features from client-side definitions rather than API data | Don't Hand-Roll | If plan definitions change frequently, hardcoded features would become stale |

## Open Questions (RESOLVED)

1. **Plan data source for Step 2?** — RESOLVED: Hardcode plan definitions client-side as a `PLANS` constant in the wizard component (Free = 1 server, basic; Hobby = 5 servers, auto-backups; Pro = 20 servers, all features), matching known `serverStore.js` limits. Avoids a loading step. A future TODO can fetch from API.

2. **How to handle "CreateServerModal" fields that the wizard simplifies?** — RESOLVED: Expose only CONTEXT.md-specified fields (name, version, port, template). All other fields (online_mode, difficulty, server_type, jvm_opts, etc.) get sensible defaults matching CreateServerModal.jsx defaults. No version picker in the wizard.

3. **Should PocketMine-MP and Nukkit be separate options in Step 1?** — RESOLVED: Follow CONTEXT.md explicitly — 3 cards in Step 1 (Java, Bedrock, PocketMine). PocketMine-MP vs Nukkit variant selection deferred to Step 3 variant dropdown (same pattern as CreateServerModal's template/variant).

## Environment Availability

> Step 2.6: SKIPPED (no external dependencies — purely frontend code changes with no new tooling or services required)

## Validation Architecture

> Skipped — `workflow.nyquist_validation` is not explicitly set to `true` in `.planning/config.json`. No test infrastructure exists in the project (no test/config files found in glob scan, no test dependencies in package.json).

## Security Domain

> Skipped — no new backend API endpoints, no authentication changes, no data storage changes. Security scope limited to input validation on Step 3 fields (name max length, port validity range), which is already handled by the same validation in CreateServerModal.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | yes | Port within 10000-30000 range validation (same as CreateServerModal.jsx `validatePort()`), name max length |
| V2 Authentication | no | Auth handled by existing ProtectedRoute wrapper |
| V3 Session Management | no | No session changes |
| V4 Access Control | no | No access control changes — same API endpoint |

## Sources

### Primary (HIGH confidence)
- [VERIFIED: codebase] — `app/src/pages/dashboard/DashboardPage.jsx` — Empty state button at line 237
- [VERIFIED: codebase] — `app/src/features/server/CreateServerModal.jsx` — Existing 815-line modal, MINECRAFT_VERSIONS, serversApi.create()
- [VERIFIED: codebase] — `app/src/pages/servers/ServerManagerPage.jsx` — Import/create modal pattern
- [VERIFIED: codebase] — `app/src/components/Onboarding.jsx` — Existing onboarding, dot progress pattern
- [VERIFIED: codebase] — `app/src/pages/dashboard/WelcomeModal.jsx` — Modal overlay pattern
- [VERIFIED: codebase] — `app/src/app/App.jsx` — Routes, layout, isOnboarded rendering
- [VERIFIED: codebase] — `app/src/store/serverStore.js` — checkServerLimit(), addServer()
- [VERIFIED: codebase] — `app/src/store/uiStore.js` — setOnboarded(), addToast(), isOnboarded
- [VERIFIED: codebase] — `app/src/lib/api.js` — serversApi.create(), api.templates()
- [VERIFIED: codebase] — `app/src/index.css` — CSS variables, glass-panel, theme tokens
- [VERIFIED: codebase] — `app/package.json` — All dependency versions

### Secondary (MEDIUM confidence)
- [VERIFIED: UI-SPEC] — `.planning/phases/83/83-UI-SPEC.md` — Design contract for visual decisions
- [VERIFIED: CONTEXT.md] — `.planning/phases/83/83-CONTEXT.md` — All locked decisions

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — All dependency versions verified from package.json, no new libraries needed
- Architecture: HIGH — Patterns verified from existing codebase (modal pattern, step pattern, API call pattern)
- Pitfalls: HIGH — All identified pitfalls are concrete behaviors observed in the existing CreateServerModal.jsx or common wizard UX issues

**Research date:** 2026-06-16
**Valid until:** 2026-07-16 (stable — no fast-moving dependencies)
