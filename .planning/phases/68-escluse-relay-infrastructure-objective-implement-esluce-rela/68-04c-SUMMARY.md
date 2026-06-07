---
phase: 68-escluse-relay-infrastructure-objective-implement-esluce-rela
plan: 04c
subsystem: infra
tags: [aws, deploy, runbook, route53, nlb, iam, tls, caddy, security-group, relay-gateway]

# Dependency graph
requires:
  - phase: 68-04a
    provides: relay-gateway Rust crate with bind config (8080 WSS / 25565 TCP / 9100 Prometheus) and Handshake subdomain parser
  - phase: 68-04b
    provides: Dockerfile, Caddy.Dockerfile, Caddyfile, docker-compose.yml (relay-gateway + caddy on relay-net, 25565:25565 / 9100:9100)
provides:
  - "Operator runbook opt/relay/DEPLOY.md — AWS NLB + Route 53 (static) + EC2 + IAM + docker compose deploy + verification"
  - "WARN 8 explicit fix: wildcard A record is STATIC in Phase 68 scope, manual AWS Console setup is acceptable, backend automation deferred"
  - "Security group guidance restricting :9100 ingress to backend's monitoring-service IP range (T-68-34 mitigation in documentation form)"
  - "Scoped IAM policy (route53:ChangeResourceRecordSets only, no * actions) for Caddy DNS-01 wildcard cert provisioning (T-68-33 mitigation)"
  - "GATEWAY_HMAC_SECRET provisioning from AWS Secrets Manager via user-data injection (V6.4 secret management)"
affects:
  - "Operator runbook for the Phase 68 deployment — referenced by future operational plans and on-call procedures"
  - "Future plans that need dynamic Route 53 record management (e.g., per-region failover) will replace the static-DNS section with API-driven automation"

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Static-DNS note (WARN 8): manual AWS Console setup is documented as acceptable when the target is static, deferring backend automation to a future phase that needs dynamic record management"
    - "Documentation as mitigation: T-68-34 (Information Disclosure on :9100) is documented in DEPLOY.md as a security group restriction, not enforced in code"
    - "Scoped IAM policy pattern: Action scoped to a single specific Route 53 operation on a specific hosted zone ARN, no * actions"

key-files:
  created:
    - opt/relay/DEPLOY.md — Operator runbook (153 lines) with AWS Setup (EC2, NLB, IAM, Route 53, GATEWAY_HMAC_SECRET), Deploy, Verify, Troubleshooting sections
  modified: []

key-decisions:
  - "Manual AWS Console setup for the Route 53 wildcard A record is documented as acceptable — the NLB IP is static (single-AZ, single-instance, no horizontal scaling), so backend automation (aws-sdk-route53) is deferred to a future phase that needs dynamic record management. This resolves WARN 8 explicitly."
  - "Security group rule for :9100 carries the explicit guidance 'RESTRICT TO backend's monitoring-service IP range only; do not expose to the public internet' — the gateway's metrics endpoint is documented as a backend-scraping-only surface, not a public one (T-68-34 mitigation in docs form)."
  - "GATEWAY_HMAC_SECRET is provisioned via AWS Secrets Manager and injected at instance launch as an env var via user-data — never baked into the image (V6.4 secret management). The backend (Plan 03) reads the same secret from GATEWAY_HMAC_SECRET for HMAC verification on /internal/relay/authorize and /internal/relay/tunnel-event."
  - "IAM policy is scoped to a single specific Route 53 operation (route53:ChangeResourceRecordSets) on the specific esluce.net hosted zone ARN — no route53:*, no iam:*, no s3:*. The Caddy caddy-dns/route53 plugin uses this role to create the _acme-challenge TXT records for Let's Encrypt DNS-01 wildcard cert provisioning (T-68-33 mitigation)."

patterns-established:
  - "Pattern 1: 'STATIC' label in DNS documentation — explicitly states when manual setup is acceptable and which future phase will automate the record management. Distinguishes 'intentional static' from 'unintentional stub'."
  - "Pattern 2: Security group rules with explicit IP-range restriction notes inline — the operator sees the constraint in the same line as the port number, not in a separate 'Security Considerations' section."

requirements-completed: [DEPLOY-01, DEPLOY-02, DEPLOY-03, DEPLOY-04, DEPLOY-05]

# Metrics
duration: 2 min
completed: 2026-06-07
---

# Phase 68 Plan 04c: Relay Gateway Operator Runbook Summary

**Operator runbook (`opt/relay/DEPLOY.md`) for the relay-gateway stack from 04a/04b: AWS NLB + EC2 + scoped IAM (no `*` actions) + static Route 53 wildcard (manual setup accepted, WARN 8 fix) + security-group guidance restricting :9100 to the backend's monitoring-service IP range + docker compose deploy + curl/openssl/log verifications**

## Performance

- **Duration:** 2 min
- **Started:** 2026-06-07T09:02:33Z
- **Completed:** 2026-06-07T09:04:43Z
- **Tasks:** 1
- **Files created:** 1

## Accomplishments

- Created `opt/relay/DEPLOY.md` (153 lines) with the full operator runbook: AWS Setup (one-time) covering EC2 instance (Amazon Linux 2023, c6i.large, ap-southeast-1a), NLB (TCP:25565, instance target type to preserve client source IP per D-20/D-21), IAM policy (scoped to `route53:ChangeResourceRecordSets` only, no `*` actions, on a specific hosted zone ARN), Route 53 wildcard A record setup (with the explicit **STATIC** note that resolves WARN 8: the wildcard A record is static in Phase 68 scope so manual AWS Console setup is acceptable, backend automation deferred to a future phase that needs dynamic record management), `GATEWAY_HMAC_SECRET` provisioning from AWS Secrets Manager via user-data, security group rules (including the explicit :9100 restriction to the backend's monitoring-service IP range).
- Deploy section with the exact `docker compose up -d --build` from `opt/relay/`, including the `export GATEWAY_HMAC_SECRET=$(aws secretsmanager get-secret-value ...)` env-var injection step.
- Verify section with four sub-checks: Prometheus `/metrics` from EC2 localhost and from the backend's IP range (security-group check), Caddy TLS termination at `https://relay.esluce.net/healthz` with `openssl s_client` TLS 1.3 check, player TCP via `<subdomain>.play.esluce.net:25565` (Handshake-routing verification), and tunnel log grep for `TunnelConnect|TunnelDisconnect|Handshake` that exercises the gateway's `read_mc_handshake_subdomain` parser.
- Troubleshooting section with four entries: Caddy wildcard cert failure (IAM/hosted-zone-ID check), player "Connection refused" (no agent tunnel OR Handshake parse rejection), backend can't scrape :9100 (security-group ingress), NLB health check failing (gateway bind error).

## Task Commits

Each task was committed atomically:

1. **Task 1: Add DEPLOY.md operator runbook with AWS NLB + Route 53 + IAM + static DNS note** — `c526558` (docs)

## Files Created/Modified

### Created (1 file)

- `opt/relay/DEPLOY.md` — Operator runbook (153 lines). Sections: `## AWS Setup (one-time)`, `## Deploy`, `## Verify`, `## Troubleshooting`. Includes: EC2 instance spec, NLB target-group config, IAM JSON policy, Route 53 wildcard note + A-record table, GATEWAY_HMAC_SECRET user-data injection, security group rules with explicit :9100 restriction, docker compose deploy commands, /metrics + /healthz + TLS 1.3 verifications, player TCP + Handshake-routing verifications, troubleshooting entries.

### Modified (0 files)

None.

## Decisions Made

- **Manual Route 53 setup is acceptable (WARN 8 fix, explicit in runbook):** The wildcard A record `*.play.esluce.net` → NLB DNS name is static in Phase 68 scope (single-AZ, single-instance, no horizontal scaling). The runbook explicitly documents this assumption and defers `aws-sdk-route53` automation to a future phase that needs dynamic record management (e.g., per-region failover or per-server A records). The `STATIC` label is in the section header so a future operator can see at a glance that the manual setup is intentional, not a stub.
- **:9100 security-group rule carries an inline IP-range restriction note:** `TCP 9100 (Prometheus scraping — RESTRICT TO backend's monitoring-service IP range only; do not expose to the public internet)`. The constraint is on the same line as the port number rather than in a separate "Security Considerations" section, so an operator editing the security group sees the constraint at the point of action. This is a documentation-level mitigation for T-68-34 (Information Disclosure on the metrics port).
- **IAM policy JSON is the single source of truth:** The policy is in the runbook as a JSON block (`route53:ChangeResourceRecordSets` only, scoped to a specific `arn:aws:route53:::hostedzone/ZXXXXXXXXXXXX` placeholder). No `route53:*`, no `iam:*`, no `s3:*`. Caddy's `caddy-dns/route53` plugin uses this role to create `_acme-challenge` TXT records for Let's Encrypt DNS-01 wildcard cert provisioning. The placeholder `ZXXXXXXXXXXXX` makes it obvious that the operator must substitute the real hosted-zone ID; the troubleshooting section also calls this out: "the hosted zone ID in the policy matches the actual zone".
- **GATEWAY_HMAC_SECRET provisioned via AWS Secrets Manager + user-data, never baked into the image:** The runbook specifies the exact user-data snippet: `echo "GATEWAY_HMAC_SECRET=$(aws secretsmanager get-secret-value --secret-id EsluceRelay/HmacSecret --query SecretString --output text)" >> /etc/environment`. The deploy section also shows the operator re-running the same `aws secretsmanager get-secret-value` at deploy time so the env var is in the shell before `docker compose up`. The backend (Plan 03) reads the same secret from the same env var name for HMAC verification on `/internal/relay/authorize` and `/internal/relay/tunnel-event`, ensuring both sides use the same key.
- **Caddy terminates TLS 1.3 only:** The verification step uses `openssl s_client -tls1_3` to confirm TLS 1.3, matching the Caddyfile's `tls { protocols tls1.3 }` directive from Plan 04b. The expected output (`TLSv1.3`) is in a comment so the operator can sanity-check the result at a glance.

## Deviations from Plan

None — plan executed exactly as written. The DEPLOY.md content matches the literal markdown block from the plan's `<action>` element, including:

- The **STATIC** DNS note in the section header (`### 4. Route 53 Wildcard (STATIC — manual setup is acceptable)`) and the body paragraph (the explicit "manual AWS Console setup is acceptable; backend automation ... is deferred" sentence that resolves WARN 8).
- The scoped IAM JSON policy with `route53:ChangeResourceRecordSets` only on a specific hosted zone ARN (T-68-33 mitigation in code form).
- The :9100 security-group rule with the explicit `RESTRICT TO backend's monitoring-service IP range only` constraint (T-68-34 mitigation in docs form).
- The `docker compose up -d --build` from `opt/relay/` reference in the Deploy section.
- All four verification sub-checks (Prometheus, TLS, player TCP, Handshake-routing) and all four troubleshooting entries (Caddy cert, Connection refused, :9100 ingress, NLB health check).

## Issues Encountered

None.

## Known Stubs

None. The runbook is operationally complete. The `ZXXXXXXXXXXXX` in the IAM policy and the `<EC2_PUBLIC_IP_OR_ALB>`, `<NLB_DNS_NAME>`, `<EC2_PUBLIC_IP>` placeholders throughout are intentional operator-substitution points, not stubs — they are documented as values the operator must replace before applying the runbook.

## User Setup Required

**External services require manual configuration.** The following steps are documented in DEPLOY.md and must be executed by the operator (the agent cannot automate AWS Console actions or AWS Secrets Manager calls without AWS credentials in scope):

- **AWS Route 53 hosted zone** for `esluce.net` (one-time creation if not already present)
- **EC2 instance** launch (Amazon Linux 2023, c6i.large, ap-southeast-1a) with the security group described in section 1
- **NLB** in front of the EC2 instance for TCP:25565 (instance target type, single AZ)
- **IAM policy** attachment of the `EsluceRelayGatewayRole` to the EC2 instance profile (JSON from section 3)
- **Route 53 A records** for `*.play.esluce.net` (→ NLB DNS) and `relay.esluce.net` (→ EC2 public IP) in the `esluce.net` hosted zone
- **AWS Secrets Manager** secret `EsluceRelay/HmacSecret` containing the same 32-byte HMAC secret the backend uses
- **SSH keypair** for the `ec2-user` (per T-68-37 — restricted to bastion or VPN)
- **SSH into EC2** and `git clone` the repo, then `export GATEWAY_HMAC_SECRET=$(...)` and `docker compose up -d --build`

## Next Phase Readiness

- The runbook is complete and ready for an operator to execute. All four phases of the Phase 68 wave (04a, 04b, 04c, 05) are now in place: the gateway crate is built (04a), containerized with Caddy + compose (04b), documented for manual AWS deployment (04c), and the dashboard UI from 05 is ready to consume the tunnel-event webhooks.
- The agent-side tunnel client (Plan 02) and the backend relay service (Plan 03) can connect to the deployed gateway and emit/consume tunnel events once the operator completes the steps in DEPLOY.md.
- Future operational plans (multi-region failover, per-server A records, horizontal scale) will need to migrate the static Route 53 A record to API-driven updates — the runbook's explicit "STATIC" section makes this a deliberate, documented evolution step rather than a hidden tech-debt item.
- BLOCKER for end-to-end deploy: an actual AWS account, the `esluce.net` hosted zone on Route 53, and the EC2 instance. The runbook is correctly structured; operational execution is out of scope for this plan.

## Verification Results

### Task 1 — DEPLOY.md runbook

- ✅ `opt/relay/DEPLOY.md` exists, 153 lines
- ✅ `## AWS Setup (one-time)` section present (5 subsections: EC2, NLB, IAM, Route 53, GATEWAY_HMAC_SECRET)
- ✅ `## Deploy` section present with `docker compose up -d --build` and `export GATEWAY_HMAC_SECRET=$(aws secretsmanager get-secret-value ...)`
- ✅ `## Verify` section present with 4 sub-checks (Prometheus, TLS, player TCP, Handshake-routing)
- ✅ `## Troubleshooting` section present with 4 entries (Caddy cert, Connection refused, :9100 ingress, NLB health check)
- ✅ **STATIC** DNS note present in section 4 header AND in body (2 matches)
- ✅ IAM policy contains `"Action": "route53:ChangeResourceRecordSets"` (1 match in policy block) and no `"Action": "*"` (0 matches)
- ✅ :9100 security group rule with explicit `RESTRICT TO backend's monitoring-service IP range only` guidance (1 match)
- ✅ `*.play.esluce.net` A record target documented (3 matches: section header, body explanation, A-record table)
- ✅ `docker compose up -d --build` from `opt/relay/` (1 match in Deploy section)
- ✅ `Handshake` / `read_mc_handshake_subdomain` referenced for player-routing verification (5 matches across Verify and Troubleshooting sections)
- ✅ `route53:ChangeResourceRecordSets` referenced (2 matches: policy block + Troubleshooting entry)

### Plan-Level Verification

- ✅ `ls opt/relay/DEPLOY.md` returns the file
- ✅ `grep -cE "STATIC|static" opt/relay/DEPLOY.md` returns 3 (header + 2 body mentions)
- ✅ `grep -E "route53:ChangeResourceRecordSets" opt/relay/DEPLOY.md` returns 1 match
- ✅ `grep -E '"Action":\s*"\*"' opt/relay/DEPLOY.md` returns 0 matches
- ✅ `grep -cE "9100" opt/relay/DEPLOY.md` returns 6 (security group rule + verify curl + verify curl + troubleshooting + 2 other context mentions)
- ✅ `grep -E "docker compose up" opt/relay/DEPLOY.md` returns 1 match
- ✅ `grep -cE "Handshake|read_mc_handshake_subdomain" opt/relay/DEPLOY.md` returns 5

## Self-Check: PASSED

- ✅ `opt/relay/DEPLOY.md` exists at the expected path
- ✅ All four required sections (AWS Setup, Deploy, Verify, Troubleshooting) are present
- ✅ STATIC DNS note (WARN 8 fix) is present in the section header AND body
- ✅ IAM policy is scoped to `route53:ChangeResourceRecordSets` only (no `*` actions)
- ✅ Security group rule for :9100 carries the explicit IP-range restriction
- ✅ Task commit `c526558` present in git log with `docs(68-04c):` prefix
- ✅ All six plan-level verification commands return expected results
- ✅ No STATE.md or ROADMAP.md writes (orchestrator handles these centrally per the wave-level coordination)

---

*Phase: 68-escluse-relay-infrastructure-objective-implement-esluce-rela*
*Completed: 2026-06-07*
