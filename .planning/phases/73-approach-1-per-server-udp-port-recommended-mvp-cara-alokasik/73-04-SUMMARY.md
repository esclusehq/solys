# 73-04 SUMMARY — Dashboard Bedrock Address Display + Route 53 SRV Records + Infra Docs

**Status:** ✅ Complete

## Tasks

### Task 1: Backend — Route 53 SRV Record Methods ✅ (already done)
- `relay_service.rs` already had `create_srv_record`, `delete_srv_record`, `delete_srv_for_server` methods
- `push_relay_config` already calls `create_srv_record` for Bedrock servers
- SRV format: `_minecraft._udp.bedrock-{subdomain}.play.esluce.com` → `0 0 {port} relay.esluce.net`
- Both methods are best-effort (warn on failure, no error propagation)

### Task 2: Frontend — Bedrock Address Display + UDP Badge ✅
- **ConnectivitySection.jsx:** Added "Bedrock Connection" card showing:
  - Direct IP:Port (`relay.esluce.net:{port}`)
  - SRV Address (`bedrock-{subdomain}.play.esluce.com`)
  - Visible only when `server.mc_loader === 'bedrock'`
- **TunnelHealthCard.jsx:** Added `mode` prop and "UDP" badge next to title when `mode='udp'`
- ConnectivitySection passes `mode={server?.mc_loader === 'bedrock' ? 'udp' : 'tcp'}`

### Task 3: Infrastructure Documentation ✅
- Added Section 6 to `opt/relay/DEPLOY.md`:
  - 6a: NLB UDP Listener (19132-19231)
  - 6b: Security Group UDP inbound rules
  - 6c: Gateway config UDP section (`port_start`, `port_end`, `grace_period_secs`)
  - 6d: Route 53 Hosted Zone ID for SRV records
  - 6e: Verification steps (nmap, dashboard, Bedrock client)

## Deviations from Plan

None — plan executed as written. Backend Task 1 was already implemented in a prior session, so no code changes were needed.

## Self-Check: PASSED

- ✅ `relay_service.rs` has `create_srv_record` (line 440) and `delete_srv_record` (line 549) and `delete_srv_for_server` (line 519)
- ✅ `push_relay_config` calls `create_srv_record` for Bedrock servers (line 374)
- ✅ `ConnectivitySection.jsx` renders Bedrock address block when `server.mc_loader === 'bedrock'`
- ✅ `TunnelHealthCard.jsx` accepts `mode` prop and shows "UDP" badge when mode='udp'
- ✅ `opt/relay/DEPLOY.md` has UDP/NLB/SRV documentation in Section 6
