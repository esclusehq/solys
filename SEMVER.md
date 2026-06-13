# Semantic Versioning (SemVer) - Escluse

Dokumen ini menjelaskan aturan versioning yang wajib diikuti untuk semua release Escluse.

## Format Version

```
MAJOR.MINOR.PATCH
```

| Level | Format | Arti |
|-------|--------|------|
| Major | X.0.0 | Breaking changes - tidak backward compatible |
| Minor | 0.X.0 | Fitur baru yang backward compatible |
| Patch | 0.0.X | Bug fix yang backward compatible |

## Aturan untuk Escluse

### 1. Release Schedule — Weekly (Seminggu Sekali)

Release dilakukan **setiap hari Minggu** (atau hari terakhir minggu itu). Satu minggu = satu versi. Jika tidak ada perubahan di minggu tersebut, tetap rilis dengan versi patch bump dan entry changelog "No significant changes".

### 2. Early Stage (v0.x.y)

Selama masih di fase awal (v0), gunakan aturan berikut:

- **0.1.0, 0.2.0, 0.3.0, ...** - Minor bump setiap minggu untuk fitur baru
- **0.0.x** - Patch hanya untuk hotfix di luar jadwal mingguan

Tidak ada major bump sampai rilis stabil (v1.0.0).

### 3. Weekly Bump Pattern

Setiap minggu, versi MINOR naik 1:

```
Minggu 1: 0.1.0
Minggu 2: 0.2.0
Minggu 3: 0.3.0
...
Minggu N: 0.N.0
```

Patch bump (0.0.x) hanya untuk hotfix darurat di tengah minggu. Jika ada hotfix, tetap merge ke entry mingguan berikutnya.

### 4. Hotfix di Tengah Minggu

Jika deploy dilakukan di luar jadwal Minggu (hotfix/patch), bump PATCH pada versi saat ini:

```
Minggu:  0.4.0  (release mingguan)
Selasa:  0.4.1  (hotfix — deploy darurat)
Minggu:  0.5.0  (next release, merge semua hotfix)
```

Hotfix tetap dicatat di changelog dan akan digabung ke entry mingguan berikutnya.

### 5. Versi Tidak Boleh Minus

Tidak pernah ada versi negatif. Selalu naik:

```
0.0.0 → 0.1.0 → 0.2.0
0.2.0 → 0.2.1 (hotfix) → 0.3.0
```

### 6. Urutan Release

Release harus diurutkan dari yang terbaru ke yang tertua (numerik, bukan tanggal):

```
0.3.0  (terbaru — minggu ini)
0.2.0
0.1.0
0.0.0  (tertua)
```

---

## Wajib: Update Changelog Tiap Release

**WAJIB** update Changelog di landing page setiap kali rilis mingguan.

### Lokasi Changelog

`landing-page-escluse/src/pages/Changelog.tsx`

### Format Entry Changelog

```typescript
{
  version: '0.3.0',
  date: '2026-06-01',
  type: 'minor',  // 'minor' (mingguan) | 'patch' (hotfix)
  changes: {
    added: ['[app] Fitur baru 1', '[solys] Fitur baru 2'],
    improved: ['[app] Perbaikan 1'],
    fixed: ['[solys] Bug fix 1', '[app] Bug fix 2'],
    removed: ['[app] Fitur yang dihapus'],
    security: ['[app] Security fix']
  }
}
```

### Component Tags

Setiap item di changelog WAJIB diberi tag komponen:

| Tag | Komponen | Repo |
|-----|----------|------|
| `[app]` | Dashboard frontend | esclusehq/escluse-dashboard |
| `[solys]` | Agent (instalasi, binary, CI) | esclusehq/solys |
| `[landing]` | Landing page & marketing | esclusehq/escluse-landing-page |
| `[api]` | Backend API & worker | esclusehq/escluse-cloud |
| `[docs]` | Dokumentasi | esclusehq/escluse-docs |

### Urutan Changelog

1. Versi terbaru selalu di TOP array
2. Satu versi = satu tanggal (tidak ada rentang)
3. Gunakan format SemVer yang benar

---

## Checklist Sebelum Deploy

Setiap minggu sebelum deploy ke EC2 (lihat [DEPLOY.md](./DEPLOY.md)):

- [ ] Apakah sudah hari Minggu (atau akhir minggu)?
- [ ] Apakah changelog sudah diupdate dengan entry versi baru?
- [ ] Apakah format changelog sesuai SemVer (Added, Improved, Fixed, Removed, Security)?
- [ ] Apakah semua commit dari semua repo sudah tercakup? (API, frontend, agent, landing page)

### Jika YA - Ada Perubahan

1. Bump MINOR di `Changelog.tsx` (0.N.0)
2. Kumpulkan semua commit dari semua repo sejak release terakhir
3. Commit dengan format: `docs(changelog): add v0.N.0 release notes`
4. Push ke GitHub
5. Deploy via [DEPLOY.md](./DEPLOY.md)

### Jika TIDAK - Tidak Ada Perubahan

Tetap rilis dengan patch bump dan entry:
```
changes: { improved: ['No significant changes this week'] }
```

---

## Commit Message Format

Gunakan prefix yang sesuai:

| Prefix | Arti |
|--------|------|
| `feat:` | Fitur baru |
| `fix:` | Bug fix |
| `refactor:` | Refactoring |
| `docs:` | Dokumentasi |
| `chore:` | Maintenance |

Contoh:
```
feat: add user profile page
fix: resolve login redirect issue
docs(changelog): add v0.1.0 release notes
```

---

## Referensi

- [Semantic Versioning 2.0.0](https://semver.org/)
- [DEPLOY.md](./DEPLOY.md) - Panduan deployment ke EC2
- [PUSH_COMMIT.md](./PUSH_COMMIT.md) - Panduan push ke GitHub