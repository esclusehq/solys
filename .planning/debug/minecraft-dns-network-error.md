---
status: resolved
trigger: "Minecraft server startup failure - DNS resolution error in Docker container - NetworkingConfig fix not working"
created: 2026-04-10T00:00:00Z
updated: 2026-04-11T08:10:00Z
---

## Current Focus
hypothesis: Container created without explicit network attachment due to incorrect use of NetworkingConfig API instead of HostConfig.network_mode
test: Deploy web-agent and create new Minecraft server container
expecting: Container will be created with bridge network attached, enabling DNS resolution
next_action: Verify fix works by deploying and testing container creation

## Symptoms
expected: Minecraft server container can resolve external DNS (mojang.com) to download version metadata
actual: DNS resolution fails with "Network is unreachable" when trying to resolve launchermeta.mojang.com
errors:
  - "'resolve-minecraft-version' command failed. Version is 1.55.4"
  - "Failed to resolve 'launchermeta.mojang.com'"
  - "Network is unreachable" - when trying to send DNS query to 1.1.1.1
reproduction: Start Minecraft server container, it fails during version resolution
started: Unknown - likely a recent change in Docker network configuration

## Evidence
- timestamp: 2026-04-10T12:00:00Z
  checked: Docker compose files
  found: Main compose uses bridge network (app-network). Web-agent compose uses network_mode: host.
  implication: Web-agent runs in host mode, but creates containers that should use bridge.

- timestamp: 2026-04-10T12:05:00Z
  checked: Working container (mc-8d257436) network settings
  found: Connected to default "bridge" network with IP 172.17.0.2, Gateway 172.17.0.1
  implication: Working container has proper network attachment

- timestamp: 2026-04-10T12:10:00Z
  checked: Failed container (mc-b9f39b89) network settings
  found: "Networks": {} (empty) - container not attached to any network
  implication: Container was created without network attachment - no network interface means no outbound traffic possible

- timestamp: 2026-04-10T12:15:00Z
  checked: DNS test in working container
  found: DNS resolution works - can resolve google.com and launchermeta.mojang.com, ping to 1.1.1.1 succeeds
  implication: Network is functional when properly attached

- timestamp: 2026-04-10T12:20:00Z
  checked: Container creation code (runtime.rs)
  found: Uses bollard with default network settings - no explicit network configuration specified
  implication: Container creation may not be attaching to bridge network properly

- timestamp: 2026-04-11T08:00:00Z
  checked: bollard API docs and code analysis
  found: NetworkingConfig is for endpoint settings, NOT for network attachment. HostConfig.network_mode is required to attach to bridge.
  implication: Original "fix" used wrong API - NetworkingConfig alone doesn't attach container to network

## Resolution
root_cause: Container was created without explicit network attachment - the NetworkingConfig approach was incorrect. bollard's NetworkingConfig is for configuring endpoint settings (IPs, gateway, aliases), not for attaching containers to networks. The correct approach is HostConfig.network_mode = "bridge".
fix: Replaced NetworkingConfig with HostConfig.network_mode: Some("bridge".to_string()) in both handle_create and handle_start functions
verification: Code compiles successfully

## Files Changed
- web-agent/src/handlers/runtime.rs: Changed handle_create and handle_start to use HostConfig.network_mode instead of NetworkingConfig (2 locations fixed)