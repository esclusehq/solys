# GitHub Workflows Guide

Dokumentasi tentang GitHub Actions Workflows untuk project Escluse.

---

## Apa itu GitHub Workflows?

GitHub Workflows adalah otomatisasi CI/CD dan proses development yang berjalan di GitHub Actions.

## Kegunaan Utama

### 1. Continuous Integration (CI)
- Otomatis jalankan test setiap push/PR
- Build project
- Lint/format code check

### 2. Continuous Deployment (CD)
- Auto deploy ke server (Vercel, Netlify, AWS, dll)
- Release management (auto versioning, changelog)

### 3. Otomatisasi Lainnya
- Notifikasi (Slack, Discord)
- Auto label/assign issues
- Code scanning (security vulnerabilities)
- Auto reply to PRs

---

## Cara Kerja

1. Buat file `.github/workflows/*.yml`
2. Trigger: push, PR, schedule, manual
3. Run di GitHub runners (gratis untuk public repos)

---

## Contoh Workflows untuk Escluse

### 1. Rust Backend - CI (Test & Build)

```yaml
name: Rust CI

on:
  push:
    branches: [master, develop]
  pull_request:
    branches: [master]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test Suite
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo index
        uses: actions/cache@v4
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo build
        uses: actions/cache@v4
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}

      - name: Run tests
        run: cargo test --all

      - name: Run clippy
        run: cargo clippy --all -- -D warnings

      - name: Build
        run: cargo build --release
```

### 2. React/Vite Frontend - CI (Test & Build)

```yaml
name: Frontend CI

on:
  push:
    branches: [master, develop]
  pull_request:
    branches: [master]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'

      - name: Install dependencies
        run: npm ci

      - name: Run tests
        run: npm test

      - name: Build
        run: npm run build
```

### 3. Auto Deploy ke Vercel (Landing Page / Dashboard)

```yaml
name: Deploy to Vercel

on:
  push:
    branches: [master]
    paths:
      - 'landing-page-escluse/**'

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install Vercel CLI
        run: npm install -g vercel-cli

      - name: Pull Vercel Environment
        run: vercel pull --yes --environment=production --token=${{ secrets.VERCEL_TOKEN }}

      - name: Build
        run: vercel build --prod --token=${{ secrets.VERCEL_TOKEN }}

      - name: Deploy
        run: vercel deploy --prebuilt --prod --token=${{ secrets.VERCEL_TOKEN }}
```

### 4. Auto Sync Issue ke GitHub Project

```yaml
name: Auto Add Issues to Project

on:
  issues:
    types: [opened]

jobs:
  add-to-project:
    runs-on: ubuntu-latest
    steps:
      - name: Get project number
        run: echo "PROJECT_NUMBER=1" >> $GITHUB_ENV

      - name: Add issue to project
        uses: actions/add-to-project@v0.5.0
        with:
          project-url: https://github.com/orgs/esclusehq/projects/1
          github-token: ${{ secrets.GITHUB_TOKEN }}
          labeled: ''
          label-operator: ''
```

### 5. Auto Label Issues

```yaml
name: Auto Label Issues

on:
  issues:
    types: [opened, labeled]
  pull_request:
    types: [opened, labeled]

jobs:
  label:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/labeler@v5
        with:
          repo-token: ${{ secrets.GITHUB_TOKEN }}
          configuration-path: .github/labeler.yml
```

Buat file `.github/labeler.yml`:
```yaml
bug:
  - any: ['**/*.rs']
    all: ['bug', 'fix']

feature:
  - any: ['**/*.rs', '**/*.tsx', '**/*.ts']
    all: ['feat', 'feature']

documentation:
  - any: ['**/*.md', '**/*.mdx', 'docs/**']
    all: ['docs']
```

### 6. Auto Update Project Status

```yaml
name: Update Project Status

on:
  issues:
    types: [closed, reopened]
  pull_request:
    types: [closed, merged]

jobs:
  update-project:
    runs-on: ubuntu-latest
    steps:
      - name: Move issue to Done
        if: github.event_name == 'issues' && github.event.action == 'closed'
        uses: actions/update-project-item-fields@v1
        with:
          project-url: https://github.com/orgs/esclusehq/projects/1
          item-url: ${{ github.event.issue.html_url }}
          field: Status
          value: Done
          token: ${{ secrets.GITHUB_TOKEN }}
```

### 7. Security Scanning

```yaml
name: Security Audit

on:
  push:
    branches: [master]
  schedule:
    - cron: '0 0 * * 0'  # Weekly

jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run Securify (Rust)
        uses: rustsec/audit-check@v1.4.1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}

      - name: Run npm audit
        if: contains(github.event.repository.name, 'landing')
        run: npm audit --audit-level=high
```

### 8. Auto Release dengan Changelog

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Generate Changelog
        id: changelog
        uses: metcalfc/changelog-generator@v4
        with:
          myToken: ${{ secrets.GITHUB_TOKEN }}

      - name: Create Release
        uses: actions/create-release@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tag_name: ${{ github.ref }}
          release_name: Release ${{ github.ref }}
          body: ${{ steps.changelog.outputs.changelog }}

      - name: Upload Release Asset
        uses: actions/upload-release-asset@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          upload_url: ${{ steps.create_release.outputs.upload_url }}
          asset_path: ./build/artifacts.tar.gz
          asset_content_type: application/gzip
```

---

## Secrets yang Diperlukan

Simpan di Settings → Secrets and variables → Actions:

| Secret | Kegunaan |
|--------|----------|
| `VERCEL_TOKEN` | Deploy ke Vercel |
| `VERCEL_PROJECT_ID` | ID project Vercel |
| `VERCEL_ORG_ID` | ID organization Vercel |
| `DEPLOY_KEY` | SSH key untuk deploy ke server |
| `SLACK_WEBHOOK` | Notifikasi ke Slack |

---

## Repository yangPerlu Workflow

| Repo | Workflow |
|------|----------|
| `escluse-cloud` | Rust CI, Security audit |
| `escluse-landing-page` | Vercel deploy |
| `escluse-dashboard` | Vercel deploy |
| `escluse-docs` | Vercel deploy |
| `escluse-sdk` | npm publish |

---

## Referensi

- [GitHub Actions Docs](https://docs.github.com/en/actions)
- [ Marketplace Actions](https://github.com/marketplace?type=actions)
- [Workflow Syntax](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions)

---

*Terakhir diupdate: 2026-05-17*