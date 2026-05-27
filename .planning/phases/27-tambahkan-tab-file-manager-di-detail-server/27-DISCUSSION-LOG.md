# Phase 27: File Manager Tab - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-20
**Phase:** 27-file-manager
**Areas discussed:** Layout, Features, Backend, Navigation

---

## Layout

| Option | Description | Selected |
|--------|-------------|----------|
| Single panel | File list only - simple, consistent with existing patterns | |
| Split view | Tree navigator on left, file list/content on right - more powerful but complex | ✓ |

**User's choice:** Split view
**Notes:** FileManager component exists but may need update to support split view

---

## Features

| Option | Description | Selected |
|--------|-------------|----------|
| Browse + Upload/Download | Navigate folders, upload new files, download existing files | |
| Full CRUD | Browse, upload, download, edit, delete, rename - complete file management | ✓ |
| Read-only | Browse and download only - safe but limited | |

**User's choice:** Full CRUD
**Notes:** Complete file management with all operations

---

## Backend

| Option | Description | Selected |
|--------|-------------|----------|
| SFTP API | REST API endpoints for file operations - established pattern in codebase | ✓ |
| WebSocket | Streaming file transfer over WebSocket - real-time but needs new implementation | |

**User's choice:** SFTP API
**Notes:** Uses existing SFTP endpoints in codebase

---

## Navigation

| Option | Description | Selected |
|--------|-------------|----------|
| Tree view | Collapsible folder tree on left - common pattern, easy to understand | ✓ |
| Breadcrumbs | Path shown at top, click to navigate up - simpler but less visual | |

**User's choice:** Tree view
**Notes:** Standard file explorer pattern

---

## Agent's Discretion

Layout is split view with tree on left - this is a specific implementation choice made by user.

## Deferred Ideas

None mentioned during discussion