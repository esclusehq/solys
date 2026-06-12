---
phase: 73-approach-1-per-server-udp-port-recommended-mvp-cara-alokasik
plan: 02
subsystem: agent
tags: udp, bedrock, relay, tlv, yamux, tunnel

requires:
  - phase: 73-approach-1-per-server-udp-port-recommended-mvp-cara-alokasik
    plan: 01
    provides: RelayServerConfig.loader field, local_mc_addr in deploy config
provides:
  - TunnelConnect JSON extended with loader field (null for Java, "bedrock" for Bedrock)
  - drive_inbound_streams Bedrock detection via is_udp flag, branching to UDP relay
  - run_udp_relay_session() with TLV framing for bidirectional UDP forwarding
affects:
  - 73-03 (gateway tunnel handler will consume the loader field)
  - 73-04 (integration testing)

tech-stack:
  added: []
  patterns:
    - TLV framing protocol (0x01 type byte, 4-byte big-endian length, payload)
    - UDP binding to ephemeral port via UdpSocket::bind("0.0.0.0:0")
    - Concurrent yamux↔UdpSocket forwarding with tokio::select!

key-files:
  created: []
  modified:
    - agent/solys/src/handlers/relay_client.rs
    - agent/solys/src/handlers/relay_session.rs

key-decisions:
  - "Task 2 and Task 3 have a forward dependency (Task 2 references run_udp_relay_session created in Task 3) — both implemented before final cargo check verification"
  - "TLV type byte 0x01 for datagrams, 0xFF reserved for future control frames"
  - "UDP path skips Docker container IP resolution and uses local_mc_addr directly per D-11"

requirements-completed: []

duration: 4 min
completed: 2026-06-12
---

# Phase 73: Approach 1 — Per-Server UDP Port, Plan 02 Summary

**Agent UDP session handler: TunnelConnect loader field + `run_udp_relay_session()` with TLV framing for Bedrock relay**

## Performance

- **Duration:** 4 min
- **Started:** 2026-06-12T18:22:09Z
- **Completed:** 2026-06-12T18:28:05Z
- **Tasks:** 3
- **Files modified:** 2

## Accomplishments

- Extended TunnelConnect JSON with `"loader": cfg.loader` field — serializes as `"bedrock"` for Bedrock servers, `null` for Java servers
- Added `is_udp` flag to `drive_inbound_streams` — Bedrock detection via `cfg.loader.as_deref() == Some("bedrock")`
- Branched spawn logic: `run_udp_relay_session` for UDP streams, `run_relay_session` for TCP streams
- Skipped Docker container IP resolution for UDP path (uses `local_mc_addr` directly per D-11)
- Implemented `run_udp_relay_session()` with TLV framing protocol — bidirectional UDP forwarding between yamux stream and local UdpSocket
- TLV framing: `[0x01 type byte][4-byte big-endian length][datagram payload]`
- Both directions: yamux→UdpSocket (player→container) and UdpSocket→yamux (container→player)
- Byte counting toward rekey threshold with `Arc<AtomicU64>`
- All code compiles and builds successfully (cargo check + cargo build pass)

## Task Commits

Each task was committed atomically to the `agent/solys` nested repo:

1. **Task 1: Add loader field to TunnelConnect JSON** — `1c2682f` (feat)
2. **Task 2: Wire is_udp flag for Bedrock detection in drive_inbound_streams** — `cdfebc4` (feat)
3. **Task 3: Add run_udp_relay_session with TLV framing** — `d2fb255` (feat)

## Files Created/Modified

- `agent/solys/src/handlers/relay_client.rs` — Extended TunnelConnect JSON with loader field, added is_udp detection and dispatch, Docker resolve guard
- `agent/solys/src/handlers/relay_session.rs` — Added TLV constants and `run_udp_relay_session()` function (134 lines), added `AsyncReadExt` and `AsyncWriteExt` imports

## Decisions Made

- Task 2 references `run_udp_relay_session` which is created in Task 3 — both tasks were implemented before final cargo check verification passed
- TLV framing uses type byte `0x01` for datagrams; `0xFF` reserved for future control frames
- UDP binding to ephemeral port (`0.0.0.0:0`) per D-10 to avoid port exhaustion
- UDP path uses `local_mc_addr` directly per D-11, skipping Docker container IP resolution
- `AsyncReadExt` and `AsyncWriteExt` imports added for `read_exact()` and `flush()` methods

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added missing AsyncReadExt import in relay_session.rs**
- **Found during:** Task 3 verification (cargo check)
- **Issue:** `read_exact()` method requires `AsyncReadExt` trait in scope — not just `AsyncRead`
- **Fix:** Added `AsyncReadExt` to the imports in relay_session.rs
- **Files modified:** agent/solys/src/handlers/relay_session.rs
- **Verification:** cargo check passes
- **Committed in:** `d2fb255` (Task 3 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Required for compilation. No scope creep.

## Issues Encountered

- Task 2 references `run_udp_relay_session` which is defined in Task 3 — resolved by implementing Task 3 code before final cargo check verification

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Agent UDP session handler complete: TunnelConnect includes loader field, Bedrock servers trigger UDP relay sessions
- Ready for Plan 73-03 (gateway tunnel handler consuming the loader field)
- Ready for Plan 73-04 (integration testing)

## Verification Results

### Plan-level Verification

1. ✅ TunnelConnect JSON contains `"loader"` field — line 183 of relay_client.rs
2. ✅ `drive_inbound_streams` accepts `is_udp` and branches correctly — line 291/325
3. ✅ Docker resolve skipped for UDP path — `if is_udp { local_mc_addr.to_string() }` at line 296
4. ✅ `run_udp_relay_session` exists with TLV framing + UdpSocket — relay_session.rs line 188
5. ✅ `cargo check` and `cargo build` pass for agent/solys
6. ✅ Bidirectional forwarding: yamux→UdpSocket AND UdpSocket→yamux (yamux_to_udp + udp_to_yamux tasks)

### TDD Gate Compliance

Not applicable — plan type is `execute`, not `tdd`.

## Self-Check: PASSED

- ✅ SUMMARY.md exists at `.planning/phases/73-approach-1-per-server-udp-port-recommended-mvp-cara-alokasik/73-02-SUMMARY.md`
- ✅ Task 1 commit found (`1c2682f`) — feat(73-02): add loader field to TunnelConnect JSON
- ✅ Task 2 commit found (`cdfebc4`) — feat(73-02): wire is_udp flag for Bedrock detection
- ✅ Task 3 commit found (`d2fb255`) — feat(73-02): add run_udp_relay_session with TLV framing
- ✅ Parent repo summary commit found (`669a405`)
- ✅ All 2 modified key files exist at expected paths
- ✅ All plan-level acceptance criteria pass
- ✅ cargo check and cargo build pass

---

*Phase: 73-approach-1-per-server-udp-port-recommended-mvp-cara-alokasik*
*Completed: 2026-06-12*
