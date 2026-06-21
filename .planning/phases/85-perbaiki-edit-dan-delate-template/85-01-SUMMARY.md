---
id: 85-01
phase: 85-perbaiki-edit-dan-delate-template
type: execute
status: completed
commits:
  - 3f1d8ad feat(85-01): add version column migration for templates table
  - aaad086 fix(85-01): add version and usage_count to get_template_by_id SELECT
  - cb59413 fix(85-01): differentiate 404 vs 500 in get_template error handling
build: cargo check passed (0 errors)
tests: compilation verified; full test suite timeout-bound
---

## Objective

Fix 404 error when loading official templates for edit/delete by correcting the `get_template_by_id` query and error handling.

## Tasks

### Task 1: Migration for version column ✅
Created `api/migrations/20260616_add_template_version_column.sql`:
```sql
ALTER TABLE templates ADD COLUMN IF NOT EXISTS version VARCHAR(50);
```

### Task 2: Fix SELECT query ✅
Added `version,` and `0 AS usage_count` to `get_template_by_id` query in `api/src/domain/server/template/repository.rs:129-132`.

### Task 3: Fix error handling ✅
Changed `get_template` handler in `api/src/presentation/handlers/template_handlers.rs:59-67` to differentiate:
- `"Template not found"` → 404
- All other errors (SQL, decode) → 500 with tracing log

## Impact
- Official templates can now be fetched by ID without `ColumnNotFound` error
- SQL/encoding errors are surfaced as 500 instead of being masked as 404
- `update_template` and `delete_template` handlers already had differentiated error handling

## Dependencies
None — this plan is standalone (Wave 1).

## Next Steps
Verify the fix end-to-end via 85-02-PLAN.md (requires a running database with seed data).
