# Phase 75: update UI https://app.esluce.com/servers - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-14
**Phase:** 75-update-ui-https-app-esluce-com-servers
**Areas discussed:** Card content & layout, Sorting & filtering, Quick actions, Auto-refresh & real-time

---

## Card Content & Layout

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal (current) | Keep current info — name, game type, status, image, node ID | ✓ |
| Detailed | Add address:port, player count, resource usage, uptime | |
| You decide | Let the agent pick | |

**User's choice:** Minimal (current)

| Option | Description | Selected |
|--------|-------------|----------|
| Keep cards as-is | Responsive card grid is fine | |
| Add list/table toggle | Let users switch between card grid and compact table | ✓ |
| Compact cards | Smaller cards, tighter spacing | |

**User's choice:** Add list/table toggle

| Option | Description | Selected |
|--------|-------------|----------|
| Persistent per user | Save preference to localStorage | ✓ |
| Session-only | Default to cards each time | |
| URL-driven | Use query parameter (?view=table|cards) | |

**User's choice:** Persistent per user

---

## Sorting & Filtering

| Option | Description | Selected |
|--------|-------------|----------|
| Name only | Simple alphabetical sort | |
| Name + status + recent | Sort by name, status, or last activity | ✓ |
| Full sorting | Sort by any column | |

**User's choice:** Name + status + recent

| Option | Description | Selected |
|--------|-------------|----------|
| Keep as-is | Status filter dropdown is fine | |
| Add game type filter | Filter by game type (Java, Bedrock, etc.) | ✓ |
| Add node filter | Filter by node | |

**User's choice:** Add game type filter

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, localStorage | Save filter + sort preference | ✓ |
| No, reset on load | Always reset to defaults | |

**User's choice:** Yes, localStorage

---

## Quick Actions

| Option | Description | Selected |
|--------|-------------|----------|
| Add Restart | Add a Restart button | ✓ |
| Console + File Manager | Shortcut to console or file manager | |
| Keep current | Just View + Start/Stop | |

**User's choice:** Add Restart

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, batch actions | Multi-select checkboxes, bulk actions | |
| No batch | Single-server actions only | ✓ |

**User's choice:** No batch

---

## Auto-Refresh & Real-Time

| Option | Description | Selected |
|--------|-------------|----------|
| Poll every 30s | Simple setInterval polling | |
| Real-time WebSocket | Live status push via WS | |
| No auto-refresh | Keep static | |

**User's choice:** Real-time WebSocket (initially), then refined to polling below

| Option | Description | Selected |
|--------|-------------|----------|
| Silent update | Update card in place without animation | |
| Subtle pulse/glimmer | Brief pulse highlight on changed cards | |
| Toast notification | Show toast when server changes status | ✓ |

**User's choice:** Toast notification

| Option | Description | Selected |
|--------|-------------|----------|
| Start with polling | 30s polling for now, WebSocket later | ✓ |
| Full WebSocket now | Implement WS right away | |

**User's choice:** Start with polling (30s setInterval)

---

## the agent's Discretion

- Visual styling of the view toggle (icon/tab/button placement)
- Exact placement of game type filter in the filter bar
- Toast placement and duration
- Table view column details (extend or simplify from legacy)

## Deferred Ideas

- WebSocket-based real-time server status updates
- Batch/multi-select server operations
- More detailed card content (resource usage, player count)
