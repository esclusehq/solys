---
status: awaiting_human_verify
trigger: Browser blocks loading of TSX module with "disallowed MIME type (text/tsx)"
created: 2026-05-10T00:00:00.000Z
updated: 2026-05-10T00:00:00.000Z
---

## Current Focus
hypothesis: "Server serves raw TSX source files instead of built JavaScript"
test: "Check if dist/ folder exists and examine nginx configuration"
expecting: "No dist folder exists; nginx incorrectly configured for TSX MIME type"
next_action: "Verify deployment serves built files correctly"

## Symptoms
expected: Browser should load and execute the frontend application
actual: Browser blocks TSX files with "disallowed MIME type (text/tsx)"
errors: Loading module from "https://esluce.com/src/main.tsx" was blocked because of a disallowed MIME type ("text/tsx")
reproduction: Visit https://esluce.com in browser
started: Started when we deployed the landing-page-escluse directory

## Eliminated

## Evidence
- timestamp: 2026-05-10T00:00:00.001Z
  checked: landing-page-escluse/package.json
  found: "build" script is "vite build" - creates dist/ folder
  implication: Project needs to be built before deployment

- timestamp: 2026-05-10T00:00:00.002Z
  checked: landing-page-escluse/dist/
  found: No files found - dist/ folder does NOT exist
  implication: Project was never built; raw source files are being served

- timestamp: 2026-05-10T00:00:00.003Z
  checked: gateway/nginx.landing.conf
  found: "text/tsx tsx;" is configured as MIME type (line 28)
  implication: Browser cannot execute text/tsx - needs text/javascript

- timestamp: 2026-05-10T00:00:00.004Z
  checked: npm run build output
  found: "dist/index.html, dist/assets/index--CdP0hyb.css, dist/assets/index-DndFAZck.js" created successfully
  implication: Production build now available

- timestamp: 2026-05-10T00:00:00.005Z
  checked: nginx.landing.conf after edit
  found: Removed text/tsx MIME type, added text/javascript for js/mjs
  implication: Nginx will now serve JS files with correct MIME type

## Resolution
root_cause: "The Vite project was not built before deployment. Raw TSX source files were being served directly. Additionally, nginx had incorrect MIME type (text/tsx instead of text/javascript)"
fix: "1. Built the Vite project with 'npm run build' to generate dist/ folder with compiled JS. 2. Fixed nginx MIME type configuration to remove text/tsx and use text/javascript."
verification: "Need deployment to verify"
files_changed:
- landing-page-escluse/dist/ (generated)
- gateway/nginx.landing.conf (MIME type fix)