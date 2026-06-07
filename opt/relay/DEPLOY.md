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
