# GSD Debug Knowledge Base

Resolved debug sessions. Used by `gsd-debugger` to surface known-pattern hypotheses at the start of new investigations.

---

## server-limit — "Server limit reached" error on Starter plan
- **Date:** 2026-04-12
- **Error patterns:** server limit reached, starter plan, 5 servers, CreateServerModal
- **Root cause:** Frontend had two bugs: 1) userPlan was hardcoded as 'starter' instead of fetching from API, 2) limits in serverStore.js were wrong (starter: 2 instead of 5)
- **Fix:** 1) Updated serverStore.js checkServerLimit to use correct limits { starter: 5, pro: 15 }, 2) Added loadUserPlan() to CreateServerModal to fetch user's actual plan from /billing/subscription API
- **Files changed:** app/src/store/serverStore.js, app/src/features/server/CreateServerModal.jsx

## clickable-dashboard-cards — Dashboard stat cards not clickable
- **Date:** 2026-04-21
- **Error patterns:** dashboard cards, clickable, navigation, Link, servers billing nodes
- **Root cause:** Cards were plain div elements without navigation
- **Fix:** Wrap each stat card in Link component from react-router-dom with to prop pointing to /servers, /billing, /nodes respectively
- **Files changed:** app/src/pages/dashboard/DashboardPage.jsx

## game-type-missing — Game column shows "-" instead of game type
- **Date:** 2026-04-21
- **Error patterns:** game_type, servers table, Game column, "-"
- **Root cause:** game_type stored in server.config JSON field, but frontend accessed server.game_type as top-level field
- **Fix:** Extract game_type from server.config.game_type, with fallback to derive from image field (e.g., palworld, valheim, minecraft)
- **Files changed:** app/src/pages/dashboard/DashboardPage.jsx