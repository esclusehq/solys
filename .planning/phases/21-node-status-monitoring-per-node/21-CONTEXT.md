# Phase 21: Node status monitoring per node - Context

**Gathered:** 2026-04-19

<domain>
## What We're Building

Display real-time status of each connected node in the nodes dashboard.

- Users can see node health at a glance
- Works alongside existing node registration feature (Phase 17/19)
- Independent feature - separate from server metrics
</domain>

<decisions>
## Status Metrics (Full)

**Chosen:** Full metrics
- CPU usage %
- RAM usage %
- Disk usage %
- Network I/O (in/out)
- Process count
- Uptime
- Server count on node

**Threshold for status colors:**
- Green (healthy): < 70% any metric
- Yellow (warning): 70-90%
- Red (critical): > 90%

## Display Location

**Chosen:** Status Badge on each row
- Color-coded badge: 🟢 🟡 🔴
- At a glance status in the nodes table
- Click for more details (future enhancement)

## Update Frequency

**Chosen:** 30 seconds
- Balance between responsiveness and load
- Works well for typical node counts (1-10)
</decisions>

<canonical_refs>
## References

- Nodes.jsx — existing nodes table
- useNodes.js — existing node hooks
- api/src/presentation/handlers/node_handlers.rs — node endpoints
- WebSocket for real-time data (reuse existing ws_client from Phase 7)
</canonical_refs>

<specifics>
## Technical Approach

1. Add `/api/v1/nodes/:id/status` endpoint returns node metrics
2. Poll in frontend every 30s using useEffect + setInterval
3. Display as status badge in nodes table
4. Color thresholds applied client-side

**Existing to reuse:**
- Node health checks (Phase 7)
- WebSocket connection (existing in frontend store)
</specifics>

<deferred>
## Deferred Ideas

- Real-time WebSocket push (instead of polling) — future enhancement
- Node detail modal with historical charts — separate phase
</deferred>

---

## ▶ Next Up

**Context ready** — Ready to plan

`/clear` then:

/gsd-plan-phase 21 ${GSD_WS} — create plan from this context