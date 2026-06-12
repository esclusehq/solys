# Changelog

## [Unreleased]

### Fixed
- [landing] Onboarding Step 3: Retry button now properly restarts 2-minute polling timer (was stuck on "Waiting for connection..." forever after retry)
- [install] install.sh `generate_config()` now checks `$AGENT_API_KEY` env var before prompting interactively (support `sudo env AGENT_API_KEY=xxx bash -c "$(curl ...)"`)
- [install] Install command on onboarding Step 3: `sudo env AGENT_API_KEY=xxx` instead of `sudo AGENT_API_KEY=xxx` (sudo resets env by default)

### Added
- [onboarding] Node connection success page: "Create Game Server" button now redirects to https://app.esluce.com instead of landing page
- [relay] UDP port pool seed migration (19132-19231) for per-server UDP allocations
- [relay] Protocol-aware port allocation (TCP vs UDP ports) in port allocation use case
- [relay] Loader field threaded through relay pipeline (ServerRelayInfo, TunnelConnect, RelayServerConfig)
- [relay] `run_udp_relay_session` with TLV framing for Bedrock UDP relay
- [relay] Route 53 SRV record methods (upsert, list, delete, resolve) for Bedrock DNS discovery

### Fixed
- [install] install.sh binary extraction: removed `|| true` that silently hid gzip CRC errors, causing corrupted binary install (was `gzip -dc | tar xf - || true`, now `tar xzf`)
- [onboarding] Full flow tested end-to-end: Retry button, 2-min timeout, agent reconnection, all working
