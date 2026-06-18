---
phase: 88-update-ui-ux-console-header-and-terminal-layout
plan: 02
subsystem: Console
tags: [console, sidebar, header, server-selection]
key-files:
  - app/src/components/ServerSidebar.jsx
  - app/src/components/ConsoleHeader.jsx
  - app/src/pages/Console.jsx
metrics:
  files_created: 2
  files_modified: 1
  build: pass
---

## Commits

| Task | Description | Hash |
|------|-------------|------|
| 1 | Create ServerSidebar component with collapsible server list, status dots, toggle button | TBD |
| 2 | Create ConsoleHeader component with breadcrumb link + server name title | TBD |
| 3 | Refactor Console.jsx: replace dropdown with sidebar, use ConsoleHeader, remove status indicator | TBD |

## Deviations

None — all tasks implemented as specified in the plan.

## Self-Check

**Result: PASSED**

- [x] Build passes (`npm run build` exit 0)
- [x] Server dropdown removed from Console page — replaced by collapsible sidebar
- [x] ServerSidebar shows servers with name + colored status dot (running=green, stopped=red, other=orange)
- [x] ServerSidebar defaults to collapsed — toggle button with PanelLeft/X icons
- [x] Sidebar animation: 300ms ease width transition 0 ↔ 224px
- [x] ConsoleHeader shows breadcrumb link + active server name (or "Console" if no server selected)
- [x] Connection status indicator removed from ConsoleHeader (moved to terminal title bar in Plan 01)
- [x] Server name visible in header when sidebar is collapsed
- [x] Toggle button positioned at sidebar edge, always accessible
- [x] Console page maintains: URL param pre-selection, server switching, Terminal rendering
