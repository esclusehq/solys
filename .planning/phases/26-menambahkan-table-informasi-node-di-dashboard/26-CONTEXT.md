# Phase 26: Menambahkan table informasi node di dashboard - Context

**Gathered:** 2026-04-20

<domain>
## What We're Building

Add a node information table to the dashboard (below servers table or as separate section).

Currently dashboard has:
- 3 cards: Servers, Billing, Agents (just count)
- Servers table

Missing: Node table like servers table

</domain>

<decisions>
## Implementation

**Chosen:** Basic columns
- Name
- Status (online/offline)
- IP Address
- Uptime (how long online)
- Servers count (how many servers on this node)

**Table location:** Below servers table in dashboard
**Empty state:** "No nodes found" message

## Current API

Nodes endpoint: GET /api/v1/nodes returns:
- id, name, ip_address, port, status, first_seen, last_seen

Need to add:
- Uptime calculation (last_seen - first_seen)
- Server count per node (filter servers by node_id)

</decisions>

<canonical_refs>
## References

- DashboardPage.jsx — existing dashboard
- Servers table pattern in DashboardPage.jsx (lines 133-180)
- useNodes.js — nodes hook
- /api/v1/nodes — node list API

</canonical_refs>

<specifics>
## Implementation

1. Add useNodes hook to DashboardPage
2. Create node table below servers table
3. Columns: Name, Status, IP, Uptime, Servers

</specifics>

<deferred>
## Deferred

- Node detail actions (future)
- Node health metrics in table

</deferred>

---

## ▶ Next Up

`/clear` then:

/gsd-plan-phase 26 ${GSD_WS} — create plan from this context