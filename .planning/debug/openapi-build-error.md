---
status: diagnosed
trigger: error: couldn't read src/presentation/routes/../../../openapi.json: No such file or directory
created: 2026-05-09T00:00:00Z
updated: 2026-05-09T00:00:00Z
---

## Current Focus
hypothesis: CONFIRMED - Dockerfile missing COPY for openapi.json AND relative path is wrong
test: Verified by reading Dockerfile line 6 and tracing path resolution
expecting: openapi.json needs to be copied; path needs fixing
next_action: ROOT CAUSE FOUND - Ready to report

## Symptoms
expected: openapi.json should be found and included at compile time
actual: error: couldn't read src/presentation/routes/../../../openapi.json: No such file or directory
errors: error: couldn't read src/presentation/routes/../../../openapi.json: No such file or directory
reproduction: Run docker build from ./api context
started: Build context setup

## Eliminated

## Evidence
- timestamp: 2026-05-09T00:00:00Z
  checked: Docker build context transfer size
  found: Only 42.81kB transferred (should include 30KB openapi.json)
  implication: File is being excluded or build context root is wrong

- timestamp: 2026-05-09T00:00:00Z
  checked: Dockerfile COPY command (line 6)
  found: `COPY Cargo.toml Cargo.lock src ./` - does NOT include openapi.json
  implication: openapi.json is never copied into the Docker image

- timestamp: 2026-05-09T00:00:00Z
  checked: Path resolution for include_str
  found: include_str!("../../../openapi.json") from src/presentation/routes/ would resolve to context-root/openapi.json during build
  implication: Path assumes file is at Docker context root (./api/), but it's never copied

- timestamp: 2026-05-09T00:00:00Z
  checked: .dockerignore existence
  found: No .dockerignore file exists in the project
  implication: .dockerignore is NOT the cause

## Resolution
root_cause: |
  The Dockerfile does not copy openapi.json into the Docker build context.
  The build context is set to `.` (./api/), but Dockerfile line 6 only copies
  Cargo.toml, Cargo.lock, and src/. The openapi.json file at api/openapi.json
  is never included in the build, so when Rust compiles openapi_routes.rs and
  tries to include_str!("../../../openapi.json"), the file doesn't exist in the
  Docker image's build context.

fix: |
  Add `COPY openapi.json ./` to the Dockerfile after line 6 (after copying Cargo.toml
  and before building). This will copy openapi.json to /app/openapi.json, which
  matches the expected path `../../../openapi.json` relative to /app/src/presentation/routes/.

  Additionally, fix the relative path: from /app/src/presentation/routes/, going up
  3 directories (../../../) lands at /app, so the path should be simply
  `../../openapi.json` (2 levels up from routes to src to app).

verification: |
  After adding COPY openapi.json ./ to Dockerfile and fixing path to ../../openapi.json,
  docker-compose build should succeed and include the 30KB openapi.json in context transfer.

files_changed: [api/Dockerfile]

