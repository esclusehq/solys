# Phase 76: update UI https://app.esluce.com/nodes - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.

**Date:** 2026-06-14
**Phase:** 76-update-ui-https-app-esluce-com-nodes
**Areas discussed:** Layout & node list, Node card content, Search & filtering, Detail panel enhancements

---

## Layout & Node List

| Option | Description | Selected |
|--------|-------------|----------|
| Keep split panel | Left: node list, right: detail on select | |
| Full-page list with detail page | Node list fills page, clicking navigates to separate detail | |
| Keep split, add toggle | Keep split panel but add table/card toggle like Phase 75 | ✓ |

**User's choice:** Keep split, add toggle (with localStorage persistence)

---

## Node Card Content

| Option | Description | Selected |
|--------|-------------|----------|
| Keep current | Name, IP, memory, CPU, status | |
| Add more info | Add agent version, OS, container runtime, uptime, last seen | |
| Minimal | Just name, IP, and status dot | |

**User's choice:** Add more info — specifically **uptime** and **last seen** (custom response)

---

## Search & Filtering

| Option | Description | Selected |
|--------|-------------|----------|
| Search by name/IP | Add a search bar filtering by name or IP | |
| Search + status filter | Search bar + filter dropdown for online/warning/offline | |
| No search/filter | Keep as-is. Node count is typically low | ✓ |

**User's choice:** No search/filter

---

## Detail Panel

| Option | Description | Selected |
|--------|-------------|----------|
| Keep as-is | 3 tabs are good | ✓ |
| Add Servers tab | Show servers running on this node | |
| Add Logs tab | Add a tab for node activity/event logs | |

**User's choice:** Keep as-is

| Option | Description | Selected |
|--------|-------------|----------|
| Keep as-is | 4 metrics grid is clear | |
| Add disk usage | Add disk storage to health metrics | |
| Visual refresh | Keep same data but improve visual presentation | ✓ |

**User's choice:** Visual refresh for health metrics (bars, charts)

---

## the agent's Discretion

- Table view column layout
- Visual style of health metrics refresh
- Exact placement of uptime/last seen in cards

## Deferred Ideas

None.
