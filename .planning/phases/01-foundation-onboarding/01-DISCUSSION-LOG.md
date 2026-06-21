# Phase 1: Foundation & Onboarding - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-18
**Phase:** 1-Foundation & Onboarding
**Areas discussed:** Welcome format, Interest selection UX, Permission model, Server settings, Auto-role timing, Project structure

---

## Welcome Format

| Option | Description | Selected |
|--------|-------------|----------|
| Embed statis | Embed dengan teks, link, dan info server — sederhana | |
| Embed interaktif dengan tombol onboarding | Embed dengan tombol "Mulai Onboarding" | |
| Multi-message sequence | Pesan teks + embed terpisah + DM follow-up | ✓ |
| Saat join, di welcome channel | Langsung saat member join, di channel welcome | ✓ |
| DM member, opsional di channel | DM dulu ke member | |
| Channel + ping member | Di channel dengan ping | |
| Sambutan, link, ajakan pilih interest | Fokus pada hal utama | |
| Lengkap — semua info server | Sambutan + link + interest + rules + info server | ✓ |
| Minimalis | Sambutan singkat + 1-2 link | |
| Indonesia | Bahasa Indonesia | |
| Inggris | Bahasa Inggris | |
| Bilingual | Bisa dikonfigurasi | ✓ |

**User's choice:** Multi-message sequence, di welcome channel saat join, konten lengkap, bilingual
**Notes:** Welcome akan terdiri dari beberapa pesan berurutan, bukan satu embed saja.

---

## Interest Selection UX

| Option | Description | Selected |
|--------|-------------|----------|
| Button row | Setiap interest satu tombol, bisa klik multiple | |
| Select menu dropdown | Pilih dari daftar dropdown, lebih compact | ✓ |
| Modal form | Centang dari daftar di modal | |
| Multiple selection | Pilih beberapa interest | ✓ |
| Single selection | Hanya satu interest | |
| Bersama welcome message | Langsung di welcome message | |
| Trigger by button/command | Tombol "Pilih Interest" atau command | ✓ |
| Via DM | DM ke member setelah join | |
| Admin via Discord command | Admin bisa tambah/hapus/edit dari server | ✓ |
| File konfigurasi | Admin edit file di repo | |
| Hardcoded | Tetap, ubah nanti | |

**User's choice:** Select menu dropdown, multiple selection, trigger by button/command, admin configurable via Discord command
**Notes:** Interest selection tidak otomatis muncul — user harus klik tombol atau kirim command untuk memulai.

---

## Permission Model

| Option | Description | Selected |
|--------|-------------|----------|
| Per-command granular | Setiap command bisa dikonfigurasi per role | ✓ |
| Role hierarchy sederhana | Admin punya akses semua, moderator tertentu | |
| Level/permission level | Numeric level, command butuh minimum level | |
| Discord command saja | Konfigurasi via Discord | |
| Admin panel web | Web panel lebih gampang | |
| Keduanya | Command + panel | ✓ |
| Restrictive — admin only by default | Admin harus explicit mengizinkan role lain | ✓ |
| Permissive — all by default | Semua bisa akses | |
| Database custom | Fleksibel, survive reinstall | ✓ |
| Discord native permissions | Sederhana, langsung jalan | |

**User's choice:** Per-command granular, Discord command + web panel, restrictive default, custom database
**Notes:** Permission system adalah fondasi untuk semua phase berikutnya.

---

## Server Settings

| Option | Description | Selected |
|--------|-------------|----------|
| JSON column | Satu kolom JSON di tabel guild_settings | |
| Tabel terpisah per kategori | Normalized, query efisien | ✓ |
| Discord command dulu | /settings dulu | ✓ |
| Web panel dulu | Web lebih gampang | |
| Keduanya | Dua-duanya dari awal | |
| Minimal dulu | Welcome channel + interest list | |
| Full server management | Welcome channel, message, interest, role-map, log, language, manage channel/server, everything | ✓ |
| Validasi otomatis dengan error embed | Cek channel/role exists, kasih error embed | ✓ |
| Simpan aja, validasi runtime | No validation | |

**User's choice:** Tabel terpisah, Discord command dulu, full server management scope, validasi otomatis dengan error embed
**Notes:** Scope settings mencakup semua aspek manajemen server, bukan hanya onboarding.

---

## Auto-Role Timing

| Option | Description | Selected |
|--------|-------------|----------|
| Setelah pilih interest | Role sesuai interest yang dipilih | |
| Langsung join — default role | Default role dulu, upgrade nanti | |
| Default + interest roles | Keduanya | ✓ |
| Ya — grace period + reminder | 24-48 jam, kalau tidak pilih reminder | |
| Tidak — role tetap | Role tetap, bisa pilih kapan saja | ✓ |
| Skip yang sudah ada | Jangan overwrite manual assignment | |
| Bot always wins | Bot tetap assign sesuai sistem | |
| Append only | Tambah role baru, tidak hapus existing | ✓ |
| Ya, log | Log ke channel logging | ✓ |
| Tidak | Tidak perlu log | |

**User's choice:** Default + interest roles, tidak ada grace period, append only, logging
**Notes:** Default role diberikan saat join, interest role setelah pilih. Tidak ada batas waktu.

---

## Project Structure

| Option | Description | Selected |
|--------|-------------|----------|
| bot/ dan api/ terpisah | Dua direktori terpisah | |
| Single package | Semua dalam satu folder | |
| Workspace monorepo | packages/bot + packages/api | ✓ |
| Folder biasa | Sederhana, nanti upgrade | |
| Turborepo | Orchestrate build, lebih rapi | ✓ |
| Nx | Lebih advance | |
| Masing-masing punya migration sendiri | Bot: Drizzle, API: sqlx | ✓ |
| Single source migration | Satu yang manage semua | |
| Tool terpisah | Migration tool dedicated | |
| Masing-masing .env | bot/.env, api/.env | ✓ |
| Root .env | Satu .env di root | |
| Docker environment | Semua dari env Docker | |

**User's choice:** Workspace monorepo, Turborepo, masing-masing migration sendiri, masing-masing .env
**Notes:** Struktur workspace dengan Turborepo untuk orchestrasi build kedua service.

---

## Deferred Ideas

None.

