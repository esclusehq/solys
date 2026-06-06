---
phase: 68-escluse-relay-infrastructure-objective-implement-esluce-rela
plan: 04c
type: execute
wave: 3
depends_on:
  - 68-01
  - 68-02
  - 68-03
  - 68-04a
  - 68-04b
files_modified:
  - opt/relay/DEPLOY.md
autonomous: true
requirements:
  - DEPLOY-01
  - DEPLOY-02
  - DEPLOY-03
  - DEPLOY-04
  - DEPLOY-05

must_haves:
  truths:
    - "Operator runbook documents the AWS NLB + Route 53 + EC2 + IAM provisioning steps for the relay gateway"
    - "DEPLOY.md explicitly notes that the wildcard A record `*.play.esluce.net` → NLB is STATIC (NLB IP does not change in Phase 68 scope), so manual AWS Console setup is acceptable and backend automation is deferred"
    - "IAM policy is scoped to `route53:ChangeResourceRecordSets` for the specific `esluce.net` hosted zone only — no `*` actions"
  artifacts:
    - path: "opt/relay/DEPLOY.md"
      provides: "Operator runbook: AWS NLB target group, Route 53 wildcard (static, no automation), IAM policy, EC2 setup, docker compose deploy, and verification steps"
      contains: "## AWS Setup"
  key_links:
    - from: "opt/relay/DEPLOY.md"
      to: "opt/relay/docker-compose.yml"
      via: "DEPLOY.md references `docker compose up -d --build` from `opt/relay/` (the compose file from Plan 04b)"
      pattern: "docker compose up -d --build"
    - from: "opt/relay/DEPLOY.md"
      to: "AWS Route 53"
      via: "DEPLOY.md documents the manual A record `*.play.esluce.net` → NLB DNS name setup; the static nature means backend automation (aws-sdk-route53) is deferred"
      pattern: "\\*\\.play\\.esluce\\.net"
---

<objective>
Add the operator runbook for provisioning the AWS infrastructure that hosts the relay gateway built in Plan 04a and containerized in Plan 04b. This is sub-plan 04c of a 3-part split of the original Plan 04. The other sub-plans are 04a (gateway crate + Handshake-parse routing) and 04b (Docker + Caddy + compose).

Output:
- `opt/relay/DEPLOY.md` — operator runbook with AWS NLB + Route 53 + EC2 + IAM + docker compose deploy steps, plus the static-DNS note (WARN 8) and the security-group guidance that restricts :9100 ingress to the backend's monitoring service.
</objective>

<execution_context>
@/home/rhnbztnl/.config/opencode/get-shit-done/workflows/execute-plan.md
@/home/rhnbztnl/.config/opencode/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/ROADMAP.md
@.planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-CONTEXT.md
@.planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-RESEARCH.md
@.planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-04a-PLAN.md
@.planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-04b-PLAN.md
</context>

<interfaces>
From Plan 04a: the gateway binds `0.0.0.0:8080` (WSS via Caddy), `0.0.0.0:25565` (raw TCP via NLB), and `0.0.0.0:9100` (Prometheus, per D-22).

From Plan 04b: the docker-compose.yml exposes `25565:25565` (NLB-targeted) and `9100:9100` (Prometheus). The Caddy image has `caddy-dns/route53` for Let's Encrypt DNS-01 wildcard cert provisioning.

From existing `.planning/phases/66-*/DEPLOY.md` (mirror the runbook format).
</interfaces>

<tasks>

<task type="auto">
  <name>Task 1: Add DEPLOY.md operator runbook with AWS NLB + Route 53 + IAM + static DNS note</name>
  <files>opt/relay/DEPLOY.md</files>
  <read_first>
    - cat .planning/phases/66-*/DEPLOY.md (if it exists, to mirror the runbook format)
    - Re-read the interfaces block in this plan
  </read_first>
  <action>
    Create `opt/relay/DEPLOY.md` with the operator runbook below. The content MUST include the static-DNS note (WARN 8 fix) and the security-group guidance for :9100.

    ```markdown
    # Esluce Relay Gateway — Operator Runbook

    This runbook provisions the AWS infrastructure that hosts the relay gateway
    (built in Plan 04a, containerized in Plan 04b) and deploys the Docker
    Compose stack.

    ## AWS Setup (one-time)

    ### 1. EC2 Instance
    - AMI: Amazon Linux 2023
    - Type: c6i.large (2 vCPU, 4 GiB RAM)
    - Region: ap-southeast-1, AZ: ap-southeast-1a
    - Security group inbound:
      - TCP 22 (admin SSH)
      - TCP 80 (Caddy HTTP-01 challenge + redirect to HTTPS)
      - TCP 443 (Caddy WSS termination for `relay.esluce.net` and `*.play.esluce.net`)
      - TCP 25565 (NLB health check + player traffic)
      - TCP 9100 (Prometheus scraping — **RESTRICT TO backend's monitoring-service IP range only**; do not expose to the public internet)
    - IAM instance profile: attach `EsluceRelayGatewayRole` (see step 3)

    ### 2. Network Load Balancer
    - Type: Network Load Balancer, internet-facing
    - Listeners: TCP:25565 → target group `relay-gateway-tg`
    - Health check: TCP:25565, 10s interval, 5s timeout
    - Cross-zone: disabled (single AZ)
    - Target type: **instance** (preserves client source IP — D-20, D-21)
    - Note the NLB's DNS name; it becomes the target of the Route 53 wildcard.

    ### 3. IAM Policy (scoped to Route 53 only)
    Attach to the EC2 instance role:
    ```json
    {
      "Version": "2012-10-17",
      "Statement": [{
        "Effect": "Allow",
        "Action": "route53:ChangeResourceRecordSets",
        "Resource": "arn:aws:route53:::hostedzone/ZXXXXXXXXXXXX"
      }]
    }
    ```
    **No `*` actions.** The Caddy `caddy-dns/route53` plugin uses this role to
    create the `_acme-challenge` TXT records needed for Let's Encrypt DNS-01
    wildcard cert provisioning.

    ### 4. Route 53 Wildcard (STATIC — manual setup is acceptable)

    **IMPORTANT (Phase 68 scope):** The wildcard A record
    `*.play.esluce.net` → NLB DNS name is **STATIC** in Phase 68 scope. The
    NLB IP does not change for the lifetime of the deployment (single-AZ,
    single-instance, no horizontal scaling, no blue/green). Because the
    target is static, **manual AWS Console setup is acceptable**; backend
    automation (e.g., `aws-sdk-route53` calls from `relay_service.rs`) is
    **deferred** to a future phase that needs dynamic record management
    (e.g., per-region failover or per-server A records).

    Create the following A records in the `esluce.net` hosted zone:
    - `*.play.esluce.net` → `<NLB_DNS_NAME>` (alias; TTL 60s)
    - `relay.esluce.net` → `<EC2_PUBLIC_IP_OR_ALB>` (alias; TTL 60s)

    The Caddy image (built in Plan 04b with `caddy-dns/route53`) will
    automatically request and renew the wildcard cert for
    `*.play.esluce.net` and the single-name cert for `relay.esluce.net`
    via Let's Encrypt DNS-01 on first boot.

    ### 5. GATEWAY_HMAC_SECRET
    Generate a 32-byte secret and store in AWS Secrets Manager as
    `EsluceRelay/HmacSecret`. Inject at instance launch as an env var via
    user-data:
    ```bash
    # /var/lib/cloud/instance/user-data.sh (excerpt)
    echo "GATEWAY_HMAC_SECRET=$(aws secretsmanager get-secret-value --secret-id EsluceRelay/HmacSecret --query SecretString --output text)" >> /etc/environment
    ```
    The backend (Plan 03) reads the same secret from
    `GATEWAY_HMAC_SECRET` and uses it to verify HMAC signatures on
    `/internal/relay/authorize` and `/internal/relay/tunnel-event`.

    ## Deploy

    ```bash
    # SSH into the EC2 instance
    ssh ec2-user@<EC2_PUBLIC_IP>

    # Clone the repo (or pull the latest)
    git clone https://github.com/escluse/escluse.git /opt/escluse
    cd /opt/escluse

    # Load the HMAC secret from AWS Secrets Manager
    export GATEWAY_HMAC_SECRET=$(aws secretsmanager get-secret-value --secret-id EsluceRelay/HmacSecret --query SecretString --output text)

    # Build and start the docker compose stack (gateway + caddy)
    cd opt/relay
    docker compose up -d --build

    # Tail the gateway logs
    docker logs -f relay-gateway
    ```

    ## Verify

    ### 1. Prometheus is reachable from the backend
    From the EC2 instance:
    ```bash
    curl -fsS http://localhost:9100/metrics | grep active_tunnels
    # Expected: 0 (no tunnels yet)
    ```
    From the backend host (or wherever the monitoring service runs):
    ```bash
    curl -fsS http://<EC2_PUBLIC_IP>:9100/metrics | grep active_tunnels
    # Expected: 0
    ```
    If the second curl fails, check the security group — :9100 must allow
    ingress from the backend's IP range.

    ### 2. TLS is terminated by Caddy
    ```bash
    curl -fsS https://relay.esluce.net/healthz
    # Expected: "OK"
    ```
    Verify TLS 1.3:
    ```bash
    echo | openssl s_client -connect relay.esluce.net:443 -tls1_3 2>&1 | grep "Protocol"
    # Expected: TLSv1.3
    ```

    ### 3. Player TCP works
    From any Minecraft Java client (NOT inside the VPC), connect to
    `<subdomain>.play.esluce.net:25565`. The client should resolve via
    Route 53 → NLB → EC2 → gateway. The gateway's `read_mc_handshake_subdomain`
    (Plan 04a) will parse the Handshake, look up the subdomain in the
    `by_subdomain` DashMap, and forward to the matching yamux stream.

    ### 4. Tunnel logs show Handshake-routing working
    ```bash
    docker logs -f relay-gateway | grep -E "TunnelConnect|TunnelDisconnect|Handshake"
    ```
    On a successful player connection, you should see:
    ```
    [RELAY] Tunnel connected: server=...
    [PLAYER] Handshake from <IP>: subdomain=abc12345
    ```

    ## Troubleshooting

    - **Caddy can't get the wildcard cert:** check the IAM role's
      `route53:ChangeResourceRecordSets` permission and the hosted zone
      ID in the policy matches the actual zone.
    - **Player gets "Connection refused":** the agent hasn't opened a
      tunnel yet, OR the Handshake parser rejected the subdomain. Check
      the gateway logs for `Failed to parse Handshake from ...` lines.
    - **Backend can't scrape :9100:** security group blocks ingress. Add
      the backend's IP range to the :9100 rule.
    - **NLB health check failing:** the gateway container isn't listening
      on :25565. Check `docker logs relay-gateway` for bind errors.
    ```
  </action>
  <verify>
    <automated>ls /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay/DEPLOY.md && echo "---" && wc -l /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay/DEPLOY.md && echo "---" && grep -E "STATIC|static|manual AWS Console|backend automation" /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay/DEPLOY.md | head -5 && echo "---" && grep -E "route53:ChangeResourceRecordSets" /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay/DEPLOY.md && echo "---" && grep -E "9100" /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay/DEPLOY.md | head -3 && echo "---" && grep -E "\\*\\.play\\.esluce\\.net" /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay/DEPLOY.md | head -3</automated>
  </verify>
  <acceptance_criteria>
    - `opt/relay/DEPLOY.md` exists
    - DEPLOY.md contains the **STATIC** DNS note explaining the wildcard A record is static in Phase 68 scope and manual AWS Console setup is acceptable (WARN 8)
    - DEPLOY.md contains the IAM policy block with `route53:ChangeResourceRecordSets` only (no `*` actions)
    - DEPLOY.md contains a security group rule for TCP 9100 with explicit guidance to restrict ingress to the backend's monitoring-service IP range
    - DEPLOY.md contains the verification section with curl commands for `/healthz`, `/metrics`, and the TLS 1.3 check
    - DEPLOY.md contains troubleshooting entries for Caddy cert provisioning, "Connection refused", and NLB health checks
  </acceptance_criteria>
  <done>Operator runbook with AWS NLB + Route 53 (static) + EC2 + IAM + docker deploy + verification; static-DNS note and security-group guidance for :9100 included</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Operator → AWS | Operator uses the AWS Console or aws-cli to provision resources. Authenticated via IAM credentials (out of scope for this plan). |
| Operator → EC2 | Operator SSH's into the EC2 instance as `ec2-user` (Amazon Linux default) to deploy the docker stack. |
| EC2 → AWS Route 53 | EC2's IAM instance profile grants `route53:ChangeResourceRecordSets` for the `esluce.net` zone only. Caddy's `caddy-dns/route53` plugin uses this for DNS-01 challenge TXT records. |
| Backend → EC2 :9100 | Backend's `monitoring_service` (Plan 03 Task 3) scrapes `http://<EC2_IP>:9100/metrics` every 15s. **:9100 is NOT exposed to the public internet** — security group restricts ingress to the backend's monitoring-service IP range. |
| Player → NLB :25565 | NLB is internet-facing; player connections arrive with their public source IP. NLB preserves this and forwards raw TCP to EC2. |
| Agent → Caddy :443 | Agent opens outbound WSS to `wss://relay.esluce.net/tunnel` (TLS 1.3 terminated by Caddy). Caddy reverse-proxies to the gateway on the internal `relay-net` bridge. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-68-33 | Spoofing | IAM role for EC2 | mitigate | Policy grants `route53:ChangeResourceRecordSets` only, scoped to the `esluce.net` hosted zone ARN. No `route53:*`, no `iam:*`, no `s3:*`. |
| T-68-34 | Information Disclosure | :9100 metrics port | mitigate | Security group restricts :9100 ingress to the backend's monitoring-service IP range. Documented in DEPLOY.md. |
| T-68-35 | Tampering | DNS-01 challenge TXT records | mitigate | Created by Caddy on the EC2 instance using the scoped IAM role; deleted after Let's Encrypt validates. Records have short TTL. |
| T-68-36 | Elevation of Privilege | Operator AWS credentials | accept | Out of scope for Phase 68; operator is trusted (per Phase 66 D-06 manual setup pattern). Use AWS SSO + scoped IAM in production. |
| T-68-37 | Tampering | SSH key on EC2 | mitigate | Operator must use a keypair that's restricted to the bastion host or VPN. Standard AWS practice; documented in the EC2 launch parameters. |
| T-68-38 | Denial of Service | NLB flood on :25565 | mitigate | In-process rate limit at the gateway (D-20 RESOLVED, 100 req/min per source IP). NLB itself is DDoS-resistant by design. |

## ASVS L1 Mappings (Phase 68 deploy tier only)

- **V1.4 Access Control:** EC2 security group, IAM role scoping, SSH keypair.
- **V2.1 Authentication:** SSH keypair (public-key only); AWS IAM for the API.
- **V6.2 Cryptographic Practices:** TLS 1.3 at Caddy; SSH keypair; HMAC-SHA256 between gateway and backend.
- **V6.4 Secret Management:** GATEWAY_HMAC_SECRET in AWS Secrets Manager, injected via user-data, never baked into the image.
- **V14.1 Configuration:** All infrastructure provisioning is declarative in DEPLOY.md; no Terraform state in the repo for Phase 68.
</threat_model>

<verification>
After the task completes:

```bash
# 1. DEPLOY.md exists
ls /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay/DEPLOY.md
# Expected: file exists

# 2. Static-DNS note is present (WARN 8 fix)
grep -E "STATIC|static" /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay/DEPLOY.md
# Expected: multiple matches (header + body text)

# 3. IAM policy is scoped (no `*` actions)
grep -E "route53:ChangeResourceRecordSets" /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay/DEPLOY.md
# Expected: 1 match
# Also verify no `route53:*` or `Action.*"\*"` in the policy block
grep -E '"Action":\s*"\*"' /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay/DEPLOY.md
# Expected: 0 matches

# 4. Security group guidance for :9100
grep -E "9100" /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay/DEPLOY.md
# Expected: >= 2 matches (security group rule + verification curl)

# 5. Docker compose reference
grep -E "docker compose up" /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay/DEPLOY.md
# Expected: 1 match

# 6. Handshake-routing verification is documented
grep -E "Handshake|read_mc_handshake_subdomain" /home/rhnbztnl/Downloads/Berguna/Projects/escluse/opt/relay/DEPLOY.md
# Expected: >= 1 match
```

End-to-end deploy behavior requires an actual AWS account, Route 53 hosted zone, and the EC2 instance from step 1. This plan only verifies the runbook is correctly structured.
</verification>

<success_criteria>
- [ ] `opt/relay/DEPLOY.md` exists with all required sections (AWS Setup, Deploy, Verify, Troubleshooting)
- [ ] DEPLOY.md contains the **STATIC** DNS note (WARN 8) explicitly stating manual setup is acceptable in Phase 68 scope
- [ ] DEPLOY.md's IAM policy is scoped to `route53:ChangeResourceRecordSets` only (no `*` actions)
- [ ] DEPLOY.md contains security group guidance that restricts :9100 to the backend's monitoring-service IP range
- [ ] DEPLOY.md references `docker compose up -d --build` from `opt/relay/`
- [ ] DEPLOY.md contains Handshake-routing verification (curl on `/healthz`, `openssl s_client` for TLS 1.3, log grep for `Handshake`)
</success_criteria>

<output>
After completion, create `.planning/phases/68-escluse-relay-infrastructure-objective-implement-esluce-rela/68-04c-SUMMARY.md`
</output>