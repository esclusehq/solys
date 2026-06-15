# Phase 83: Buat Onboarding untuk Mempermudah User Membuat Server — Pattern Map

**Mapped:** 2026-06-16
**Files analyzed:** 2 (1 new, 1 modified)
**Analogs found:** 2 / 2

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `app/src/features/server/ServerOnboardingWizard.jsx` | component | request-response + event-driven | `app/src/features/server/CreateServerModal.jsx` | exact |
| `app/src/pages/dashboard/DashboardPage.jsx` | page | request-response | `app/src/pages/servers/ServerManagerPage.jsx` | role-match |

## Pattern Assignments

### `app/src/features/server/ServerOnboardingWizard.jsx` (component, request-response + event-driven step navigation)

**Analog:** `app/src/features/server/CreateServerModal.jsx` (primary — API logic, game types, modal pattern)
**Analog:** `app/src/components/Onboarding.jsx` (secondary — step navigation pattern, progress dots)

---

#### Imports Pattern

**Source:** `CreateServerModal.jsx` lines 1-5 — standard project import convention:
```jsx
import { useState, useEffect } from 'react'
import { useServerStore } from '../../store/serverStore'
import { useUIStore } from '../../store/uiStore'
import { serversApi, api } from '../../lib/api'
```

**Source:** `Onboarding.jsx` lines 1-3 — step navigation imports:
```jsx
import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useUIStore } from '../store/uiStore'
```

**Recommended imports for ServerOnboardingWizard:**
```jsx
import { useState, useRef, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { useServerStore } from '../../store/serverStore'
import { useUIStore } from '../../store/uiStore'
import { serversApi, api, billingApi } from '../../lib/api'
import { Cpu, Shield, Smartphone, Server, ChevronRight, ChevronLeft, Rocket, CheckCircle2, Sparkles } from 'lucide-react'
```

---

#### Step Navigation Pattern (useState + useRef)

**Source:** `Onboarding.jsx` lines 29-31, 33-40 — simple step counter with local state:
```jsx
export default function Onboarding() {
  const [currentStep, setCurrentStep] = useState(0)
  const { setOnboarded } = useUIStore()
  const navigate = useNavigate()

  const handleNext = () => {
    if (currentStep < steps.length - 1) {
      setCurrentStep(currentStep + 1)
    } else {
      // final step action
    }
  }
```

**Step data persistence pattern (useRef):**
Per RESEARCH.md lines 205-208 — use `useRef` to persist wizard selections across steps:
```jsx
const [currentStep, setCurrentStep] = useState(0)
const wizardData = useRef({
  gameType: 'minecraft',
  plan: null,
  name: '',
  mcVersion: '26.2',
  port: '',
  template: null,
})
```

**canProceed validation pattern:**
```jsx
const canProceed = () => {
  switch (currentStep) {
    case 0: return !!wizardData.current.gameType
    case 1: return !!wizardData.current.plan
    case 2: return wizardData.current.name?.trim()
    case 3: return true
    default: return false
  }
}
```

---

#### Modal Overlay Pattern

**Source:** `CreateServerModal.jsx` lines 390-394 — modal container structure:
```jsx
if (!isOpen) return null

return (
  <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div className="bg-[var(--color-bg-secondary)] rounded-lg p-6 w-full max-w-lg max-h-[90vh] overflow-y-auto">
```

**Source:** `WelcomeModal.jsx` lines 55-63 — more polished modal with backdrop click-close:
```jsx
<div
  className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 p-4"
  onClick={handleClose}
  role="dialog"
  aria-modal="true"
>
  <div
    className="bg-[var(--color-bg-secondary)] border border-[var(--color-border)] rounded-lg p-8 max-w-md w-full space-y-6"
    onClick={(e) => e.stopPropagation()}
  >
```

**Use the WelcomeModal pattern** — it's the more recent and polished modal pattern with backdrop click-close and proper aria attributes. The wizard needs `max-w-lg` (larger) due to 4-step content.

---

#### Progress Bar / Progress Dots Pattern

**Source:** `Onboarding.jsx` lines 58-67 — dot indicators:
```jsx
<div className="flex justify-center gap-2 mb-8">
  {steps.map((_, i) => (
    <div
      key={i}
      className={`w-3 h-3 rounded-full transition-colors ${
        i === currentStep ? 'bg-[var(--color-cosmic-cyan)]' : 'bg-[var(--color-border)]'
      }`}
    />
  ))}
</div>
```

**For the wizard, use a progress bar** (per D-03 multi-step modal with progress bar). Pattern from RESEARCH.md lines 280-283:
```jsx
<div className="h-2 bg-[var(--color-cosmic-border)] rounded-full mb-6">
  <div className="h-full bg-[var(--color-cosmic-cyan)] rounded-full transition-all duration-300"
       style={{ width: `${((currentStep + 1) / 4) * 100}%` }} />
</div>
```

---

#### Step Content Rendering Pattern

Per RESEARCH.md lines 286-289:
```jsx
{currentStep === 0 && <Step1Type wizardData={wizardData} />}
{currentStep === 1 && <Step2Resources wizardData={wizardData} />}
{currentStep === 2 && <Step3Config wizardData={wizardData} />}
{currentStep === 3 && <Step4Deploy wizardData={wizardData} />}
```

For a single-file approach (recommended — < 500 lines), define steps as inner sub-components or render inline with conditionals.

---

#### Navigation Buttons Pattern

Per RESEARCH.md lines 292-316:
```jsx
<div className="flex justify-between mt-8">
  <div>
    {currentStep > 0 && (
      <button onClick={handleBack}
        className="px-4 py-2 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]">
        Back
      </button>
    )}
  </div>
  <div className="flex gap-3">
    <button onClick={onClose}
      className="px-4 py-2 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]">
      Skip
    </button>
    {currentStep < 3 ? (
      <button onClick={handleNext} disabled={!canProceed()}
        className="px-6 py-2 bg-[var(--color-cosmic-cyan)] text-white rounded-lg hover:brightness-110 disabled:opacity-50">
        Next
      </button>
    ) : (
      <DeployButton wizardData={wizardData} onClose={onClose} />
    )}
  </div>
</div>
```

---

#### Server Creation API Call Pattern

**Source:** `CreateServerModal.jsx` lines 274-335 — the primary pattern to reuse:

**Server limit check** (lines 279-282):
```jsx
if (!checkServerLimit(userPlan)) {
  addToast({ type: 'error', message: 'Server limit reached. Upgrade your plan!' })
  return
}
```

**Input validation** (lines 284-291):
```jsx
if (!name || name.trim() === '') {
  addToast({ type: 'error', message: 'Server name is required' })
  return
}
if (!validatePort(port)) {
  return
}
```

**API call with error handling** (lines 293-334):
```jsx
try {
  const serverData = {
    name: name.trim(),
    game_type: gameType,
    minecraft_version: gameType === 'bedrock' ? undefined : mcVersion,
    ram_mb: gameType === 'bedrock' ? 2048 : parseInt(ram) * 1024,
    port: parseInt(port),
    template_id: selectedTemplate?.id || undefined,
    node_id: nodeId || undefined,
  }
  const server = await serversApi.create(serverData)
  if (server && server.id) {
    addServer(server)
  }
  const serversResponse = await serversApi.list()
  const allServers = serversResponse?.servers || serversResponse || []
  setServers(allServers.filter ? allServers : [])
  addToast({ type: 'success', message: 'Server created successfully!' })
  onClose()
} catch (err) {
  addToast({ type: 'error', message: err.message || 'Failed to create server' })
}
```

**IMPORTANT:** For the wizard, omit `checkServerLimit` check if not applicable (onboarding first server). Replace `onClose()` + `navigate('/servers')` with `navigate('/servers/' + server.id)` to redirect to server detail page (D-07).

---

#### MINECRAFT_VERSIONS Data Pattern

**Source:** `CreateServerModal.jsx` lines 7-82 — hardcoded array with grouped optgroups:
```jsx
const MINECRAFT_VERSIONS = [
  { value: '26.2', label: '26.2 (Chaos Cubed) - Latest', group: '26.x' },
  // ... ~76 entries
]
```

**Reuse strategy (anti-duplication):** Either extract to a shared constants file, or import from CreateServerModal.jsx directly. RESEARCH.md recommends co-locating in a shared constants file to avoid duplication. For the planner: create `app/src/features/server/constants.js` if extract needed, or reference the array directly.

---

#### Game Type Conditional Rendering Pattern

**Source:** `CreateServerModal.jsx` lines 663-773 — bedrock has different fields:
- **Java (lines 502-660):** Name, version, RAM, max_ram, max_players, port, online_mode, world_seed, difficulty, op, server_type, JVM opts
- **Bedrock (lines 663-773):** max_players, online_mode, game_mode, difficulty, allow_cheats, level_name, world_seed, port

For the wizard's Step 3, conditionally show the version dropdown only for Java Edition.

---

#### Port Validation Pattern

**Source:** `CreateServerModal.jsx` lines 359-373:
```jsx
const validatePort = (portValue) => {
  const portNum = parseInt(portValue)
  if (isNaN(portNum) || portNum < 10000 || portNum > 30000) {
    const hint = gameType === 'bedrock' ? 'Bedrock default is 19132' : 'Java default is 25565'
    setPortError(`Port must be between 10000 and 30000 (${hint})`)
    return false
  }
  const usedServer = existingServers.find(s => s && s.port === portNum)
  if (usedServer) {
    setPortError(`Port already in use by server: ${usedServer?.name || 'unknown'}`)
    return false
  }
  setPortError('')
  return true
}
```

---

#### Template Fetching Pattern

**Source:** `CreateServerModal.jsx` lines 184-194:
```jsx
const loadTemplates = async () => {
  try {
    const data = await api.templates()
    const templateList = data.templates || data || []
    setTemplates(templateList)
  } catch (err) {
    console.error('Failed to load templates:', err)
    setTemplates([])
  }
}
```

Use this in Step 3 for template selection (optional). Called once on wizard open via `useEffect` with `isOpen` dependency.

---

#### Billing / Plan Data Loading Pattern

**Source:** `CreateServerModal.jsx` lines 212-231:
```jsx
const loadUserPlan = async () => {
  setPlanLoading(true)
  try {
    const [planResponse, serversResponse] = await Promise.all([
      api.get('/billing/subscription'),
      serversApi.list()
    ])
    const planName = planResponse?.plan?.name || 'free'
    setUserPlan(planName)
    const servers = (serversResponse?.servers || serversResponse || [])
      .filter(s => s && s.id && typeof s === 'object')
    setExistingServers(servers)
  } catch (err) {
    console.error('Failed to load user plan or servers:', err)
    setUserPlan('free')
    setExistingServers([])
  } finally {
    setPlanLoading(false)
  }
}
```

Use this in the wizard to determine available plans/servers for Step 2 (Resources).

---

#### Button Styling Pattern (Project-wide Standard)

**Source:** Multiple files — consistent button styles:
```jsx
// Primary CTA (cyan accent):
<button className="px-6 py-2 bg-[var(--color-cosmic-cyan)] text-white rounded-lg hover:brightness-110 disabled:opacity-50">

// Secondary / ghost:
<button className="px-4 py-2 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]">

// Inline link with icon:
<Link to="/servers" className="glass-panel p-6 border border-[var(--color-cosmic-border)] hover:border-[var(--color-cosmic-cyan)]/30 hover:bg-[rgba(255,255,255,0.06)] transition-all">
```

---

#### Input Styling Pattern (Project-wide Standard)

**Source:** `CreateServerModal.jsx` lines 399-407:
```jsx
<input
  type="text"
  value={name}
  onChange={(e) => setName(e.target.value)}
  placeholder="My Minecraft Server"
  className="w-full px-4 py-2 bg-[var(--color-bg-secondary)] text-[var(--color-text-primary)] rounded focus:outline-none focus:ring-2 focus:ring-[var(--color-cosmic-cyan)]"
  required
/>
```

**Select styling** (lines 411-416):
```jsx
<select
  value={gameType}
  onChange={handleGameTypeChange}
  className="w-full px-4 py-2 bg-[var(--color-bg-secondary)] text-[var(--color-text-primary)] rounded focus:outline-none focus:ring-2 focus:ring-[var(--color-cosmic-cyan)]"
  required
>
```

---

### `app/src/pages/dashboard/DashboardPage.jsx` (page — MODIFY, request-response)

**Change needed:** Line 237 — replace `navigate('/servers')` with opening ServerOnboardingWizard.

**Current code** (lines 233-241):
```jsx
{totalServers === 0 ? (
  /* Empty state */
  <div className="glass-panel p-12 text-center border border-[var(--color-cosmic-border)]">
    <p className="text-[var(--color-text-muted)] text-lg mb-4">No servers yet</p>
    <button
      onClick={() => navigate('/servers')}
      className="px-6 py-3 bg-[var(--color-cosmic-cyan)] text-[var(--color-deep-space)] rounded-lg font-semibold hover:brightness-110 transition-all"
    >
      Create your first server
    </button>
```

**Target pattern:** Same as `ServerManagerPage.jsx` lines 16-17, 184-192 — modal state + conditional render:
```jsx
const [showCreateModal, setShowCreateModal] = useState(false)

// In JSX:
<button
  onClick={() => setShowCreateModal(true)}
  className="bg-[var(--color-cosmic-cyan)] text-[var(--color-deep-space)] px-4 py-2 rounded-lg font-semibold hover:brightness-110"
>
  + Add Server
</button>

<CreateServerModal isOpen={showCreateModal} onClose={() => setShowCreateModal(false)} />
```

**Modified pattern for DashboardPage:**
```jsx
import ServerOnboardingWizard from '../../features/server/ServerOnboardingWizard'

// Inside component:
const [showWizard, setShowWizard] = useState(false)

// Replace line 237 onClick:
onClick={() => setShowWizard(true)}

// Render wizard at bottom of return (after <WelcomeModal />):
<ServerOnboardingWizard isOpen={showWizard} onClose={() => setShowWizard(false)} />
```

---

## Shared Patterns

### Modal Overlay
**Source:** `WelcomeModal.jsx` lines 55-63
**Apply to:** `ServerOnboardingWizard.jsx` (outer wizard container)
```jsx
<div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 p-4"
     onClick={handleClose} role="dialog" aria-modal="true">
  <div className="bg-[var(--color-bg-secondary)] border border-[var(--color-border)] rounded-lg p-8 max-w-md w-full space-y-6"
       onClick={(e) => e.stopPropagation()}>
```

### Server Creation API
**Source:** `CreateServerModal.jsx` lines 296-328
**Apply to:** `ServerOnboardingWizard.jsx` (Step 4 deploy handler)
```jsx
const serverData = {
  name: name.trim(),
  game_type: gameType,
  minecraft_version: gameType === 'bedrock' ? undefined : mcVersion,
  ram_mb: gameType === 'bedrock' ? 2048 : parseInt(ram) * 1024,
  port: parseInt(port),
  template_id: selectedTemplate?.id || undefined,
  node_id: undefined, // auto-select
}
const server = await serversApi.create(serverData)
```

### Toast Notification Pattern
**Source:** `useUIStore` — `addToast()` called in `CreateServerModal.jsx` lines 326, 331
**Apply to:** `ServerOnboardingWizard.jsx` (success/error feedback)
```jsx
const { addToast } = useUIStore()
addToast({ type: 'success', message: 'Server created successfully!' })
addToast({ type: 'error', message: err.message || 'Failed to create server' })
```

### Store Updates After Creation
**Source:** `CreateServerModal.jsx` lines 319-325
**Apply to:** `ServerOnboardingWizard.jsx` deploy flow
```jsx
const { addServer, setServers } = useServerStore()
if (server && server.id) addServer(server)
// Refresh server list
const serversResponse = await serversApi.list()
const allServers = serversResponse?.servers || serversResponse || []
setServers(allServers.filter ? allServers : [])
```

### Redirect After Creation
**Source:** `Onboarding.jsx` line 38 — uses `useNavigate`
**Apply to:** `ServerOnboardingWizard.jsx` (D-07: auto-redirect to server detail)
```jsx
const navigate = useNavigate()
// On success:
navigate(`/servers/${server.id}`)
```

### CSS Variables for Theming
**Source:** `app/src/index.css` lines 3-25
**Apply to:** All new components
```
--color-deep-space: #080b15       (page backgrounds)
--color-nebula: #0d0f1a           (card/section backgrounds)
--color-cosmic-card: rgba(255, 255, 255, 0.03)  (glass-panel bg)
--color-cosmic-border: rgba(255, 255, 255, 0.08) (borders)
--color-cosmic-cyan: #0ddff2      (primary accent, buttons)
--color-text-main: #e2e8f0        (primary text)
--color-text-muted: #64748b        (secondary text)
```

### Button & Input Styling
**Source:** `CreateServerModal.jsx` — consistent form element styling
**Apply to:** All form inputs in wizard steps
- **Inputs/Selects:** `w-full px-4 py-2 bg-[var(--color-bg-secondary)] text-[var(--color-text-primary)] rounded focus:outline-none focus:ring-2 focus:ring-[var(--color-cosmic-cyan)]`
- **Primary button:** `px-6 py-2 bg-[var(--color-cosmic-cyan)] text-white rounded-lg hover:brightness-110 disabled:opacity-50`
- **Ghost button:** `px-4 py-2 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]`

### Step Guard / Validation
**Source:** RESEARCH.md pattern (derived from Onboarding.jsx flow)
**Apply to:** Step navigation between wizard steps
```jsx
const canProceed = () => {
  switch (currentStep) {
    case 0: return !!wizardData.current.gameType
    case 1: return !!wizardData.current.plan
    case 2: return wizardData.current.name?.trim()
    case 3: return true
  }
}
```

### Loading State Pattern
**Source:** `CreateServerModal.jsx` lines 293, 805-808
**Apply to:** Deploy button in Step 4
```jsx
const [isDeploying, setIsDeploying] = useState(false)
// ...
<button disabled={isDeploying} className="...">
  {isDeploying ? 'Deploying...' : 'Deploy Server'}
</button>
```

### Option Card Selection (for Step 1 Type Cards)
**Source:** `WelcomeModal.jsx` lines 90-103 — feature list pattern (icon + text)
**Apply to:** Game type cards in Step 1 and plan cards in Step 2
```jsx
<li className="flex items-start gap-3 text-sm text-[var(--color-text-primary)]">
  <CheckCircle2 className="w-4 h-4 text-[var(--color-cosmic-green)] shrink-0 mt-0.5" />
  <span>{feature}</span>
</li>
```

For cards with selection state, use a pattern like:
```jsx
<div
  onClick={() => selectGameType('minecraft')}
  className={`p-4 rounded-lg border cursor-pointer transition-all ${
    selectedGameType === 'minecraft'
      ? 'border-[var(--color-cosmic-cyan)] bg-[var(--color-cosmic-cyan)]/10'
      : 'border-[var(--color-cosmic-border)] bg-[var(--color-cosmic-card)] hover:border-[var(--color-cosmic-cyan)]/30'
  }`}
>
  <Cpu className="w-8 h-8 text-[var(--color-cosmic-cyan)] mb-2" />
  <h3 className="font-semibold text-[var(--color-text-primary)]">Java Edition</h3>
  <p className="text-sm text-[var(--color-text-secondary)]">Full mod support</p>
</div>
```

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| None | — | — | All files have sufficient analogs in existing codebase |

## Metadata

**Analog search scope:** `app/src/features/server/`, `app/src/pages/dashboard/`, `app/src/pages/servers/`, `app/src/components/`, `app/src/store/`, `app/src/lib/`
**Files scanned:** 9 (CreateServerModal.jsx, DashboardPage.jsx, ServerManagerPage.jsx, Onboarding.jsx, WelcomeModal.jsx, serverStore.js, uiStore.js, api.js, index.css)
**Pattern extraction date:** 2026-06-16
