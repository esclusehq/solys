---
phase: 26-menambahkan-table-informasi-node-di-dashboard
plan: "01"
completed_at: 2026-05-04
---

# Summary: Phase 26 - Menambahkan table informasi node di dashboard

## Completed

Add node information table to dashboard below servers table. Table shows: Name, Status (online/offline), IP Address, Uptime, Server Count.

## Implementation

- Added nodes table below servers table in DashboardPage.jsx
- Used existing `useNodes()` hook
- Columns: Name, Status, IP Address, Uptime, Servers, Actions
- Status badges: green for online, red for offline
- Uptime calculation from `first_seen` timestamp
- Server count per node via filter
- Empty state with "Add your first node" button

## Verification

| Criteria | Status |
|----------|--------|
| Table renders below servers table | ✓ |
| Name column | ✓ |
| Status column with badges | ✓ |
| IP Address column | ✓ |
| Uptime column | ✓ |
| Servers count column | ✓ |
| Empty state handling | ✓ |

## Files Modified

- `app/src/pages/dashboard/DashboardPage.jsx`

---

## ▶ Next Up

Phase 19: user-bisa-add-multiple-nodes-via-dashboard