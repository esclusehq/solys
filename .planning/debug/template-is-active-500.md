---
status: root_cause_found
trigger: "500 Internal Server Error saat PUT /api/v1/templates/:id dengan is_active: false — template Rust"
created: 2026-06-16T16:45:00.000Z
updated: 2026-06-16T16:55:00.000Z
---

# Debug: template-is-active-500

## Symptoms

1. **Expected behavior:** Template dapat di-update dengan field `is_active: false` (uncheck Active checkbox)
2. **Actual behavior:** 500 Internal Server Error
3. **Error messages:** `{"success":false,"data":null,"error":{"code":"INTERNAL_ERROR","message":"Internal Server Error"}}`
4. **Timeline:** Baru bermasalah — dulu bisa
5. **Reproduction:** Buka Edit Template Rust → uncheck Active checkbox (set `is_active` ke false) → klik Update Template

## Current Focus

- **Hypothesis:** RETURNING clause pada query `update_template` tidak menyertakan kolom `version` yang baru ditambahkan — menyebabkan sqlx gagal mapping ke struct `Template`
- **Test:** Trace alur `is_active` dari DTO → entity → repository → query SQL — semua benar
- **Root cause:** Kolom `version` ditambahkan ke table & struct `Template` tetapi `update_template` RETURNING clause tidak diupdate → sqlx error
- **Next action:** Memperbaiki RETURNING clause di `update_template` dan `create_template`

## Evidence

1. **DTO (`UpdateTemplateRequest`)** ✅ Field `is_active: Option<bool>` (line 21 di `template_dtos.rs`)
2. **Use Case** ✅ `is_active: req.is_active.unwrap_or(existing.is_active)` (line 132 di `template_use_cases.rs`)
3. **Repository SQL** ✅ `SET` clause menyertakan `is_active = $6` (line 190 di `repository.rs`)
4. **Binding** ✅ `template.is_active` di-bind ke $6 (line 202 di `repository.rs`)
5. **🔥 Return clause missing `version`** — RETURNING tidak menyertakan `version` padahal struct Template punya `version: Option<String>` tanpa `#[sqlx(default)]`

## Eliminated

- ❌ Frontend payload — `is_active: false` dikirim dengan benar dari `TemplateCreatePage.jsx` line 77
- ❌ `UpdateTemplateRequest` DTO — semua field `Option`, `is_active: Option<bool>` proper
- ❌ Validasi / ownership — use case tidak reject request
- ❌ Constraint database — `is_active BOOLEAN NOT NULL DEFAULT true`, tidak ada CHECK constraint

## Specialist Hint: rust

## Resolution

- **Root cause:** Query `update_template` di `SqlxTemplateRepository` memiliki RETURNING clause yang tidak menyertakan kolom `version`. Kolom `version` baru ditambahkan ke tabel `templates` (migration `20260616_add_template_version_column.sql`) dan ke struct `Template` (`model.rs`), namun query `update_template` dan `create_template` tidak diupdate. Karena `version: Option<String>` tidak memiliki `#[sqlx(default)]`, sqlx gagal melakukan decoding hasil query ke struct `Template`, menghasilkan 500 Internal Server Error. Issues ini terjadi pada SEMUA operasi update/create template, bukan spesifik `is_active: false` — user baru menyadari saat mengubah `is_active`.
- **Fix:** Tambahkan `version` ke RETURNING clause pada query `update_template` (dan `create_template` untuk antisipasi).
