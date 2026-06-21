---
phase: quick-fix
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - landing-page-escluse/src/components/auth/Footer.tsx
  - landing-page-escluse/src/App.tsx
autonomous: true
requirements: []

must_haves:
  truths:
    - "Auth Footer internal links use <Link> from react-router-dom, avoiding full page reloads"
    - "useAuthStore import in App.tsx is grouped with top-of-file imports"
    - "SupportedGames component uses game.name as alt text for accessibility"
  artifacts:
    - path: "landing-page-escluse/src/components/auth/Footer.tsx"
      provides: "Auth footer with <Link> navigation"
      contains: "from 'react-router-dom'"
    - path: "landing-page-escluse/src/App.tsx"
      provides: "Clean import ordering and accessible alt text"
      contains: "import { useAuthStore }"
  key_links:
    - from: "auth/Footer.tsx"
      to: "react-router-dom"
      via: "import { Link } from 'react-router-dom'"
    - from: "App.tsx"
      to: "./lib/stores/authStore"
      via: "import at top of file (not line 648)"

---

<objective>
Fix 3 code review findings from Phase 90 code review in a single pass:
1. WR-05 — Replace `<a>` with `<Link>` in auth Footer for SPA routing
2. IN-01 — Move `useAuthStore` import to top of App.tsx
3. IN-02 — Fix alt text in SupportedGames from `game.desc` to `game.name`

Purpose: Resolve code review findings efficiently with atomic changes.
Output: Updated auth/Footer.tsx and App.tsx, verified by successful build.
</objective>

<execution_context>
@/home/rhnbztnl/.config/opencode/get-shit-done/workflows/execute-plan.md
@/home/rhnbztnl/.config/opencode/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@landing-page-escluse/src/components/auth/Footer.tsx
@landing-page-escluse/src/src/App.tsx

<interfaces>
From react-router-dom (already used in App.tsx):
```typescript
export { Link } from 'react-router-dom';
```
Link is the standard SPA navigation component that avoids full page reloads by intercepting navigation and using client-side routing.
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Replace `<a>` with `<Link>` in auth/Footer.tsx</name>
  <files>
    landing-page-escluse/src/components/auth/Footer.tsx
  </files>
  <action>
    Per WR-05: The auth Footer component uses `<a>` tags with `href` for internal SPA routes, causing full page reloads. Replace them with `<Link>` from react-router-dom.

    1. Add import at the top of the file:
       ```
       import { Link } from 'react-router-dom';
       ```

    2. Replace each `<a>` with `<Link>` and `href` with `to` for the 4 internal links:
       - Line 12: `<a className="..." href="/privacy-policy">` → `<Link className="..." to="/privacy-policy">`
       - Line 13: `<a className="..." href="/terms-of-service">` → `<Link className="..." to="/terms-of-service">`
       - Line 14: `<a className="..." href="/security">` → `<Link className="..." to="/security">`
       - Line 15: `<a className="..." href="/status">` → `<Link className="..." to="/status">`

    Do NOT change the anchor closing tags `</a>` to `</Link>` — they remain as `<a>` becomes `<Link>` via the opening tag only. Actually, replace both opening and closing tags:
    - `<a` → `<Link` (opening)
    - `</a>` → `</Link>` (closing)
    - `href=` → `to=` (attribute)

    All class names and content remain identical.
  </action>
  <verify>
    <automated>grep -c "from 'react-router-dom'" landing-page-escluse/src/components/auth/Footer.tsx</automated>
  </verify>
  <done>
    auth/Footer.tsx imports `Link` from `react-router-dom` and all 4 internal nav links use `<Link to="...">` instead of `<a href="...">`.
  </done>
</task>

<task type="auto">
  <name>Task 2: Move `useAuthStore` import to top of App.tsx</name>
  <files>
    landing-page-escluse/src/App.tsx
  </files>
  <action>
    Per IN-01: The `import { useAuthStore } from './lib/stores/authStore'` on line 648 is placed inline in the file, between component definitions. This violates the convention of grouping all imports at the top of the file.

    1. Add the import at the top section, after the lucide-react import (line 38) and before the component definitions (line 40):
       ```
       import { useAuthStore } from './lib/stores/authStore';
       ```
       Insert it right after line 38 (after the lucide-react imports), keeping alphabetical order among local imports.

    2. Remove the standalone import statement at line 648. The line reads:
       ```
       import { useAuthStore } from './lib/stores/authStore';
       ```
       Remove this entire line.

    After the change, the `useAuthStore` import should only appear once, at the top of the file with all other imports.
  </action>
  <verify>
    <automated>grep -c "import.*useAuthStore" landing-page-escluse/src/App.tsx</automated>
  </verify>
  <done>
    `import { useAuthStore }` appears exactly once at the top of App.tsx (not at line 648 or anywhere else in the file body).
  </done>
</task>

<task type="auto">
  <name>Task 3: Fix alt text in SupportedGames component</name>
  <files>
    landing-page-escluse/src/App.tsx
  </files>
  <action>
    Per IN-02: The SupportedGames component renders game card images with `alt={game.desc}` (line 500), but the alt text should describe the game name for screen readers and accessibility. Change it to `alt={game.name}`.

    On line 500, change:
    ```
    alt={game.desc}
    ```
    to:
    ```
    alt={game.name}
    ```

    This is a single-character property change on one line. The rest of the component is unchanged.
  </action>
  <verify>
    <automated>grep "alt={game.name}" landing-page-escluse/src/App.tsx | wc -l</automated>
  </verify>
  <done>
    SupportedGames img elements use `game.name` (not `game.desc`) for alt text, providing meaningful game name descriptions for screen readers.
  </done>
</task>

</tasks>

<verification>
After all 3 tasks are complete, run the build to verify no regressions:

```bash
cd landing-page-escluse && npm run build
```

The build must exit with code 0. Check for:
- No TypeScript compilation errors
- No module resolution errors (Link import, useAuthStore import)
- No linting issues
</verification>

<success_criteria>
- [ ] auth/Footer.tsx: 4 links use `<Link to="...">` with import from react-router-dom
- [ ] App.tsx: `useAuthStore` import is at the top of file, not at line 648
- [ ] App.tsx: SupportedGames img alt text uses `game.name` not `game.desc`
- [ ] `npm run build` passes with exit code 0
</success_criteria>

<output>
After completion, create `.planning/quick/260621-mkp-fix-auth-footer-links-import-order-alt-t/260621-mkp-SUMMARY.md`
</output>
