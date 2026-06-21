---
title: "Compliance Review Report: Landing Page Copy & CTA Audit"
phase: 89
date: 2026-06-21
framework: Hybrid — UU ITE No. 1/2024, UU PDP No. 27/2022, UU Perlindungan Konsumen No. 8/1999, UU Merek No. 20/2016, GDPR (EU) 2016/679, FTC Act §5
classification: Compliance Review (not legal advice)
---

# Compliance Review Report: Landing Page Copy & CTA Audit

## ⚠️ DISCLAIMER

> ⚠️ **DISCLAIMER:** Laporan ini merupakan compliance review yang bersifat informatif dan bukan merupakan legal advice. Untuk keputusan hukum yang mengikat, konsultasikan dengan praktisi hukum berkompeten di yurisdiksi yang relevan.

---

## Executive Summary

This report presents the results of a structured compliance audit of all copy and call-to-action (CTA) elements across the Escluse landing page (`landing-page-escluse/src/`). A hybrid legal framework (UU ITE, UU PDP, UU Perlindungan Konsumen, UU Merek, GDPR, US FTC guidelines) was applied to approximately **200+ copy elements** across **16 source files** (~2,300 lines of code).

### Findings by Risk Tier

| Tier | Count | Description |
|------|-------|-------------|
| 🟢 Safe | ~130 | Legally permissible — standard navigation, factual FAQ, changelog, standard UI labels |
| 🟡 Risky | ~20 | Requires disclaimers, qualification, or caveats |
| 🔴 Must Avoid | 4 | Game trademark references (Minecraft, Rust, Terraria) + trademarked game images |

### Per-Section Breakdown

| Menu Section | 🟢 Safe | 🟡 Risky | 🔴 Must Avoid | Most Significant Risk |
|-------------|---------|---------|---------------|-----------------------|
| Product | ~70 | ~10 | 4 | **Game trademarks** (Minecraft, Rust, Terraria) — ⚠️ Must Avoid |
| Resources | ~15 | 0 | 0 | (Minimal copy — external links only) |
| Company | ~45 | ~10 | 0 | **Entity disclosure** (Resonance Systems), **Date inconsistencies** |

### Key Findings Summary

- **🔴 Game Trademark References (Critical):** The "Supported Games" section uses unqualified game names "Minecraft", "Rust", and "Terraria" — registered trademarks of Mojang AB/Microsoft, Facepunch Studios, and Re-Logic respectively. These MUST be rephrased with qualifiers (e.g., "Minecraft-compatible servers") and accompanied by a trademark disclaimer. Additionally, the game image assets (SVG/PNG files) may contain trademarked artwork requiring modification.
- **🟡 Performance Claims (Medium-High):** "Instant Setup", "in seconds", "instantly deployed", "Make Game Infrastructure Effortless" — absolute performance claims that need substantiation or qualification under UU ITE Pasal 28(1) and FTC Act §5.
- **🟡 Privacy Data Claims (Medium):** "We never sell your personal data" — an absolute commitment requiring legal qualification under UU PDP and GDPR transparency standards.
- **🟡 Security Claims (Medium):** SOC 2 compliance, AES-256, TLS 1.3 — specific technical claims that must be factually accurate regarding Escluse's actual certification status.
- **🟡 Legal Page Date Inconsistency (Low-Medium):** Legal.tsx shows "May 12" for both ToS and Privacy, while the actual ToS and Privacy pages show "May 14".

---

## 🟢 Safe Items

### Product

| # | Copy Element | File:Line | Compliance Reason |
|---|-------------|-----------|-------------------|
| 1 | "Escluse" logo/brand name | `App.tsx:116,174,575` | Standard brand name — no claims made |
| 2 | "EARLY ACCESS" badge | `App.tsx:117-119,175-177` | Industry standard status label — product is genuinely in early development |
| 3 | "Features" nav link | `App.tsx:123,192` | Standard navigation — no claims |
| 4 | "Pricing" nav link | `App.tsx:124,193` | Standard navigation — no claims |
| 5 | "Log in" CTA buttons | `App.tsx:163,221` | Standard CTA — clear action label |
| 6 | "Logout" button | `App.tsx:154,212` | Standard CTA — clear action label |
| 7 | User email display | `App.tsx:146-148` | Session-based user info — no claims |
| 8 | Hero body copy | `App.tsx:258-260` | General value proposition — not a specific measurable claim |
| 9 | "Get Started" CTA (Hero) | `App.tsx:274` | Standard CTA — clear action label |
| 10 | "View Documentation" CTA | `App.tsx:281` | Standard CTA — factual link |
| 11 | "Currently Seeking Partners" | `App.tsx:304` | Standard social proof — not a specific performance claim |
| 12 | "We're Growing" | `App.tsx:307` | Standard social proof — generic, unmeasurable |
| 13 | "Built for Reliability & Scale" (section heading) | `App.tsx:319` | Standard marketing puffery — no specific claim |
| 14 | Feature section description | `App.tsx:320` | General description — no specific measurable claim |
| 15 | "Multi-Server Management" | `App.tsx:353` | Feature name — no performance claim in heading itself |
| 16 | Feature 2 body | `App.tsx:354` | Factual description of functionality |
| 17 | "Real-time Monitoring" | `App.tsx:369` | Feature name — factual (system status monitoring) |
| 18 | Feature 3 body | `App.tsx:370` | Factual description |
| 19 | "Advanced Control Panel" | `App.tsx:385` | Feature name — standard descriptive term |
| 20 | Feature 4 body | `App.tsx:386` | General description |
| 21 | Feature list items (4) | `App.tsx:388` | Factual feature listings — "One-click mod management", "Automated backups", "File access via SFTP", "Scheduled tasks" |
| 22 | "Three Steps to Sovereignty" | `App.tsx:427` | Metaphorical marketing — standard puffery |
| 23 | How It Works section body | `App.tsx:428` | General benefit description |
| 24 | "Connect Nodes" (step 1) | `App.tsx:408-409` | Factual step label |
| 25 | Step 1 description | `App.tsx:409-410` | Factual procedural description |
| 26 | "Configure Games" (step 2) | `App.tsx:412-413` | Factual step label |
| 27 | Step 2 description | `App.tsx:413-414` | Factual procedural description (note: "in seconds" is a separate issue — see Risky) |
| 28 | "Go Live" (step 3) | `App.tsx:416-417` | Factual step label |
| 29 | "Supported Games" section heading | `App.tsx:483` | Standard section heading |
| 30 | Supported Games section body | `App.tsx:484` | General description — "More coming soon" |
| 31 | "Frequently Asked Questions" | `App.tsx:545` | Standard section heading |
| 32 | FAQ 1: "Do I pay for server resources?" | `App.tsx:524-525` | Factual FAQ — clear, accurate answer |
| 33 | FAQ 1 Answer | `App.tsx:525-526` | Factual statement about BYO infrastructure model |
| 34 | FAQ 2: "Can I upgrade anytime?" | `App.tsx:528-529` | Factual FAQ — standard question |
| 35 | FAQ 2 Answer | `App.tsx:529-530` | Standard service promise |
| 36 | FAQ 3: "What services do you support?" | `App.tsx:532-533` | Factual FAQ |
| 37 | FAQ 3 Answer | `App.tsx:533-534` | Factual statement — "currently supports Minecraft" |
| 38 | "Escluse" logo in footer | `App.tsx:575` | Standard brand identification |
| 39 | "Product" footer heading | `App.tsx:583` | Standard navigation heading |
| 40 | Footer nav links (Features, Pricing, Supported Games, Changelog) | `App.tsx:585-588` | Standard navigation — no claims |
| 41 | GitHub social link | `App.tsx:617-618` | Standard social media link |
| 42 | Discord social link | `App.tsx:620-621` | Standard social media link |
| 43 | Changelog page heading | `Changelog.tsx:325` | Factual page identification |
| 44 | Changelog subtitle | `Changelog.tsx:327` | Factual description — no claims |
| 45 | All changelog version entries | `Changelog.tsx:10-255` | Factual version history — no promotional language |
| 46 | Changelog category labels | `Changelog.tsx:289-293` | Standard U/I classification labels ("Added", "Improved", "Fixed", etc.) |
| 47 | "Available Now" badge (Minecraft) | `App.tsx:463` | Factual claim — verifiable (if Minecraft support is genuinely live) |
| 48 | 💡 "Bring your own infrastructure." | `PricingSection.tsx:272` | Clear disclosure of business model |
| 49 | Pricing footnote body | `PricingSection.tsx:274-276` | Factual description — no claims |
| 50 | PlanCard: "Most Popular" badge | `PlanCard.tsx:49-51` | Standard marketing designation — substantiation required but low risk |
| 51 | 🆓 Free plan name | `PlanCard.tsx:63-66` | Standard freemium plan naming |
| 52 | "$0 / month" (Free plan) | `PlanCard.tsx:66` | Factual pricing — clearly $0 |
| 53 | "Current Plan" label | `PlanCard.tsx:147` | Standard status label |
| 54 | "Manage" button | `PlanCard.tsx:153` | Standard CTA — clear action label |
| 55 | "Get Started Free" CTA | `PlanCard.tsx:142` | Standard CTA — "Free" is clearly associated with the Free plan |
| 56 | "Locked" badge on Hobby/Pro | `PlanCard.tsx:58` | Clear indication that plans are not available for subscription |
| 57 | "Coming Soon" badge on Pro | `PlanCard.tsx:58` | Clear status indicator (distinguished from per D-07 "Coming Soon" timing concern) |
| 58 | "Locked" CTA on Hobby card | `PlanCard.tsx:166` | Clear disabled state for unavailable plans |
| 59 | PlanCard feature lists | `PlanCard.tsx:38-75` | Factual feature listings |
| 60 | Onboarding: "Get Started" CTA (step 1) | `Onboarding.tsx:131` | Standard CTA — clear action |
| 61 | Onboarding: "← Back" | `Onboarding.tsx:140` | Standard navigation |
| 62 | Onboarding: "Add Infrastructure" heading (step 2) | `Onboarding.tsx:156` | Factual step label |
| 63 | Onboarding: "Node Name" input label | `Onboarding.tsx:161` | Standard form label |
| 64 | Onboarding: "Generate Command" CTA | `Onboarding.tsx:187` | Standard CTA — clear action |
| 65 | Onboarding: "Install Agent" heading (step 3) | `Onboarding.tsx:206` | Factual step label |
| 66 | Onboarding: Step 3 install command | `Onboarding.tsx:225` | Procedural instruction — no claims |
| 67 | Onboarding: "Agent connected successfully! Redirecting..." | `Onboarding.tsx:237` | Factual status message |
| 68 | Onboarding: "Connection timed out after 2 minutes" | `Onboarding.tsx:243` | Factual error message |
| 69 | Onboarding: "Retry" CTA | `Onboarding.tsx:256` | Standard CTA — clear action |
| 70 | Onboarding: "Infrastructure Ready" (step 4) | `Onboarding.tsx:285` | Factual step label |
| 71 | Onboarding: Step 4 body | `Onboarding.tsx:286` | Factual status message |
| 72 | Onboarding: "Create Game Server" CTA | `Onboarding.tsx:306-309` | Standard CTA — redirects to app.esluce.com |
| 73 | Onboarding: "Add Another Server" CTA | `Onboarding.tsx:315` | Standard secondary CTA |
| 74 | VerifyEmailPage: "Verifying Email" | `VerifyEmailPage.tsx:112` | Factual status label |
| 75 | VerifyEmailPage: "Email Verified!" | `VerifyEmailPage.tsx:120` | Factual success message |
| 76 | VerifyEmailPage: "Verification Failed" | `VerifyEmailPage.tsx:134` | Factual error label |
| 77 | VerifyEmailPage: "Email Not Verified" | `VerifyEmailPage.tsx:157` | Factual status label |
| 78 | VerifyEmailPage: "Resend Verification Email" CTA | `VerifyEmailPage.tsx:184` | Standard CTA — clear action |
| 79 | VerifyEmailPage: "Go to Dashboard" / "Go to Sign In" | `VerifyEmailPage.tsx:126` | Standard CTA |
| 80 | VerifyEmailPage: "Back to Home" | `VerifyEmailPage.tsx:193` | Standard navigation |
| 81 | Pricing heading: "Simple, Transparent Pricing" | `PricingSection.tsx:178` | Standard puffery — see Special Notes (D-08, low priority) |

### Resources

| # | Copy Element | File:Line | Compliance Reason |
|---|-------------|-----------|-------------------|
| 1 | "Docs" nav link | `App.tsx:125,194` | Standard external navigation link |
| 2 | "Resources" footer heading | `App.tsx:593` | Standard navigation heading |
| 3 | "Documentation" footer link | `App.tsx:595` | Standard external link to docs.esluce.com |
| 4 | "API Reference" footer link | `App.tsx:596` | Standard placeholder link (href="#") |
| 5 | "Community Forum" footer link | `App.tsx:597` | Standard external link to Discord |
| 6 | "Security" footer link (auth footer) | `components/auth/Footer.tsx:14` | Standard navigation |
| 7 | "Status" footer link (auth footer) | `components/auth/Footer.tsx:15` | Standard navigation |
| 8 | "Back to Home" CTA (Onboarding) | `Onboarding.tsx:140` | Standard navigation |

### Company

| # | Copy Element | File:Line | Compliance Reason |
|---|-------------|-----------|-------------------|
| 1 | "About Us" footer link | `App.tsx:604` | Standard navigation |
| 2 | "Legal" footer link | `App.tsx:606` | Standard navigation |
| 3 | "Contact" footer link | `App.tsx:607` | Standard navigation |
| 4 | "About Us" page heading | `AboutUs.tsx:18` | Standard page identification |
| 5 | "Our Mission" heading | `AboutUs.tsx:26` | Standard page subheading — mission statement |
| 6 | Mission body | `AboutUs.tsx:27-29` | General mission statement — no specific claims |
| 7 | "Why We Build This" heading | `AboutUs.tsx:31` | Standard page subheading |
| 8 | Why body | `AboutUs.tsx:32-34` | General narrative — no specific claims |
| 9 | "Contact" page heading | `Contact.tsx:18` | Standard page identification |
| 10 | Contact intro text | `Contact.tsx:22` | Standard invitation — no claims |
| 11 | "Email" section heading | `Contact.tsx:27` | Standard section label |
| 12 | Email description | `Contact.tsx:28` | Factual description |
| 13 | "admin@esluce.com" email (multiple pages) | `Contact.tsx:29, AboutUs.tsx:41, TermsOfService.tsx:78, PrivacyPolicy.tsx:68` | Standard business contact information |
| 14 | "Discord" section heading | `Contact.tsx:33` | Standard section label |
| 15 | Discord description | `Contact.tsx:34` | Factual description |
| 16 | Discord link | `Contact.tsx:35-37` | Standard external link |
| 17 | "Back to Home" (Contact page) | `Contact.tsx:16-17` | Standard navigation |
| 18 | "Back to Home" (Legal page) | `Legal.tsx:16-17` | Standard navigation |
| 19 | "Back to Home" (About Us) | `AboutUs.tsx:16-17` | Standard navigation |
| 20 | "Back to Home" (Changelog) | `Changelog.tsx:16-17` | Standard navigation |
| 21 | "Back to Home" (Terms) | `TermsOfService.tsx:16-17` | Standard navigation |
| 22 | "Back to Home" (Privacy) | `PrivacyPolicy.tsx:16-17` | Standard navigation |
| 23 | "Back to Home" (Security) | `Security.tsx:16-17` | Standard navigation |
| 24 | "Back to Home" (Status) | `Status.tsx:16-17` | Standard navigation |
| 25 | "Legal" page heading | `Legal.tsx:18` | Standard page identification |
| 26 | "Terms of Service" (Legal.tsx section heading) | `Legal.tsx:22` | Standard section label |
| 27 | "Privacy Policy" (Legal.tsx section heading) | `Legal.tsx:31` | Standard section label |
| 28 | "License" (Legal.tsx section heading) | `Legal.tsx:40` | Standard section label |
| 29 | Sign In page: "Welcome Back" heading | `SignIn.tsx:57` | Standard page greeting |
| 30 | Sign In: subtitle | `SignIn.tsx:58` | Standard instruction text |
| 31 | Sign In: OAuth buttons (Google, GitHub, Discord) | `SignIn.tsx:73,84,94` | Standard OAuth authorization CTAs |
| 32 | Sign In: "OR" divider | `SignIn.tsx:104` | Standard U/I divider |
| 33 | Sign In: "Email address" / "Password" labels | `SignIn.tsx:116,131` | Standard form labels |
| 34 | Sign In: "Remember me" checkbox | `SignIn.tsx:157` | Standard form option |
| 35 | Sign In: "Forgot password?" link | `SignIn.tsx:160-161` | Standard password recovery link |
| 36 | Sign In: "Sign In" CTA | `SignIn.tsx:170` | Standard CTA — clear action |
| 37 | Sign In: "Don't have an account? Sign up" link | `SignIn.tsx:184-193` | Standard cross-navigation |
| 38 | Sign Up: "Create Account" heading | `SignUp.tsx:61` | Standard page heading |
| 39 | Sign Up: "Join Escluse to get started" | `SignUp.tsx:62` | Standard welcome text — no claims |
| 40 | Sign Up: OAuth buttons | `SignUp.tsx:77` | Standard OAuth CTAs |
| 41 | Sign Up: "Full Name" label | `SignUp.tsx:120` | Standard form label |
| 42 | Sign Up: "Create Account" CTA | `SignUp.tsx:178` | Standard CTA |
| 43 | Sign Up: "Already have an account? Sign in" | `SignUp.tsx:184-193` | Standard cross-navigation |
| 44 | Legal notice (Sign In): "By signing in, you agree to our Terms of Service and Privacy Policy." | `SignIn.tsx:54` | Standard legal notice for auth forms |
| 45 | Legal notice (Sign Up): "By signing up, you agree to our Terms of Service and Privacy Policy." | `SignUp.tsx:30` | Standard legal notice for auth forms |
| 46 | Auth Footer: "Escluse" brand name | `components/auth/Footer.tsx:9` | Standard brand identification |
| 47 | Auth Footer: "Privacy Policy" link | `components/auth/Footer.tsx:12` | Standard navigation |
| 48 | Auth Footer: "Terms of Service" link | `components/auth/Footer.tsx:13` | Standard navigation |
| 49 | Auth Footer: copyright | `components/auth/Footer.tsx:18` | Standard copyright notice |
| 50 | "System Status" page heading | `Status.tsx:46` | Standard page identification |
| 51 | "Last updated: May 14, 2026" (Status) | `Status.tsx:47` | Factual metadata |
| 52 | "All Systems Operational" | `Status.tsx:53` | Dynamic system status — must be factually accurate at time of display |
| 53 | Status page: "Back to Home" | `Status.tsx:16-17` | Standard navigation |
| 54 | "Want to receive status updates?..." CTA | `Status.tsx:72` | Standard informational CTA |
| 55 | Security page heading | `Security.tsx:18` | Standard page identification |
| 56 | "Authentication" section (OAuth, MFA) | `Security.tsx:37-40` | Factual description of auth methods |
| 57 | "Monitoring & Response" section | `Security.tsx:57-60` | Standard security practice description |
| 58 | "Report a Vulnerability" section | `Security.tsx:63-67` | Standard security disclosure practice |
| 59 | "Contact Us" page heading | `Contact.tsx:18` | Standard page identification |
| 60 | "Security" link (Sign In footer) | `SignIn.tsx:57` | Standard navigation link |
| 61 | "Status" link (Sign In footer) | `SignIn.tsx:59` | Standard navigation link |
| 62 | "Docs" link (Sign In footer) | `SignIn.tsx:61` | Standard external link |
| 63 | Legal: Terms of Service body (Legal.tsx) | `Legal.tsx:25-26` | Standard summary of legal terms |
| 64 | Privacy Policy heading | `PrivacyPolicy.tsx:18` | Standard page identification |
| 65 | "Information We Collect" section | `PrivacyPolicy.tsx:23-30` | Standard data collection disclosure |
| 66 | "How We Use Your Data" section | `PrivacyPolicy.tsx:34-42` | Standard data use disclosure |
| 67 | "Cookies" section | `PrivacyPolicy.tsx:51-53` | Standard cookie notice |
| 68 | "Your Rights" section | `PrivacyPolicy.tsx:56-63` | Standard data rights disclosure (access, correction, deletion, export) |
| 69 | Terms of Service: "Acceptance of Terms" | `TermsOfService.tsx:23-24` | Standard legal clause |
| 70 | Terms of Service: "Description of Service" | `TermsOfService.tsx:28-30` | Standard service description |
| 71 | Terms of Service: "Your Responsibilities" | `TermsOfService.tsx:33-41` | Standard responsibilities clause |
| 72 | Terms of Service: "Prohibited Activities" | `TermsOfService.tsx:45-53` | Standard prohibited use clause |
| 73 | Terms of Service: "Fees and Billing" | `TermsOfService.tsx:62-64` | Standard billing terms |
| 74 | Terms of Service: "Changes to Terms" | `TermsOfService.tsx:72-74` | Standard terms modification clause |
| 75 | Terms of Service: "Contact" | `TermsOfService.tsx:78` | Standard contact information |
| 76 | "Report a Vulnerability" — email link | `Security.tsx:65-67` | Standard contact for security disclosure |

---

## 🟡 Risky Items

### Product

---

### Tagline/Badge: "Your Infrastructure. Your Control." — 🟡 Risky

**File:** `App.tsx:252`
**Current text:** "Your Infrastructure. Your Control."
**Legal provisions:** UU ITE No. 1/2024 Pasal 28(1) (informasi menyesatkan — implied performance claim); FTC Act §5 (deceptive acts or practices)
**Risk assessment:**
  - Enforcement likelihood: Low
  - Reputational impact: Low-Medium
  - Platform policy implications: Minimal
  - Risk severity: Low
**Diagnosis:** While this tagline is primarily branding, it implies that users will have full control over their infrastructure. If the service limits control in practice (e.g., locked features on Free plan, "Locked" badges on Hobby/Pro), there may be a minor tension between this claim and the actual user experience. Low risk — primarily puffery.
**Fix suggestion:** No change strictly necessary. If revising, ensure "Your Control" is not contradicted by any locked/unavailable features.

---

### Hero Headline: "Make Game Infrastructure Effortless" — 🟡 Risky

**File:** `App.tsx:254-257`
**Current text:** "Make Game Infrastructure Effortless" (with "Effortless" in gradient styling)
**Legal provisions:** UU ITE No. 1/2024 Pasal 28(1) — dilarang menyebarkan informasi menyesatkan yang mengakibatkan kerugian konsumen; UU Perlindungan Konsumen No. 8/1999 Pasal 8 — pelaku usaha dilarang memproduksi/memperdagangkan barang/jasa yang tidak sesuai janji; FTC Act §5 and FTC Advertising Substantiation Policy Statement — must have reasonable basis for performance claims
**Risk assessment:**
  - Enforcement likelihood: Low-Medium
  - Reputational impact: Medium (if users find setup not "effortless")
  - Platform policy implications: Low
  - Risk severity: Medium
**Diagnosis:** "Effortless" is a strong absolute claim about ease of use. While it may be interpreted as puffery, combined with other speed claims ("Instant Setup", "in seconds", "instantly deployed"), it creates a pattern of unqualified performance promises. Under UU ITE Pasal 28(1) and FTC substantiation doctrine, these claims must have a reasonable basis. If a user finds the setup process complex (e.g., needing to configure a VPS, install agent, troubleshoot connections), the claim could be misleading.
**Fix suggestion:** Replace "Effortless" with "Simplify" or add a qualifier. Suggested: "Simplify Game Infrastructure Management" or "Game Infrastructure, Simplified."

---

### Feature Heading: "Instant Setup" + "in seconds" — 🟡 Risky

**File:** `App.tsx:335-336`
**Current text:** "Instant Setup" / "Set up and manage your infrastructure in seconds."
**Legal provisions:** UU ITE No. 1/2024 Pasal 28(1); FTC Advertising Substantiation Policy Statement — temporal/performance claims require substantiation; UU Perlindungan Konsumen No. 8/1999 Pasal 8 — tidak sesuai janji
**Risk assessment:**
  - Enforcement likelihood: Medium (temporal claims are frequently challenged)
  - Reputational impact: Medium-High (users have concrete expectations about setup time)
  - Platform policy implications: Google Ads, Meta Ads may reject "instant" claims
  - Risk severity: Medium
**Diagnosis:** "Instant Setup" and "in seconds" are absolute temporal claims. Actual setup involves: connecting a VPS/machine, running an install command, waiting for agent connection (up to 2 minutes per the polling timeout in Onboarding.tsx), and configuring games. This is demonstrably not "instant" — it takes at minimum several minutes. This is the most actionable Risky finding in the Product section.
**Fix suggestion:** "Quick Setup" or "Setup in Minutes" with a link to documentation showing expected setup time. Change body text to: "Quickly set up your infrastructure with Escluse's automated configuration — typically ready in minutes."

---

### How It Works Step 3: "instantly deployed" — 🟡 Risky

**File:** `App.tsx:418-419`
**Current text:** "Your server is instantly deployed, monitored, and ready for players."
**Legal provisions:** UU ITE No. 1/2024 Pasal 28(1); FTC Act §5; UU Perlindungan Konsumen No. 8/1999 Pasal 10 — dilarang menawarkan/mempromosikan secara menyesatkan
**Risk assessment:**
  - Enforcement likelihood: Low-Medium
  - Reputational impact: Medium
  - Platform policy implications: Low
  - Risk severity: Medium
**Diagnosis:** "Instantly deployed" is another absolute speed claim. Actual server deployment depends on game type, server configuration, and infrastructure readiness. The word "instantly" sets an unrealistic expectation that could lead to consumer disappointment and complaints.
**Fix suggestion:** "Your server is deployed and monitored, ready for players." (Remove "instantly" — the rest of the sentence is accurate without it.)

---

### Feature Body: "in seconds" (How It Works Step 2) — 🟡 Risky

**File:** `App.tsx:413-414`
**Current text:** "Choose your game, customize settings, and install mods in seconds."
**Legal provisions:** Same as above — UU ITE Pasal 28(1), FTC substantiation
**Risk assessment:**
  - Enforcement likelihood: Low-Medium
  - Reputational impact: Low-Medium
  - Platform policy implications: Low
  - Risk severity: Low-Medium
**Diagnosis:** "In seconds" for customizing game settings and installing mods overstates the speed. Users may need to browse templates, customize configurations, and wait for mod downloads.
**Fix suggestion:** "Choose your game, customize settings, and install mods quickly and easily."

---

### FAQ: "enterprise grade security" — 🟡 Risky

**File:** `App.tsx:537-538`
**Current text:** "The Escluse platform itself remains protected by enterprise grade security and mitigation systems."
**Legal provisions:** FTC Advertising Substantiation Policy Statement — "enterprise grade" is a specific quality claim requiring substantiation; UU ITE No. 1/2024 Pasal 28(1); UU PDP No. 27/2022 Pasal 35 — keamanan data
**Risk assessment:**
  - Enforcement likelihood: Low-Medium
  - Reputational impact: Medium (implies institutional-level security)
  - Platform policy implications: Low
  - Risk severity: Medium
**Diagnosis:** "Enterprise grade" is a qualitative claim that implies a specific level of security sophistication comparable to large enterprises. If pressed for substantiation, Escluse would need to demonstrate what makes its security "enterprise grade" — specific certifications, standards, protocols, or architecture. The term is overused in marketing but carries legal weight when making security promises.
**Fix suggestion:** "The Escluse platform remains protected by robust security and mitigation systems." Drop "enterprise grade" unless it can be substantiated.

---

### "Coming Soon" badges (Rust and Terraria) — 🟡 Risky

**File:** `App.tsx:469,475`
**Current text:** Badge: "Coming Soon" (on Rust and Terraria game cards)
**Legal provisions:** UU Perlindungan Konsumen No. 8/1999 Pasal 10 — dilarang menawarkan/mempromosikan secara menyesatkan; FTC Dot Com Disclosures — clear and conspicuous disclosure requirements
**Risk assessment:**
  - Enforcement likelihood: Low
  - Reputational impact: Low-Medium (users may feel misled if "soon" becomes months)
  - Platform policy implications: Low
  - Risk severity: Low-Medium
**Diagnosis:** "Coming Soon" is inherently vague — it creates an expectation that the feature will be available in the near future. If development timelines are uncertain, this could be considered misleading under consumer protection law. Risk increases proportionally with time elapsed.
**Fix suggestion:** Replace "Coming Soon" with "In Development" or "Planned" — these are less time-bound and set more accurate expectations. Alternatively, add a timeframe: "Planned for Q3 2026."

---

### "Available Now" badge (Minecraft) — 🟡 Risky

**File:** `App.tsx:463`
**Current text:** Badge: "Available Now" (on Minecraft game card)
**Legal provisions:** UU Perlindungan Konsumen No. 8/1999 Pasal 8
**Risk assessment:**
  - Enforcement likelihood: Low
  - Reputational impact: Low-Medium (if Minecraft deployment is limited or broken)
  - Platform policy implications: Low
  - Risk severity: Low
**Diagnosis:** "Available Now" must be factually accurate — Minecraft deployment must actually work for all users who can access it. If there are restrictions (plan limits, beta access, limited regions), the claim could be misleading.
**Fix suggestion:** Keep as-is if Minecraft deployment is genuinely available to all users. If there are restrictions, add a qualifier like "Available on paid plans."

---

### Onboarding: "Deploy your first game server in minutes." — 🟡 Risky

**File:** `Onboarding.tsx:122`
**Current text:** "Deploy your first game server in minutes."
**Legal provisions:** FTC substantiation doctrine; UU ITE Pasal 28(1)
**Risk assessment:**
  - Enforcement likelihood: Low
  - Reputational impact: Low-Medium
  - Platform policy implications: Low
  - Risk severity: Low
**Diagnosis:** "In minutes" is a temporal claim. Actual time to first deployment includes: entering node name (step 2), generating command, running install script, waiting for agent connection (up to 2 min), then creating a game server. This could be "in minutes" but is more nuanced than the headline suggests. Low risk given the complexity of the process.
**Fix suggestion:** Add a qualifier: "Deploy your first game server in minutes — just add your hardware and run one command."

---

### Pricing Body: "no hidden costs" — 🟡 Risky

**File:** `PricingSection.tsx:181-182`
**Current text:** "no hidden costs, no infrastructure lock-in."
**Legal provisions:** UU Perlindungan Konsumen No. 8/1999 Pasal 10; FTC Act §5; FTC Dot Com Disclosures
**Risk assessment:**
  - Enforcement likelihood: Low (per D-08, standard marketing scrutiny)
  - Reputational impact: Low
  - Platform policy implications: Low
  - Risk severity: Low
**Diagnosis:** "No hidden costs" is an absolute negative claim. If any fees or costs exist beyond the stated pricing (e.g., transaction fees, usage-based charges, mandatory add-ons), this claim would be misleading. Per D-08, this receives standard marketing scrutiny only — no deep analysis required. Noted for completeness.
**Fix suggestion:** Keep as-is but verify there are genuinely no undisclosed costs. Add a link to full pricing details if desired.

---

### Savings Claim: "Save ~17% with yearly billing" — 🟡 Risky

**File:** `PricingSection.tsx:191-193`
**Current text:** "Save ~17% with yearly billing"
**Legal provisions:** UU Perlindungan Konsumen No. 8/1999 Pasal 8; FTC Act §5 — savings claims must be substantiated
**Risk assessment:**
  - Enforcement likelihood: Low
  - Reputational impact: Low
  - Platform policy implications: Low
  - Risk severity: Low
**Diagnosis:** Savings claims require substantiation — the ~17% figure must be an accurate comparison between monthly and yearly pricing. Per D-08, this is standard marketing scrutiny.
**Fix suggestion:** Keep as-is if the calculation is accurate (yearly price vs 12× monthly price). Include a brief footnote showing the calculation for transparency.

---

### "🆓 Free" Plan / "Get Started Free" — 🟡 Risky

**File:** `PlanCard.tsx:63-66, 142`
**Current text:** "🆓 Free" / "$0 / month" / "Get Started Free"
**Legal provisions:** FTC Guides Against Deceptive Pricing; UU Perlindungan Konsumen No. 8/1999 Pasal 10
**Risk assessment:**
  - Enforcement likelihood: Low
  - Reputational impact: Low
  - Platform policy implications: Low
  - Risk severity: Low
**Diagnosis:** "Free" must genuinely be free with no hidden obligations or time limits. The Free plan appears to be genuinely $0/month. Per D-08, standard scrutiny only.
**Fix suggestion:** Ensure the Free plan remains genuinely free with no automatic conversion to paid.

---

### Onboarding → "Create Game Server" redirects to external CTA — 🟡 Risky

**File:** `Onboarding.tsx:306-309`
**Current text:** "Create Game Server" (redirects to `https://app.esluce.com`)
**Legal provisions:** Standard CTA concern — ensure action matches label
**Risk assessment:**
  - Enforcement likelihood: Low
  - Reputational impact: Low
  - Platform policy implications: Low
  - Risk severity: Low
**Diagnosis:** The CTA "Create Game Server" on the Escluse landing page redirects to app.esluce.com (external dashboard). This is functionally correct — users create servers in the dashboard. Low risk but worth noting that users may expect inline action.
**Fix suggestion:** No change needed for current scope. If redesigning, consider making this a same-origin redirect or clarifying the destination.

---

### Company

---

### "Resonance Systems" Entity Claim — 🟡 Risky

**File:** `AboutUs.tsx:23`, `App.tsx:614`
**Current text:** "Escluse is a product of Resonance Systems — a technology company focused on building tools that make game server management accessible to everyone." / "© 2026 Resonance Systems. All rights reserved. Escluse is a product of Resonance Systems."
**Legal provisions:** UU Perseroan Terbatas No. 40/2007 — entity disclosure requirements; UU Cipta Kerja No. 11/2020; UU ITE No. 1/2024 — identitas pelaku usaha
**Risk assessment:**
  - Enforcement likelihood: Low-Medium (if "Resonance Systems" is not a registered entity)
  - Reputational impact: Medium
  - Platform policy implications: Low
  - Risk severity: Medium
**Diagnosis:** The footer and About Us page present "Resonance Systems" as the company behind Escluse. If "Resonance Systems" is not registered as a formal business entity (PT, CV, or equivalent), the corporate representation could be misleading under Indonesian business law. The absence of PT/CV designation may be noted by regulators or business partners.
**Fix suggestion:** If Resonance Systems is registered: add the entity designation (e.g., "PT Resonance Systems" or "CV Resonance Systems"). If not registered: use "a brand of" or "a product by" to avoid implying corporate entity status.

---

### "Founded by Reyhan Buztanil." — 🟡 Risky

**File:** `AboutUs.tsx:19`
**Current text:** "Founded by Reyhan Buztanil."
**Legal provisions:** UU ITE No. 1/2024 — accuracy of personal information; UU PDP No. 27/2022 — personal data processing
**Risk assessment:**
  - Enforcement likelihood: Low
  - Reputational impact: Low
  - Platform policy implications: Low
  - Risk severity: Low
**Diagnosis:** Personal name attribution requires the individual's consent and must be factually accurate. With the founder's name publicly associated, this is low risk but should be verified for consent.
**Fix suggestion:** Keep as-is. Ensure Reyhan Buztanil has consented to this public attribution.

---

### "We never sell your personal data" — 🟡 Risky

**File:** `Legal.tsx:35`, `PrivacyPolicy.tsx:47`
**Current text:** "We do not sell your personal data." / "We never sell your personal data to third parties."
**Legal provisions:** UU PDP No. 27/2022 Pasal 5 — prinsip transparansi; UU PDP Pasal 20-24 — dasar pemrosesan data; GDPR Art. 5(1)(a) — lawfulness, fairness, transparency; GDPR Art. 12-14 — transparency obligations
**Risk assessment:**
  - Enforcement likelihood: Medium (absolute data practice claims are scrutinized)
  - Reputational impact: Medium-High (if ever found to share data with third parties for consideration)
  - Platform policy implications: App Store, Google Play policies require accurate data practices disclosure
  - Risk severity: Medium
**Diagnosis:** "Never sell" is an absolute commitment that carries significant legal weight. Under UU PDP and GDPR, "selling" personal data has specific definitions. If Escluse shares data with analytics providers (Umami, etc.) or uses third-party services that process data, it must be clear this does not constitute "selling" under applicable law. The Legal.tsx version ("We do not sell") is paired with "we only process it to deliver the service" — this qualification is important.
**Fix suggestion:** Change to: "We do not sell your personal data as defined under applicable data protection laws. Your data is only processed to deliver the service you request." This adds the legal qualification while maintaining the commitment.

---

### Limitation of Liability Clause — 🟡 Risky

**File:** `TermsOfService.tsx:67-69`
**Current text:** "Escluse is not responsible for any indirect, incidental, or consequential damages arising from the use of our service. Your use of the platform is at your own risk."
**Legal provisions:** UU Perlindungan Konsumen No. 8/1999 Pasal 18 — klausula baku yang merugikan konsumen batal demi hukum; UU ITE No. 1/2024
**Risk assessment:**
  - Enforcement likelihood: Medium (Pasal 18 is enforceable — standard clauses against consumer interests are void by law)
  - Reputational impact: Low-Medium
  - Platform policy implications: Low
  - Risk severity: Medium
**Diagnosis:** Under UU Perlindungan Konsumen Pasal 18, standard clauses (klausula baku) that shift liability away from the business to the detriment of consumers are void by operation of law. The current liability clause is broad and could be challenged if Escluse's service causes direct damages to a consumer's business. The clause lacks the standard saving language.
**Fix suggestion:** Add: "to the fullest extent permitted by applicable law" at the beginning of the liability clause. Consider distinguishing between consumers (where Pasal 18 applies) and business users. Example: "To the fullest extent permitted by applicable law, Escluse shall not be liable for any indirect, incidental, or consequential damages..."

---

### Service Availability Clause — 🟡 Risky

**File:** `TermsOfService.tsx:58`
**Current text:** "Escluse strives to maintain high availability but does not guarantee 100% uptime."
**Legal provisions:** UU Perlindungan Konsumen No. 8/1999 Pasal 8 — tidak sesuai janji; UU ITE No. 1/2024
**Risk assessment:**
  - Enforcement likelihood: Low
  - Reputational impact: Low-Medium (if service disruptions affect users)
  - Platform policy implications: Low
  - Risk severity: Low-Medium
**Diagnosis:** "Strives to maintain high availability" is reasonable language. The explicit disclaimer about 100% uptime is good practice. However, "high availability" is not defined — what constitutes "high" could be interpreted differently. Adding a specific metric would strengthen the clause.
**Fix suggestion:** Define what "high availability" means (e.g., "99.9% uptime target") or remove the term entirely. Current language is adequate but could be more precise.

---

### AGPLv3 License Statement — 🟡 Risky

**File:** `Legal.tsx:41`
**Current text:** "Infrastructure configurations are available under AGPLv3."
**Legal provisions:** AGPLv3 — Affero General Public License version 3 obligations; UU Hak Cipta No. 28/2014
**Risk assessment:**
  - Enforcement likelihood: Medium (AGPLv3 has specific obligations that must be stated clearly)
  - Reputational impact: Medium
  - Platform policy implications: Low
  - Risk severity: Medium
**Diagnosis:** The current statement is ambiguous — "Infrastructure configurations" is not a clearly defined scope. Under AGPLv3, users who modify and use the software over a network must make the source code available. The scope of what is AGPLv3-licensed vs. proprietary should be clearly delineated to avoid confusion and potential license violation claims.
**Fix suggestion:** "The Escluse relay agent and related infrastructure tooling are open source under AGPLv3. The Escluse platform, dashboard, and cloud services remain proprietary. See individual repositories for specific license terms."

---

### Security Claims: "SOC 2 compliance" — 🟡 Risky

**File:** `Security.tsx:49`
**Current text:** "Our infrastructure runs on secure cloud providers with SOC 2 compliance."
**Legal provisions:** FTC Act §5 — substantiation requirement for specific claims; UU ITE No. 1/2024 Pasal 28(1); UU PDP No. 27/2022 Pasal 35 — keamanan data
**Risk assessment:**
  - Enforcement likelihood: Medium (SOC 2 is a specific, auditable certification)
  - Reputational impact: High (false SOC 2 claims would be a significant credibility issue)
  - Platform policy implications: Enterprise procurement often requires SOC 2 verification
  - Risk severity: Medium
**Diagnosis:** The current phrasing says "cloud providers with SOC 2 compliance" — implying the cloud provider (e.g., AWS, GCP, Azure) has SOC 2, not Escluse itself. This is technically accurate if Escluse runs on SOC 2-compliant infrastructure. However, the proximity of "our infrastructure" and "SOC 2 compliance" could be read as Escluse being SOC 2 certified. The wording should be clarified.
**Fix suggestion:** No change to the factual statement — it correctly attributes SOC 2 to cloud providers. However, consider adding explicit clarification: "We run on cloud providers with SOC 2 compliance (e.g., AWS/GCP)" to remove ambiguity. See open question in RESEARCH.md — whose SOC 2 is this?

---

### Security Claims: "TLS 1.3" and "AES-256" — 🟡 Risky

**File:** `Security.tsx:29`
**Current text:** "All data is encrypted in transit using TLS 1.3 and encrypted at rest using AES-256."
**Legal provisions:** FTC Act §5 — specific technical claims must be accurate; UU ITE No. 1/2024 Pasal 28(1)
**Risk assessment:**
  - Enforcement likelihood: Medium (specific technical claims are verifiable)
  - Reputational impact: High (if found to be inaccurate)
  - Platform policy implications: Low
  - Risk severity: Medium
**Diagnosis:** These are specific, verifiable technical claims. If Escluse does not use TLS 1.3 for all data in transit, or AES-256 for all data at rest, this claim would be misleading. The claim is binary — either it's true or it's not. Given Escluse's infrastructure model (BYO hardware), it's important to clarify which parts of the system use these encryption standards.
**Fix suggestion:** Add context: "All data transmitted between the Escluse platform and your server is encrypted in transit using TLS 1.3 and encrypted at rest using AES-256." Clarify the scope of "all data" — which is Escluse's platform data, not the user's game server data on their own hardware.

---

### "Response Time: We typically respond within 24-48 hours" — 🟡 Risky

**File:** `Contact.tsx:43-45`
**Current text:** "We typically respond within 24-48 hours. For urgent issues, join our Discord for faster support."
**Legal provisions:** UU Perlindungan Konsumen No. 8/1999 Pasal 8 — tidak sesuai janji; FTC Act §5
**Risk assessment:**
  - Enforcement likelihood: Low
  - Reputational impact: Low-Medium (if response times are consistently outside this window)
  - Platform policy implications: Low
  - Risk severity: Low
**Diagnosis:** "24-48 hours" is a specific service level claim. If response is not consistently within this window, the claim could be misleading under consumer protection law. The use of "typically" provides some flexibility but still sets an expectation.
**Fix suggestion:** Change to: "We aim to respond within 24-48 hours." This adds the "aim to" qualifier, making it a commitment of effort rather than a guaranteed service level.

---

### Date Inconsistency: Legal.tsx vs Privacy Policy/Terms — 🟡 Risky

**File:** `Legal.tsx:23, 32` vs `TermsOfService.tsx:19`, `PrivacyPolicy.tsx:19`
**Current text:** Legal.tsx shows "Last updated: May 12, 2026" for both ToS and Privacy; TermsOfService.tsx and PrivacyPolicy.tsx pages both show "Last updated: May 14, 2026"
**Legal provisions:** UU ITE No. 1/2024; UU Perlindungan Konsumen No. 8/1999 — accuracy of information; Good faith principle
**Risk assessment:**
  - Enforcement likelihood: Low
  - Reputational impact: Low-Medium (undermines trust in legal documentation accuracy)
  - Platform policy implications: Low
  - Risk severity: Low
**Diagnosis:** The Legal.tsx summary page shows "May 12, 2026" for both the Terms of Service and Privacy Policy, while the dedicated pages for both documents show "May 14, 2026." This inconsistency undermines the credibility of the legal documentation. In a dispute, inconsistent dates could be used to question the accuracy of the legal information presented.
**Fix suggestion:** Standardize all "Last updated" dates across Legal.tsx, TermsOfService.tsx, and PrivacyPolicy.tsx to the same date (the actual date of the last review). Ensure dates are factually accurate.

---

### Footer Copyright Without Entity Designation — 🟡 Risky

**File:** `components/auth/Footer.tsx:18`, `App.tsx:614`
**Current text:** "© 2026 Resonance Systems. All rights reserved." / "© 2026 Resonance Systems. All rights reserved. Escluse is a product of Resonance Systems."
**Legal provisions:** UU Perseroan Terbatas; UU Hak Cipta No. 28/2014
**Risk assessment:**
  - Enforcement likelihood: Low
  - Reputational impact: Low
  - Platform policy implications: Low
  - Risk severity: Low
**Diagnosis:** The copyright notice attributes to "Resonance Systems" without specifying whether this is a legal entity, DBA, or brand name. Standard practice is to use the registered legal entity name. This is a low-severity issue but related to the larger entity disclosure concern.
**Fix suggestion:** If Resonance Systems is a registered entity, use the full legal name (e.g., "PT Resonance Systems"). If not, consider alternative copyright attribution.

---

## 🔴 Must Avoid Items

### Product — Specifically Supported Games Section

---

### "Minecraft" Game Name — 🔴 Must Avoid

**File:** `App.tsx:460`
**Current text:** "Minecraft" (game card name in Supported Games section)
**Legal provisions:** UU Merek No. 20/2016 Pasal 21 — perlindungan merek terdaftar; Pasal 100 — pelanggaran merek (pidana penjara maksimal 4 tahun dan/atau denda maksimal Rp2 miliar); US Lanham Act 15 U.S.C. §1114 — trademark infringement; EU Trademark Directive (EU) 2015/2436
**Risk assessment:**
  - Enforcement likelihood: Medium-High (Microsoft actively protects the Minecraft trademark through legal action and platform policies)
  - Reputational impact: High (using a registered trademark without authorization suggests official affiliation)
  - Platform policy implications: High (Steam, Xbox, and cloud marketplaces prohibit unauthorized trademark use; could affect business partnerships)
  - Risk severity: Critical
**Diagnosis:** Minecraft is a registered trademark of Mojang AB (a subsidiary of Microsoft). Using the exact trademarked name "Minecraft" on a commercial service without qualifying language (e.g., "compatible with", "for") implies official affiliation, endorsement, or partnership. This is a textbook trademark concern under both Indonesian and international law. The use of the game name alongside the Minecraft grass block SVG logo on the same card intensifies the implied affiliation.
**Fix suggestion:** Replace "Minecraft" with "Minecraft-compatible servers" throughout the Supported Games section. Add a prominent disclaimer: "Minecraft is a trademark of Mojang AB. Escluse is not affiliated with, endorsed by, or sponsored by Mojang AB or Microsoft." Address the game image asset (minecraft-1.svg) in the rewrite phase per D-10.

---

### "Rust" Game Name — 🔴 Must Avoid

**File:** `App.tsx:466`
**Current text:** "Rust" (game card name in Supported Games section)
**Legal provisions:** UU Merek No. 20/2016 Pasal 21, Pasal 100; US Lanham Act 15 U.S.C. §1114; EU Trademark Directive (EU) 2015/2436
**Risk assessment:**
  - Enforcement likelihood: Medium-High (Facepunch Studios actively protects the Rust trademark; has pursued legal action against unauthorized use)
  - Reputational impact: High
  - Platform policy implications: High
  - Risk severity: Critical
**Diagnosis:** Rust is a registered trademark of Facepunch Studios. Using the exact game name "Rust" on a commercial service — even with a "Coming Soon" badge — implies a relationship with Facepunch that does not exist. Trademark owners are obligated to enforce their marks, and non-enforcement can lead to loss of trademark protection.
**Fix suggestion:** Replace "Rust" with "Rust-compatible game hosting" or "Survival game servers." Add the general disclaimer about no affiliation with game trademark owners. Address the "icons8-rust.svg" game image asset in the rewrite phase.

---

### "Terraria" Game Name — 🔴 Must Avoid

**File:** `App.tsx:472`
**Current text:** "Terraria" (game card name in Supported Games section)
**Legal provisions:** UU Merek No. 20/2016 Pasal 21, Pasal 100; US Lanham Act 15 U.S.C. §1114; EU Trademark Directive (EU) 2015/2436
**Risk assessment:**
  - Enforcement likelihood: Medium (Re-Logic has historically been less aggressive than Microsoft/Facepunch but still protects their trademark)
  - Reputational impact: High
  - Platform policy implications: High
  - Risk severity: Critical
**Diagnosis:** Terraria is a registered trademark of Re-Logic. Same trademark concern as Minecraft and Rust — unauthorized use of a registered trademark on a commercial service without qualifying language implies affiliation.
**Fix suggestion:** Replace "Terraria" with "Terraria-compatible servers" or "Sandbox game servers." Add the general disclaimer about no affiliation with game trademark owners. Address the "terraria-logo.png" game image asset in the rewrite phase.

---

### Game Asset Images (SVG/PNG) — 🔴 Must Avoid

**File:** `App.tsx:461 (minecraft-1.svg)`, `App.tsx:467 (icons8-rust.svg)`, `App.tsx:473 (terraria-logo.png)`
**Current text:** Image references in the Supported Games section — game-specific logo/icon assets
**Legal provisions:** UU Merek No. 20/2016; UU Hak Cipta No. 28/2014; US Lanham Act; EU Trademark Directive
**Risk assessment:**
  - Enforcement likelihood: Medium-High (trademarked logos are even more protected than text marks)
  - Reputational impact: High
  - Platform policy implications: High
  - Risk severity: Critical
**Diagnosis:** In addition to the game name text references, the game cards display trademarked/copyrighted artwork: Minecraft grass block icon (minecraft-1.svg), Rust icon (icons8-rust.svg), and Terraria logo (terraria-logo.png). These visual assets are likely subject to copyright and trademark protection. Using them without a license creates independent legal exposure beyond the text trademark concern.
**Fix suggestion:** Replace with generic game server icons (e.g., generic cube/block icon, generic survival icon, generic sandbox icon). Per D-10, full rewrites are not in scope for this report phase — the fix suggestion should be addressed in a future rewrite phase. For now, note this as actionable in the next copy rewrite phase.

---

## Special Notes: Game Names

### Background (per D-07)

The three game names identified as Must Avoid — Minecraft, Rust, and Terraria — are registered trademarks of their respective owners:

| Game | Owner | Jurisdiction | Risk Level |
|------|-------|-------------|------------|
| Minecraft | Mojang AB (Microsoft) | International | 🔴 Critical |
| Rust | Facepunch Studios | International | 🔴 Critical |
| Terraria | Re-Logic | International | 🔴 Critical |

### The Concern

Using unqualified game names on a commercial service creates an **implied affiliation** between Escluse and the game publishers. Even though Escluse provides hosting infrastructure for these games (not the games themselves), a reasonable consumer could assume that Escluse is officially partnered with or endorsed by the trademark owner.

### Recommended Approach for Rewrite Phase

1. **Replace game names** with qualified descriptors:

   | Current | Alternative |
   |---------|-------------|
   | Minecraft | "Minecraft-compatible servers" or "Minecraft-compatible game hosting" |
   | Rust | "Rust-compatible servers" or "Survival game servers" |
   | Terraria | "Terraria-compatible servers" or "Sandbox game servers" |

2. **Add a prominent disclaimer** (e.g., in the Supported Games section heading area):

   > "Minecraft, Rust, and Terraria are trademarks of their respective owners. Escluse is not affiliated with, endorsed by, or sponsored by Mojang AB, Facepunch Studios, or Re-Logic."

3. **Replace game image assets** with generic icons that do not depict trademarked game logos or artwork.

4. **Generalize the section heading**: Consider "Supported Game Platforms" or "Compatible Games" instead of listing specific trademarked names.

### Note on Game Images

The game image assets (`minecraft-1.svg`, `icons8-rust.svg`, `terraria-logo.png`) are classified as a separate Must Avoid concern (trademarked/copyrighted artwork). These should be addressed in the rewrite phase per D-10.

---

## Special Notes: Pricing Claims

### Scope (per D-08)

Per locked decision D-08, pricing claims receive **standard marketing scrutiny only**. Deep legal analysis is explicitly out of scope for this phase.

### Summary of Pricing Claims

| Claim | Location | Assessment | Priority |
|-------|----------|-----------|----------|
| "Simple, Transparent Pricing" | `PricingSection.tsx:178` | Standard marketing puffery — no specific measurable claim. Low risk. | Low |
| "no hidden costs" | `PricingSection.tsx:181-182` | Absolute negative claim — must be factually accurate. Flagged as Risky above. | Low |
| "🆓 Free" plan / "$0 / month" | `PlanCard.tsx:63-66` | Standard freemium pricing. Must be genuinely free. | Low |
| "Get Started Free" CTA | `PlanCard.tsx:142` | Clear association with Free plan. | Low |
| "Save ~17% with yearly billing" | `PricingSection.tsx:191-193` | Savings claim requiring substantiation. Flagged as Risky above. | Low |
| "Save $X.XX/month with yearly billing" (inline) | `PlanCard.tsx:84` | Same savings claim in card-level display. | Low |
| "$6.99/month" (Hobby) / "$24.99/month" (Pro) | `PlanCard.tsx:32-33, 53-54` | Standard tiered pricing display. Clear pricing. | Low |
| "Bring your own infrastructure." | `PricingSection.tsx:272` | Clear disclosure — not a pricing claim per se. | Low |

### Recommendation

The pricing page is generally clear and transparent. The "no hidden costs" and savings claims are noted as Risky but are low priority per D-08. No urgent action required — standard scrutiny applies.

---

## Future Recommendations

### 1. Game Trademark Compliance (Highest Priority)

Implement the changes described in the Special Notes: Game Names section above. This is the most critical legal exposure identified in this audit. The rewrite phase should:
- Replace all three game names with qualified descriptors
- Add a prominent trademark disclaimer
- Replace game image assets with generic alternatives
- Consider retitling the section "Supported Games" to "Compatible Game Platforms"

### 2. Performance Claim Qualification

Review the Hero, Features, and How It Works sections to replace absolute claims with qualified language:
- "Instant Setup" → "Quick Setup"
- "instantly deployed" → "deployed and monitored, ready for players"
- "in seconds" → "quickly" or remove
- "Make Game Infrastructure Effortless" → consider less absolute alternative

### 3. Entity Registration Clarification

Verify the legal registration status of "Resonance Systems":
- If registered as PT/CV: update all references to include the legal entity designation
- If not registered: use "a brand of" or "operating as" disclaimers
- Ensure the copyright notice and company references are consistent

### 4. Security Claim Verification

For the security claims on Security.tsx:
- Verify whether SOC 2 compliance refers to Escluse or the cloud provider. If the latter, clarify the phrasing.
- Verify TLS 1.3 and AES-256 are actually implemented across all relevant data channels.
- Consider adding a security architecture overview page for transparency.

### 5. Date Standardization

Align all "Last updated" dates across:
- Legal.tsx (currently May 12, 2026)
- TermsOfService.tsx (currently May 14, 2026)
- PrivacyPolicy.tsx (currently May 14, 2026)

All three should reference the same accurate date.

### 6. Pricing Transparency (Future Phase)

While pricing claims are low priority per D-08, a future pricing page review could:
- Clarify exactly what "no hidden costs" means in practice
- Consider linking "Free" plan to a pricing FAQ page
- Add a footnote to the savings claim showing the calculation

### 7. Cookie Consent Mechanism

If Escluse serves users in the EU, a cookie consent banner may be required under:
- GDPR Art. 7 (consent requirements)
- ePrivacy Directive (cookie consent)

Currently, the Privacy Policy mentions cookies but there is no active consent mechanism visible on the landing page. This should be addressed in a separate phase focused on privacy compliance implementation.

### 8. Data Breach Notification

The Privacy Policy does not mention a 72-hour data breach notification procedure, which is required under:
- UU PDP No. 27/2022 Pasal 57 — pemberitahuan pelanggaran data dalam 72 jam
- GDPR Art. 33-34 — breach notification to supervisory authority and affected individuals

Recommend adding a breach notification section to the Privacy Policy.

### 9. Legal Clause Review (UU Perlindungan Konsumen)

Engage a legal practitioner to review the liability limitation in TermsOfService.tsx against UU Perlindungan Konsumen Pasal 18 (klausula baku). The current wording may be void by law if challenged by a consumer.

### 10. CTA Audit for "Locked" Plans

The "Locked" and "Coming Soon" status on Hobby/Pro plan cards clearly indicates unavailability. This is Good from a consumer protection perspective — no hidden subscription traps. For future phases, consider:
- Removing the "Locked" badge and associated CTA if the plans will not be available
- Or making them subscribe-able with appropriate "coming later" messaging

---

## Appendix: Verification Checklist

| # | Criterion | Status |
|---|-----------|--------|
| 1 | Report file exists (`89-AUDIT-REPORT.md`) | ✅ |
| 2 | ⚠️ DISCLAIMER present at top (per D-05) | ✅ |
| 3 | Executive Summary with tier counts | ✅ |
| 4 | 🟢 Safe items organized by menu section (per D-04) | ✅ |
| 5 | 🟡 Risky items with legal provisions (per D-02) | ✅ |
| 6 | 🔴 Must Avoid items with legal provisions (per D-02) | ✅ |
| 7 | Fix suggestions for all Risky and Must Avoid findings (per D-03) | ✅ |
| 8 | Game names classified as Must Avoid (per D-07) | ✅ |
| 9 | Pricing claims at LOW priority (per D-08) | ✅ |
| 10 | Future Recommendations section separated from risk classifications (per D-06) | ✅ |
| 11 | No analysis of docs.esluce.com or app.esluce.com (per D-09) | ✅ |
| 12 | No full copy rewrites — fix suggestions are short alternatives (per D-10) | ✅ |

---

*Report generated: 2026-06-21*
*Phase: 89 — Audit Seluruh Copy dan CTA di Landing Page dari Perspektif Hukum Legalitas Bisnis Digital*
*Framework: Hybrid — UU ITE No. 1/2024, UU PDP No. 27/2022, UU Perlindungan Konsumen No. 8/1999, UU Merek No. 20/2016, GDPR (EU) 2016/679, FTC Act §5*
