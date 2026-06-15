# Phase 83: buat onboarding untuk mempermudah user membuat server yang di inginkan ketika menekan 'Create your first server' di dashboard utama - Context

**Gathered:** 2026-06-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Create a multi-step onboarding wizard that guides first-time users through server creation when they click "Create your first server" on the dashboard empty state. The wizard replaces the current `navigate('/servers')` behavior for new users, while the existing CreateServerModal remains accessible from `/servers` for experienced users. No changes to backend API — reuses existing create server logic.

</domain>

<decisions>
## Implementation Decisions

### Trigger & Wizard Format
- **D-01:** Clicking "Create your first server" on dashboard empty state opens a new ServerOnboardingWizard modal (not navigate to /servers)
- **D-02:** Existing CreateServerModal stays unchanged for experienced users accessing from /servers page
- **D-03:** Wizard is a multi-step modal with progress bar — not side panel or tooltip overlay

### Wizard Steps (4 steps)
- **D-04:** Step 1 (Type): 3 large cards — Java Edition, Bedrock, PocketMine — with icons, descriptions, and recommendations. User clicks one to select.
- **D-05:** Step 2 (Resources): Show available plan cards (Free/Hobby/Pro) with feature lists. User picks a plan, then can configure RAM/CPU/Disk within the plan's limits.
- **D-06:** Step 3 (Config): Full server configuration — name, Minecraft version (auto-select latest), port (with random suggest), template (optional).
- **D-07:** Step 4 (Deploy): Review summary of all choices + "Deploy Server" button. After successful creation, auto-redirect to the new server's detail page.

### Component Architecture
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

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Target Files
- `app/src/pages/dashboard/DashboardPage.jsx` — Empty state with "Create your first server" button (line 237), currently calls `navigate('/servers')`. Need to replace with opening ServerOnboardingWizard.
- `app/src/features/server/CreateServerModal.jsx` — Existing 815-line create server modal. Contains MINECRAFT_VERSIONS array, game type selection, node selection, server creation API call. Reference for API logic to reuse.
- `app/src/pages/servers/ServerManagerPage.jsx` — Imports and renders CreateServerModal. Reference for how the modal integrates.

### Design References
- `app/src/index.css` — Cosmic theme CSS variables (glass-panel, cosmic borders, cyan accent, etc.)
- `app/src/components/Onboarding.jsx` — (check if exists) existing onboarding component for reference

### API (no new endpoints)
- Uses existing `serversApi.create()` — same API call as CreateServerModal

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `CreateServerModal.jsx` — Contains MINECRAFT_VERSIONS array, game type selection state, server creation API call, template fetching, modpack fetching. The API call logic can be extracted/referenced.
- `glass-panel` utility class — for wizard modal container
- Cosmic CSS variables — for consistent styling with existing theme

### Established Patterns
- Modal pattern: Zustand-based open/close state, glass-panel container, backdrop blur
- Page routing: react-router-dom `useNavigate`
- Server creation: `serversApi.create(data)` returns created server object, redirect to server detail
- Styling: cosmic theme with glass-panel, cyan accent buttons, border-[var(--color-cosmic-border)]

### Integration Points
- `DashboardPage.jsx` line 236-241 — Replace `onClick={() => navigate('/servers')}` with opening ServerOnboardingWizard
- Same page state management — need `showWizard` state variable and conditional render
- No changes needed to router or existing CreateServerModal

</code_context>

<specifics>
## Specific Ideas

- Step 1 cards should clearly distinguish Java vs Bedrock vs PocketMine — each has different version ranges and capabilities
- Version dropdown should auto-filter by selected game type
- For first-time users, default to recommended options as much as possible (latest version, random port, no template)
- Plan cards should show features prominently (not just specs) — e.g., "Auto-backups", "Plugins support", "Custom domain"

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 83-buat-onboarding-untuk-mempermudah-user-membuat-server-yang-d*
*Context gathered: 2026-06-16*
