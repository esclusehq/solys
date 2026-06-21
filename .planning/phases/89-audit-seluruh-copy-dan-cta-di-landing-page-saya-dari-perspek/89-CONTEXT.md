# Phase 89: Audit Seluruh Copy dan CTA di Landing Page — Context

**Gathered:** 2026-06-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Audit seluruh copy dan CTA di landing page Escluse (esluce.com) dari perspektif hukum/legalitas bisnis digital. Mencakup 3 menu grup: Product (Features, Pricing, Supported Games, Changelog), Resources (Documentation, API Reference, Community Forum), Company (About Us, Legal, Contact). Output berupa laporan dengan klasifikasi 3-tier: Safe / Risky / Must Avoid.

Hanya file landing page di `landing-page-escluse/src/` — external sites (docs.esluce.com, app.esluce.com) **explicitly scoped out**.

</domain>

<spec_lock>
## Requirements (locked via UI-SPEC.md)

**11 risk areas and 15 pages/components** are catalogued. See `89-UI-SPEC.md` for full inventory, boundaries, and risk area identification.

Downstream agents MUST read `89-UI-SPEC.md` before planning or implementing.

**In scope (from UI-SPEC.md):** Audit copy pada landing page dan semua sub-halamannya (15 halaman/komponen) mencakup 11 area risiko hukum
**Out of scope (from UI-SPEC.md):** Perubahan UI/UX, implementasi desain baru, external sites

</spec_lock>

<decisions>
## Implementation Decisions

### Legal Framework
- **D-01 (Hybrid Framework):** Gunakan hybrid — Indonesian law (UU ITE, UU PDP, consumer protection) + international (GDPR, US FTC guidelines) sebagai benchmark. Tidak terbatas pada satu yurisdiksi.

### Audit Detail & Legal Reasoning
- **D-02 (Legal Detail):** Cite specific legal provisions (UU PDP articles, UU ITE provisions, GDPR articles, FTC guidance). Assess business risk per finding: enforcement likelihood, reputational impact, platform policy implications, risk severity.
- **D-03 (Actionability):** Untuk item Risky dan Must Avoid, berikan diagnosis + fix suggestions konkret (alternative phrasings yang legally safe).

### Report Structure & Output
- **D-04 (Report Structure):** Organize by risk tier (Safe / Risky / Must Avoid). Within each tier, group findings by menu section (Product, Resources, Company).
- **D-05 (Disclaimer):** Sertakan disclaimer prominent di awal laporan bahwa ini adalah compliance review dan bukan legal advice.
- **D-06 (Strategic Output):** 3-tier classification + separate section untuk future recommendations yang bisa dipakai di fase rewrite copy nanti.

### Specific Content
- **D-07 (Supported Games/Trademarks):** Game names (Minecraft, Rust, Terraria) → klasifikasi **Must Avoid** dengan fix suggestions (e.g., "Minecraft-compatible servers", disclaimer tentang tidak ada afiliasi dengan pemilik merek).
- **D-08 (Pricing Claims):** "Simple, Transparent Pricing", "no hidden costs", "Free" plan → low priority. Skip deep analysis — standard marketing scrutiny only.

### Scope Boundaries
- **D-09 (External Sites):** docs.esluce.com dan app.esluce.com explicitly out of scope. Landing page files only.
- **D-10 (No Rewrites in Report):** Report bersifat risk-only (klasifikasi + diagnosis + fix suggestions). Tidak ada rewrite full copy — rewrite direncanakan untuk fase terpisah.

### the agent's Discretion
- Detail teknis klasifikasi per item (mana yang masuk Safe vs Risky vs Must Avoid berdasarkan evidence dari source code)
- Format spesifik dari fix suggestions
- Bobot risk severity per finding

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Artifacts
- `.planning/phases/89-audit-seluruh-copy-dan-cta-di-landing-page-saya-dari-perspek/89-UI-SPEC.md` — Inventory of 15 pages/components and 11 risk areas. Defines audit scope.

### Source Code
- `landing-page-escluse/src/` — All landing page source files to be audited
- `landing-page-escluse/src/App.tsx` — Main navigation and routing (menentukan struktur menu Product/Resources/Company)
- `landing-page-escluse/src/pages/` — All sub-pages (AboutUs, Changelog, Contact, Legal, PrivacyPolicy, TermsOfService, Security, Status, SignIn, SignUp)
- `landing-page-escluse/src/components/` — Components including PricingSection

### Prior Phases
- `.planning/phases/84-menambahkan-halaman-pricing-pada-app/` — Pricing implementation context (tidak ada carry-over signifikan)
- `.planning/phases/87-membuat-halaman-dashboard-untuk-user/` — Dashboard context (tidak ada carry-over signifikan)
- `.planning/phases/88-membuat-halaman-setting-pada-app/` — Settings context (tidak ada carry-over signifikan)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `landing-page-escluse/src/` — Full React+TypeScript landing page codebase. All copy is inline in component/page files.
- `landing-page-escluse/src/pages/Legal.tsx` — Existing legal page with company/contact info — perlu diverifikasi akurasi informasinya.

### Established Patterns
- All copy is in Indonesian/English mixed — perlu diperhatikan konsistensi bahasa dan potensi misleading translation.
- Navigation structure (Product, Resources, Company) — digunakan sebagai grouping untuk report structure.

### Integration Points
- Report akan standalone (tidak perlu integrasi dengan codebase). Output berupa dokumen markdown.

</code_context>

<specifics>
## Specific Ideas
- Tidak ada referensi spesifik dari user — pendekatan standard dengan hybrid framework yang sudah disepakati.
- Fix suggestions harus konkret dan legally safe (bukan sekedar "ubah kata ini").

</specifics>

<deferred>
## Deferred Ideas
- Audit external sites (docs.esluce.com, app.esluce.com) — akan dilakukan di fase terpisah jika diperlukan
- Full copy rewrite — ditunda ke fase terpisah setelah audit selesai

</deferred>

---

*Phase: 89-Audit Seluruh Copy dan CTA di Landing Page*
*Context gathered: 2026-06-21*
