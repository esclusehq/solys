# Push & Commit Guide - Escluse Repositories

Dokumen ini berisi step-by-step untuk commit dan push semua repository Escluse ke GitHub.

## Prerequisites

```bash
# GitHub Personal Access Token (PAT)
# Buat di: https://github.com/settings/tokens
# Scopes: repo (full control)
```

## Repository Mapping

| Folder Lokal | GitHub Repo | Description |
|--------------|-------------|-------------|
| `agent/solys/` | `esclusehq/solys` | **Monorepo root** — Solys Agent (Rust), install scripts, CI/CD, CHANGELOG |
| `agent/agent-core/` | `esclusehq/agent-core` | Agent Core Framework - Rust crates |
| `app/` | `esclusehq/escluse-dashboard` | React Dashboard - Frontend |
| `api/` + `migration/` | `esclusehq/escluse-cloud` | Rust Backend - API + Migrations |
| `landing-page-escluse/` | `esclusehq/escluse-landing-page` | Landing Page - Marketing |
| `packages/` | `esclusehq/escluse-sdk` | SDK - Types, SDK Node, SDK Python, CLI |
| `docs/` | `esclusehq/escluse-docs` | Documentation - VitePress |
| `gateway/`, `compose/`, `docker/`, `opt/`, `docker-compose.yml` | `esclusehq/escluse-infra` | Infrastructure - Docker, Caddy, Relay Gateway |
| `worker/` | `esclusehq/escluse-cloud` (worker dir) | Background worker service |

**Note:** Folder `worker/` adalah bagian dari `escluse-cloud`, masukkan saat push/update api.

---

## Step-by-Step untuk Setiap Repo

### 1. Inisialisasi Git (jika belum ada)

```bash
cd /path/to/repo
git init
git remote add origin https://github.com/esclusehq/{repo-name}.git
git config user.email "dev@esluce.com"
git config user.name "Escluse Dev"
```

### 2. Buat .gitignore

```bash
# Contoh untuk project Node.js
cat > .gitignore << 'EOF'
node_modules/
dist/
.env
.env.*
*.log
.DS_Store
target/
Cargo.lock
EOF
```

### 3. Create GitHub Repo (jika belum ada)

```bash
curl -s -X POST \
  -H "Authorization: token YOUR_PAT" \
  -H "Content-Type: application/json" \
  https://api.github.com/orgs/esclusehq/repos \
  -d '{"name":"repo-name","description":"Description","private":false}'
```

### 4. Make Repo Public (jika private)

```bash
curl -s -X PATCH \
  -H "Authorization: token YOUR_PAT" \
  -H "Content-Type: application/json" \
  https://api.github.com/repos/esclusehq/repo-name \
  -d '{"private": false}'
```

### 5. Stage Files

```bash
# Exclude sensitive files
git add .gitignore README.md src/ package.json Cargo.toml ...

# Check what's staged
git status --short
```

### 6. Buat Commit

```bash
git commit -m "Initial commit: [Project Name]

[Features list]"
```

### 7. Push ke GitHub

```bash
git push -u origin master << 'EOF'
GITHUB_USERNAME
YOUR_PAT_TOKEN
EOF
```

---

## Checklist Sebelum Commit & Push

**WAJIB** dilakukan sebelum setiap commit dan push ke GitHub:

### 1. Update Dokumentasi

Sebelum commit, selalu periksa dan update file dokumentasi berikut:

| File | Harus diupdate jika... |
|------|----------------------|
| `CHANGELOG.md` (per repo) | Ada fitur baru, bug fix, atau perubahan API |
| **Landing Page** (`/changelog`) | **WAJIB** untuk perubahan platform Escluse mencakup: `escluse-landing-page`, `escluse-docs`, `escluse-infra`, `escluse-cloud`, `escluse-dashboard`. **KECUALI** untuk `escluse-sdk`, `solys`, `agent-core` yang sudah punya CHANGELOG.md masing-masing |
| `README.md` | Ada perubahan cara install, setup, atau fitur utama |
| `DEPLOY.md` | Ada perubahan deployment atau konfigurasi server |
| `ARCHITECTURE.md` | Ada perubahan arsitektur atau alur data |
| `CONTRIBUTING.md` | Ada perubahan cara berkontribusi |
| `.md lain` | Ada perubahan yang relevan untuk didokumentasikan |

### 2. Checklist Commit

- [ ] Ubah kode/features?
- [ ] CHANGELOG.md diupdate? (WAJIB jika ada perubahan - lihat [SEMVER.md](./SEMVER.md))
- [ ] README.md diupdate? (jika ada perubahan install/setup)
- [ ] Semua file `.md` sudah konsisten dengan perubahan?
- [ ] Tidak ada secrets atau credentials di commit
- [ ] Branch sudah benar (`master` atau feature branch)

### 3. Format Commit Message

```bash
# Format: type(scope): description
#
# Types:
#   feat    - Fitur baru
#   fix     - Bug fix
#   docs    - Dokumentasi (WAJIB setelah setiap fitur/bug fix)
#   refactor - Refactoring kode
#   chore   - Maintenance, dependency update
#
# Contoh:
git commit -m "feat(sdk): add WebSocket support for real-time events"
git commit -m "fix(changelog): update v0.2.0 release notes"
git commit -m "docs: update installation steps in README.md"
```

### 4. Versioning Rules

**WAJIB** mengikuti aturan di [SEMVER.md](./SEMVER.md).

Ringkasan:
- Format: `MAJOR.MINOR.PATCH` (tanpa prefix `v`)
- Early stage: mulai dari `0.1.0` untuk initial release
- Patch: `0.0.x` untuk bug fix
- Minor: `0.x.0` untuk fitur baru

### 5. Update Changelog Setelah Commit & Push

**WAJIB** untuk setiap perubahan yang di-deploy ke production:

1. Setelah push ke GitHub, update changelog di landing page:
   - File: `landing-page-escluse/src/pages/Changelog.tsx`
   - Ikuti format: Added, Improved, Fixed, Removed, Security
   - Versi terbaru selalu di TOP array

2. Commit update changelog dengan format:
   ```
   git add -A
   git commit -m "docs(changelog): add vX.Y.Z - [deskripsi]"
   git push origin master
   ```

3. Deploy via [DEPLOY.md](./DEPLOY.md)

**Lihat [SEMVER.md](./SEMVER.md) untuk detail lengkap.**

### GitHub Projects Integration (WAJIB)

Setiap commit yang punya perubahan kode/features WAJIB juga:

**1. Buat/Update Issue di GitHub:**
```bash
# Bug fix
gh issue create --repo esclusehq/escluse-cloud \
  --title "fix: [deskripsi bug]" \
  --body "## Problem\n[deskripsi]\n\n## Solution\n[solusi]" \
  --label bug

# Feature baru
gh issue create --repo esclusehq/escluse-cloud \
  --title "feat: [deskripsi feature]" \
  --body "## Summary\n[deskripsi]" \
  --label feature

# Phase baru (dari .planning/ROADMAP.md)
# Gunakan standar GitHub: feat:, fix:, docs:, bukan "phase:"
gh issue create --repo esclusehq/escluse-infra \
  --title "feat: [nama phase]" \
  --body "## Goals\n[goals dari PLAN.md]\n\n## Depends on\n[phase number]" \
  --add-label enhancement
```

**2. Add ke Project "Development Roadmap":**
```bash
gh project item-add 1 --owner esclusehq --url "ISSUES_URL"
```

**3. Update Status (jika sudah selesai):**
- Buka https://github.com/orgs/esclusehq/projects/1
- Klik item → ubah Status: To Do → In Progress → Done

**Labels yang WAJIB digunakan:**

| Label | Penggunaan |
|-------|-------------|
| `bug` | Bug fix dan perbaikan error |
| `feature` | Feature baru |
| `enhancement` | Perbaikan/optimasi yang bukan bug |
| `documentation` | Perubahan docs (README, CHANGELOG, dll) |
| `chore` | Maintenance, dependency update |
| `refactor` | Code refactoring |

**Priority Labels (opsional):**
| Label | Penggunaan |
|-------|-------------|
| `priority:high` | Urgent, harus selesai secepatnya |
| `priority:medium` | Normal priority |
| `priority:low` | Tidak urgent |

**Contoh lengkap:**
```bash
# Bug fix dengan priority
gh issue create --repo esclusehq/escluse-cloud \
  --title "fix: [deskripsi]" \
  --body "## Problem\n..." \
  --label "bug,priority:high"

# Feature baru
gh issue create --repo esclusehq/escluse-landing-page \
  --title "feat: [deskripsi]" \
  --body "## Summary\n..." \
  --label "feature,priority:medium"

# Docs update
gh issue create --repo esclusehq/escluse-docs \
  --title "docs: [deskripsi]" \
  --body "..." \
  --label "documentation"

# Refactor
gh issue create --repo esclusehq/escluse-cloud \
  --title "refactor: [deskripsi]" \
  --body "..." \
  --label "refactor"
```

**Project URL:** https://github.com/orgs/esclusehq/projects/1

---

## Quick Command untuk Semua Repo

### agent/solys

```bash
cd agent/solys
git add -A && git status --short  # Check semua perubahan

# WAJIB: Update CHANGELOG.md sebelum commit
# Edit CHANGELOG.md dengan perubahan terbaru

git commit -m "feat: [description]"
git push origin master
```

### agent/agent-core

```bash
cd agent/agent-core
git add -A && git status --short  # Check semua perubahan

# WAJIB: Update CHANGELOG.md sebelum commit
# Edit CHANGELOG.md dengan perubahan terbaru

git commit -m "feat: [description]"
git push origin master
```

### app/ (Dashboard)

```bash
cd app
git add -A && git status --short  # Check semua perubahan

# WAJIB: Update CHANGELOG.md dan README.md sebelum commit

git commit -m "feat: [description]"
git push origin master
```

### api/ + worker/ (Backend)

```bash
cd api
git add -A && git status --short  # Check semua perubahan

# WAJIB: Update CHANGELOG.md dan README.md sebelum commit

git commit -m "feat: [description]"
git push origin master
```

### landing-page-escluse/

```bash
cd landing-page-escluse
git add -A && git status --short  # Check semua perubahan

# WAJIB: Update 2 changelog sebelum commit
# 1. CHANGELOG.md di repo (untuk escluse-sdk, solys, agent-core)
# 2. Changelog.tsx di landing page (untuk platform: escluse-landing-page, escluse-docs, escluse-infra, escluse-cloud, escluse-dashboard)

# Contoh update Changelog.tsx:
# Tambahkan entry baru di array changelog di src/pages/Changelog.tsx
# mencakup perubahan dari: escluse-landing-page, escluse-docs, escluse-infra, escluse-cloud, escluse-dashboard

git commit -m "feat: [description]"
git push origin master
```

### packages/ (SDK)

```bash
cd packages
git add -A && git status --short  # Check semua perubahan

# WAJIB: Update CHANGELOG.md dan README.md sebelum commit

git commit -m "feat: [description]"
git push origin master
```

### docs/

```bash
cd docs
git add -A && git status --short  # Check semua perubahan

# WAJIB: Update index.md atau file dokumentasi yang relevan sebelum commit

git commit -m "docs: [description]"
git push origin master
```

### gateway/ + compose/ + docker/ + opt/ + docker-compose.yml (Infrastructure → `esclusehq/escluse-infra`)

Infrastructure files (Caddy config, Docker compose, relay gateway) dikelola di repo terpisah `esclusehq/escluse-infra`. Cara sync:

```bash
# Clone atau pull escluse-infra
cd /tmp
rm -rf escluse-infra
git clone https://github.com/esclusehq/escluse-infra.git

# Copy files dari monorepo utama
cp /home/rhnbztnl/Downloads/Berguna/Projects/escluse/docker-compose.yml /tmp/escluse-infra/
cp -r /home/rhnbztnl/Downloads/Berguna/Projects/escluse/gateway/ /tmp/escluse-infra/
cp -r /home/rhnbztnl/Downloads/Berguna/Projects/escluse/compose/ /tmp/escluse-infra/
cp -r /home/rhnbztnl/Downloads/Berguna/Projects/escluse/docker/ /tmp/escluse-infra/
cp -r /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/ /tmp/escluse-infra/

cd /tmp/escluse-infra
git config user.email "dev@esluce.com"
git config user.name "Escluse Dev"

git add -A
git status --short  # Verifikasi hanya file yang diinginkan
git commit -m "feat: sync infra files"
git push origin master
```

---

## Folder Structure Summary

```
escluse/
├── agent/
│   ├── solys/           → github.com/esclusehq/solys          (ROOT — tracked by solys)
│   └── agent-core/     → github.com/esclusehq/agent-core      (embedded repo, gitignored)
├── app/                → github.com/esclusehq/escluse-dashboard (embedded repo, gitignored)
├── api/                → github.com/esclusehq/escluse-cloud    (embedded repo, gitignored)
│   └── worker/         (included in api)
├── migration/          → github.com/esclusehq/escluse-cloud    (embedded repo, gitignored)
├── landing-page-escluse/ → github.com/esclusehq/escluse-landing-page (embedded repo, gitignored)
├── packages/           → github.com/esclusehq/escluse-sdk      (embedded repo, gitignored)
├── docs/               → github.com/esclusehq/escluse-docs     (embedded repo, gitignored)
├── compose/            → github.com/esclusehq/escluse-infra    (sync manual, gitignored)
├── docker/             → github.com/esclusehq/escluse-infra    (sync manual, gitignored)
├── opt/                → github.com/esclusehq/escluse-infra    (sync manual, gitignored)
├── gateway/            → github.com/esclusehq/escluse-infra    (sync manual, gitignored)
├── docker-compose.yml  → github.com/esclusehq/escluse-infra    (sync manual, gitignored)
├── installer/          → github.com/esclusehq/solys            (tracked by solys)
├── packaging/          → github.com/esclusehq/solys            (tracked by solys)
└── src/                → github.com/esclusehq/solys            (tracked by solys — Rust agent)
```
└── Dockerfile.landing  → github.com/esclusehq/escluse-landing-page
```

---

## Troubleshooting

### Remote already exists

```bash
git remote remove origin
git remote add origin https://github.com/esclusehq/repo.git
```

### Push rejected (secrets detected)

```bash
# Check commit untuk secrets
git log --oneline

# Amend commit untuk hapus secrets
git commit --amend --no-edit
git push --force
```

### Permission denied

```bash
# Pastikan PAT punya scope 'repo'
curl -s -H "Authorization: token YOUR_PAT" https://api.github.com/user
```

---

## Automation Script

```bash
#!/bin/bash
# push-all.sh - Push semua repo ke GitHub

TOKEN="YOUR_PAT"
REPOS=(
  "agent/solys:solys"
  "agent/agent-core:agent-core"
  "app:escluse-dashboard"
  "api:escluse-cloud"
  "landing-page-escluse:escluse-landing-page"
  "packages:escluse-sdk"
  "docs:escluse-docs"
  "gateway:escluse-infra"
)

for repo in "${REPOS[@]}"; do
  dir="${repo%%:*}"
  name="${repo##*:}"
  
  echo "Pushing $dir -> $name"
  
  if [ "$dir" = "gateway" ]; then
    # Special handling for infra
    mkdir -p /tmp/escluse-infra-temp
    cp -r gateway/* /tmp/escluse-infra-temp/ 2>/dev/null
    cp docker-compose.yml /tmp/escluse-infra-temp/ 2>/dev/null
    cd /tmp/escluse-infra-temp
  else
    cd "$dir"
  fi
  
  if [ ! -d ".git" ]; then
    git init
    git remote add origin "https://github.com/esclusehq/$name.git"
    git config user.email "dev@esluce.com"
    git config user.name "Escluse Dev"
  fi
  
  git push -u origin master <<< "GITHUB_USERNAME
$TOKEN"
  
  echo "Done: $name"
  echo "---"
done
```

---

## Links

| Repo | URL |
|------|-----|
| Dashboard | https://github.com/esclusehq/escluse-dashboard |
| Backend | https://github.com/esclusehq/escluse-cloud |
| Landing Page | https://github.com/esclusehq/escluse-landing-page |
| SDK | https://github.com/esclusehq/escluse-sdk |
| Documentation | https://github.com/esclusehq/escluse-docs |
| Infrastructure | https://github.com/esclusehq/escluse-infra |
| Solys Agent | https://github.com/esclusehq/solys |
| Agent Core | https://github.com/esclusehq/agent-core |

**GitHub Organization:** https://github.com/esclusehq

---

## ⚠️ Supabase Data API - Important Deadline

**Project:** escluse-app (Supabase)

**Deadline:** 30 Oktober 2026

### Apa yang Berubah?

Mulai 30 Oktober 2026, Supabase akan mengubah default behavior untuk Data API:

- **Sebelum:** Tabel baru di schema `public` otomatis bisa diakses via Data API
- **Sesudah:** BUTUH `GRANT` eksplisit agar bisa diakses via `supabase-js`, REST API, atau GraphQL

### Siapa yang Terdampak?

| Menggunakan | Terdampak? |
|-------------|------------|
| `supabase-js` (client-side) | ✅ Ya |
| REST API (`/rest/v1/`) | ✅ Ya |
| GraphQL (`/graphql/v1/`) | ✅ Ya |
| Koneksi langsung Postgres (psql, ORM) | ❌ Tidak |

### Yang Perlu Dilakukan

Untuk setiap **tabel baru** yang mau diakses via Data API, tambahkan `GRANT` di migration:

```sql
-- 1. Grant akses ke role anon (public read)
GRANT SELECT ON public.your_table TO anon;

-- 2. Grant akses ke authenticated users
GRANT ALL ON public.your_table TO authenticated;

-- 3. Grant akses ke service_role (admin)
GRANT ALL ON public.your_table TO service_role;

-- 4. Enable Row Level Security
ALTER TABLE public.your_table ENABLE ROW LEVEL SECURITY;

-- 5. Buat policy sesuai kebutuhan
CREATE POLICY "users can read own data" ON public.your_table
  FOR SELECT TO authenticated
  USING (auth.uid() = user_id);
```

### Checklist Migration

- [ ] Cek semua migration baru
- [ ] Pastikan ada `GRANT` untuk setiap tabel baru
- [ ] Pastikan ada `ALTER TABLE ... ENABLE ROW LEVEL SECURITY`
- [ ] Pastikan ada `CREATE POLICY` untuk RLS

**Referensi:** Email dari Supabase (Mei 2026) - "New Supabase projects will not expose tables in the public schema to the Data API by default"

---

*Terakhir diupdate: 2026-05-17*