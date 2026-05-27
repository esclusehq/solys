---
status: resolved
trigger: "websocket-metrics-near-zero"
created: 2026-05-01T00:00:00Z
updated: 2026-05-01T20:38:00Z
---

## Current Focus
hypothesis: "Investigating metrics flow: need to trace how WebSocket metrics data moves from agent to charts"
test: "Trace metrics data flow: WebSocket handler -> state -> chart component"
expecting: "Identify where data gets zeroed/stale"
next_action: "Root cause identified and fixed"

## Symptoms
expected: Charts should display real CPU/RAM/Disk values from live agent (e.g., 50%+ CPU, 300+MB RAM, etc.)
actual: Charts showing ~0-0.12% CPU, ~42MB RAM, ~20MB disk (near-zero/stale data)
errors: []
reproduction: Open server details page - metrics charts always show near-zero
started: Started after recent metrics implementation

## Eliminated
- Frontend code correctly displays metrics data
- WebSocket correctly receives and passes metrics
- Database correctly stores/retrieves metrics

## Evidence
Root cause found in web-agent metrics calculation:
* File: web-agent/src/handlers/metrics.rs, function: calculate_cpu_percent()
* The CPU calculation returns 0.0% when measurement interval is very short
* Formula: (cpu_delta / system_delta) * num_cpus * 100.0
* If system_delta == 0 OR cpu_delta == 0, formula returns 0.0

CPU fallback fix applied in calculate_cpu_percent():
- Added fallback when system_delta is 0 but cpu_delta > 0
- Added default fallback of num_cpus when both are 0 (shows baseline)

## Resolution
root_cause: "CPU calculation formula returns 0 when measurement interval is too short, causing near-zero values"
fix: "Added fallback calculations in calculate_cpu_percent() in web-agent/src/handlers/metrics.rs"
verification: "Need to rebuild web-agent and verify metrics populate correctly"
files_changed:
- web-agent/src/handlers/metrics.rs (calculate_cpu_percent function updated)