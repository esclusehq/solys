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

### 1. Early Stage (v0.x.y)

Selama masih di fase awal (v0), gunakan aturan berikut:

- **0.1.0** - Initial public release untuk fitur signifikan
- **0.2.0** - Fitur baru major
- **0.0.x** - Bug fix dan perbaikan minor

### 2. Versi Tidak Boleh Minus

Tidak pernah ada versi negatif. Selalu naik:

```
0.0.0 → 0.0.1 → 0.0.2
0.1.0 → 0.2.0 → 0.3.0
1.0.0 → 2.0.0 → 3.0.0
```

### 3. Urutan Release

Release harus diurutkan dari yang terbaru ke yang tertua (numerik, bukan tanggal):

```
0.1.0  (terbaru)
0.0.2
0.0.1
0.0.0  (tertua)
```

---

## Wajib: Update Changelog Sebelum Deploy

Setiap kali ada perubahan kode yang di-deploy ke production, **WAJIB** update Changelog di landing page.

### Lokasi Changelog

`landing-page-escluse/src/pages/Changelog.tsx`

### Format Entry Changelog

```typescript
{
  version: '0.1.0',
  date: '2026-05-17',
  type: 'minor',  // 'major' | 'minor' | 'patch' | 'initial'
  changes: {
    added: ['Fitur baru 1', 'Fitur baru 2'],
    improved: ['Perbaikan 1'],
    fixed: ['Bug fix 1'],
    removed: ['Fitur yang dihapus'],
    security: ['Security fix']
  }
}
```

### Urutan Changelog

1. Versi terbaru selalu di TOP array
2. Satu versi = satu tanggal (tidak ada rentang)
3. Gunakan format SemVer yang benar

---

## Checklist Sebelum Deploy

Setiap kali melakukan deploy ke EC2 (lihat [DEPLOY.md](./DEPLOY.md)):

- [ ] Apakah ada perubahan kode/features yang di-deploy?
- [ ] Apakah changelog sudah diupdate dengan entry versi baru?
- [ ] Apakah format changelog sesuai SemVer (Added, Improved, Fixed, Removed, Security)?

### Jika YA - Ada Perubahan

1. Tambah entry baru di `Changelog.tsx` dengan versi yang tepat
2. Commit dengan format: `feat: [deskripsi]` atau `fix: [deskripsi]`
3. Push ke GitHub
4. Deploy via [DEPLOY.md](./DEPLOY.md)

### Jika TIDAK - Tidak Ada Perubahan

Langsung deploy tanpa update changelog.

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