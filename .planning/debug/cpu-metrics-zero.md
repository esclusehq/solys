---
status: resolved
trigger: "Fix CPU Usage chart showing 0% - flat line at bottom"
created: 2026-04-21T00:00:00Z
updated: 2026-04-21T00:00:00Z
---

## Current Focus
hypothesis: 
test: 
expecting: 
next_action:

## Symptoms
expected: CPU Usage should show actual CPU usage over time (graph with varying values)
actual: Flat line at 0% (all values are 0)
errors: None - graph renders but shows no data
reproduction: Go to server details, Overview tab shows flat CPU graph
started: New server or recently started

## Eliminated

## Evidence
- timestamp: 2026-04-21
  checked: api/src/infrastructure/factories/simple_executor_factory.rs
  found: For executor_type "agent", factory falls back to SSH if no node_client configured (line 64)
  implication: MonitoringService uses SSH executor which has collect_metrics returning hardcoded zeros
- timestamp: 2026-04-21
  checked: api/src/infrastructure/executors/ssh_server_executor.rs
  found: collect_metrics() returned hardcoded cpu_usage: 0.0, memory: 0, disk: 0
  implication: Fix applied to use podman stats command to collect real metrics

## Resolution
root_cause: SSHServerExecutor::collect_metrics() was returning hardcoded 0.0 for CPU/memory/disk. This executor is used as fallback when agent executor cannot connect to node agent.
fix: Updated SSHServerExecutor::collect_metrics() to:
- Run `podman stats` command via SSH to get actual CPU % and memory usage
- Run `podman exec <container> rcon-cli list` for player count
- Run `podman exec <container> rcon-cli tps` for TPS
- Run `podman exec <container> du -sb /data` for disk usage
verification: Code compiles successfully with cargo check
files_changed:
- api/src/infrastructure/executors/ssh_server_executor.rs
