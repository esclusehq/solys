# Phase 81: update UI di halaman utama/dashboard app.esluce.com - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-15
**Phase:** 81-update-ui-di-halaman-utama-dashboard-app-esluce-com
**Areas discussed:** Summary Cards Design, Cosmic Theme Restyle Scope, Servers & Nodes Tables, Layout & Content Organization, Empty States & Onboarding

---

## Summary Cards Design

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal cosmic restyle | Keep current 3 cards stats, apply glass-panel + cosmic colors | ✓ |
| Enriched with MetricsCard | Use MetricsCard with sparklines and thresholds | |
| Add resource usage cards | Add RAM/CPU/Disk cards from quota API | |

**User's choice:** Minimal cosmic restyle
**Notes:** Keep current 3 cards, no new data or sparklines

---

## Cosmic Theme Restyle Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Cards + tables only | Apply glass-panel to cards and tables only | |
| Full page restyle | Glass-panel on all containers + cosmic accents | |
| Match /servers page exactly | Mirror polish from Phase 75's ServerManagerPage | ✓ |

**User's choice:** Match /servers page exactly
**Notes:** Also: welcome header gets simple cosmic container (no stats row), stars-bg overlay added

---

## Servers & Nodes Tables

| Option | Description | Selected |
|--------|-------------|----------|
| Cosmic restyle only | Apply glass-panel, borders, hover effects | |
| Add table/card toggle | View toggle like /servers page | |
| Enrich with inline features | Add search, filter, sort per table | ✓ |

**User's choice:** Enrich with inline features
**Notes:** Keep current columns, View button only (no quick start/stop)

---

## Layout & Content Organization

| Option | Description | Selected |
|--------|-------------|----------|
| Keep vertical stack | Simple, predictable scrolling | ✓ |
| Add tabs | Tab navigation (Overview / Servers / Nodes) | |
| 2-column grid | Cards + tables side by side | |

**User's choice:** Keep vertical stack
**Notes:** No stats row needed (3 cards already show info), keep current section order

---

## Empty States & Onboarding

| Option | Description | Selected |
|--------|-------------|----------|
| Cosmic restyle only | Glass-panel + cosmic colors on current empty states | |
| Add helpful content | Cosmic restyle + helpful links below CTA | ✓ |
| Illustrated empty states | SVG illustrations per empty state | |

**User's choice:** Add helpful content
**Notes:** Include both docs.esluce.com links AND feature highlights bullet points

---

## the agent's Discretion

- Exact placement of inline search/filter controls within table headers
- Visual styling of empty state helpful links
- Sort direction indicators and default sort column
- Search debounce timing and filter behavior

## Deferred Ideas

- WebSocket-based real-time dashboard updates — future phase
- Resource utilization gauges (RAM, CPU, disk charts) — future phase
- Recent activity / notification feed on dashboard — future phase
- Sidebar.jsx replacing inline App.jsx sidebar — separate refactor
