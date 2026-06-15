# Phase 83: buat onboarding untuk mempermudah user membuat server yang di inginkan ketika menekan 'Create your first server' di dashboard utama - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-16
**Phase:** 83-buat-onboarding-untuk-mempermudah-user-membuat-server-yang-d
**Areas discussed:** Trigger behavior, Onboarding flow format, Game type guidance, Existing modal integration

---

## Trigger Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Modal langsung di dashboard | Buka CreateServerModal langsung tanpa navigasi | |
| Navigasi ke /servers | Pertahankan navigate('/servers') seperti sekarang | |
| Onboarding wizard dulu | Tampilkan wizard multi-step sebelum create server | ✓ |

**User's choice:** Onboarding wizard dulu
**Notes:** User ingin pengalaman terpandu untuk first-time user, bukan langsung masuk ke form atau pindah halaman.

---

## Onboarding Flow Format

| Option | Description | Selected |
|--------|-------------|----------|
| Multi-step wizard modal | Modal besar dengan progress bar, step-by-step | ✓ |
| Side panel guidance | Modal create server dengan panel guidance di samping | |
| Tooltip/overlay tour | Navigasi ke /servers lalu overlay tutorial | |

**User's choice:** Multi-step wizard modal

---

## Wizard Steps Count

| Option | Description | Selected |
|--------|-------------|----------|
| 3 steps | Type → Config → Confirm | |
| 4 steps | Type → Resources → Config → Deploy | ✓ |
| 2 steps | Type → Everything | |

**User's choice:** 4 steps (Type → Resources → Config → Deploy)

---

## Game Type Guidance (Step 1)

| Option | Description | Selected |
|--------|-------------|----------|
| Cards with explanations | 3 kartu besar (Java, Bedrock, PocketMine) dengan ikon, deskripsi, rekomendasi | ✓ |
| Dropdown with filtering | Dropdown dengan label rekomendasi | |
| Quiz-style questions | Tanya user pertanyaan pendek, rekomendasikan otomatis | |

**User's choice:** Cards with explanations

---

## Resources Step (Step 2)

| Option | Description | Selected |
|--------|-------------|----------|
| Show plan cards | Tampilkan plan (Free/Hobby/Pro) dengan fitur, user pilih plan | ✓ |
| Slider + auto plan | Slider RAM, auto-pilih plan | |
| Skip — default plan | Default ke Free, bisa ganti nanti | |

**User's choice:** Show plan cards — dengan catatan RAM/CPU/Disk dipilih user dalam batasan plan, bukan preset

---

## Config Step (Step 3)

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal config only | Nama server + version saja | |
| Full config | Nama, version, port, template | ✓ |
| Template-first | Pilih template dulu (Vanilla, Paper, dll) | |

**User's choice:** Full config

---

## Deploy Step (Step 4)

| Option | Description | Selected |
|--------|-------------|----------|
| Review + Deploy button | Ringkasan + tombol deploy | |
| Review + auto-redirect | Ringkasan + deploy + redirect ke server detail | ✓ |
| Review + deploy + next tips | Ringkasan + deploy + tips setelahnya | |

**User's choice:** Review + auto-redirect ke server detail page setelah deploy

---

## Existing Modal Integration

| Option | Description | Selected |
|--------|-------------|----------|
| New component, reuse logic | ServerOnboardingWizard baru, reuse API create logic | ✓ |
| Replace CreateServerModal | Ganti modal lama dengan wizard baru | |
| Wizard wrapping modal | Wizard sebagai wrapper, buka CreateServerModal di step 4 | |

**User's choice:** New component (ServerOnboardingWizard), reuse API create logic. CreateServerModal tetap ada.

---

## Wizard Trigger Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Dashboard only | Wizard hanya dari dashboard empty state | ✓ |
| Both dashboard & /servers | Wizard juga muncul di /servers untuk first-time user | |

**User's choice:** Dashboard only — /servers tetap pakai CreateServerModal biasa

---

## the agent's Discretion

- Exact visual design of cards in Step 1 (icon size, layout density)
- Progress bar style and positioning
- Default values for port (random), version (latest), template (none)
- Plan card layout in Step 2
- Deploy animation/loading state after clicking "Deploy Server"
- Confetti or celebratory element on success

## Deferred Ideas

None
