# Changelog

## [Unreleased]

### Fixed
- [landing] Onboarding Step 3: Retry button now properly restarts 2-minute polling timer (was stuck on "Waiting for connection..." forever after retry)
- [install] install.sh `generate_config()` now checks `$AGENT_API_KEY` env var before prompting interactively (support `sudo env AGENT_API_KEY=xxx bash -c "$(curl ...)"`)
- [install] Install command on onboarding Step 3: `sudo env AGENT_API_KEY=xxx` instead of `sudo AGENT_API_KEY=xxx` (sudo resets env by default)
