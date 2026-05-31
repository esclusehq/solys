# Umami Analytics Deployment Guide

> Self-hosted web analytics for esluce.com using Umami v3.1.0 on EC2 + RDS PostgreSQL + Caddy reverse proxy.

## Prerequisites

- EC2 instance with Docker and Docker Compose installed
- Cloudflare account with esluce.com zone access (DNS management)
- AWS account with RDS permissions
- SSH access to the EC2 instance
- `openssl` available (for generating APP_SECRET)

---

## Step 1: Provision RDS PostgreSQL

1. Open AWS Console → RDS → Create database
2. Configure the following:
   - **Engine:** PostgreSQL 16
   - **Instance class:** `db.t3.micro` or `db.t3.small`
   - **Storage:** 20 GB gp3
   - **DB instance identifier:** `escluse-analytics`
   - **Master username:** `umami_admin`
   - **Initial database name:** `umami`
3. **Security group (critical):** Add an inbound rule for PostgreSQL (port 5432) that allows access **only from the EC2 security group ID**. Do NOT use `0.0.0.0/0`.
4. Wait for the instance status to become **Available**.
5. Note down the RDS endpoint (hostname) from the RDS console — you will need it for `DATABASE_URL`.

---

## Step 2: Create Database User and Schema (SSH on EC2)

Connect to your EC2 instance via SSH, then use the PostgreSQL client via Docker to set up the database:

```bash
docker run --rm postgres:16-alpine psql "postgresql://umami_admin:YOUR_PASSWORD@analytics-db.REGION.rds.amazonaws.com:5432/umami?sslmode=require"
```

After connecting, run the following SQL commands:

```sql
CREATE USER umami WITH PASSWORD 'your-strong-password';
GRANT ALL PRIVILEGES ON DATABASE umami TO umami;

\c umami

GRANT ALL ON SCHEMA public TO umami;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO umami;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO umami;
```

---

## Step 3: Configure DNS (Cloudflare Dashboard)

1. Open Cloudflare Dashboard → DNS → Records for `esluce.com`
2. Add a new A record:
   - **Name:** `analytics`
   - **IPv4 address:** Your EC2 instance public IP
   - **Proxy status:** Proxied (orange cloud) — enables Cloudflare CDN and DDoS protection
3. Wait a few minutes for DNS propagation.

---

## Step 4: Deploy Umami Stack on EC2

1. SSH into your EC2 instance:

```bash
ssh user@your-ec2-ip
```

2. Create the Umami directory:

```bash
sudo mkdir -p /opt/umami
```

3. Copy the files to the EC2 instance: `docker-compose.yml`, `Caddyfile`, and `env` file (or recreate the files directly on EC2 using your preferred method).

4. Generate a strong APP_SECRET:

```bash
openssl rand -hex 32
```

5. Create `.env` from `.env.example` with your actual values:

```bash
sudo nano /opt/umami/.env
```

Example `.env`:

```ini
DATABASE_URL=postgresql://umami:your-strong-password@analytics-db.xxxxxxx.ap-southeast-1.rds.amazonaws.com:5432/umami?sslmode=require
APP_SECRET=generated-via-openssl-rand-hex-32
TRACKER_SCRIPT_NAME=analytics.js
COLLECT_API_ENDPOINT=/api/collect
DISABLE_TELEMETRY=1
DISABLE_UPDATES=1
```

6. Start the stack:

```bash
cd /opt/umami
sudo docker compose up -d
```

7. Verify both containers are running:

```bash
sudo docker compose ps
```

Expected output — both `umami` and `umami-caddy` containers should show status `Up`.

8. Check Umami logs for successful startup:

```bash
sudo docker compose logs umami
```

Expected: `Running umbrella server at 3000`.

---

## Step 5: First Login and Configuration

1. Open https://analytics.esluce.com in your browser.
2. Login with default credentials:
   - **Username:** `admin`
   - **Password:** `umami`
3. **CHANGE YOUR PASSWORD IMMEDIATELY:**
   - Go to Settings → Profile → Change Password
4. Create websites for each subdomain:
   - Go to Settings → Websites → Add Website
   - Create one entry for each subdomain:
     - `esluce.com` (landing page)
     - `app.esluce.com` (dashboard)
     - `api.esluce.com`
     - `docs.esluce.com`
5. After creating each website, copy its **website ID** (data-website-id) — you will need it for Step 6.

---

## Step 6: Inject Tracking Scripts into Frontends

Add the Umami tracking script to each frontend's `index.html` file.

### Landing Page (`landing-page-escluse/index.html`)

```html
<script
    defer
    src="https://analytics.esluce.com/analytics.js"
    data-website-id="WEBSITE_ID_FOR_ESLUCE_COM"
    data-domains="esluce.com"
></script>
```

### App Dashboard (`app/index.html`)

```html
<script
    defer
    src="https://analytics.esluce.com/analytics.js"
    data-website-id="WEBSITE_ID_FOR_APP_ESLUCE_COM"
    data-domains="app.esluce.com"
></script>
```

> **Note:** Both landing page and app are React SPAs. Umami's tracking script automatically monitors SPA navigation via the History API (`pushState`/`replaceState` + `popstate`). Do NOT call `umami.track()` manually for page views — it will cause double-counting.

---

## Verification

After deployment, run these curl commands to verify the stack is working:

```bash
# Health check endpoint
curl -I https://analytics.esluce.com/api/heartbeat

# Tracking script accessible
curl -sI https://analytics.esluce.com/analytics.js

# Security headers present
curl -sI https://analytics.esluce.com | grep -i 'strict-transport-security'

# TLS certificate valid
curl -vI https://analytics.esluce.com 2>&1 | grep -i 'ssl'
```

---

## Troubleshooting

### Container won't start — database connection error
- Verify `DATABASE_URL` in `.env` is correct
- Check RDS security group: port 5432 must allow inbound from the EC2 security group
- Ensure `sslmode=require` is in the connection string
- Confirm the `umami` database and user exist on RDS with proper permissions

### No tracking data appears in dashboard
- Verify that a website was created in Umami (Settings → Websites)
- Confirm `data-website-id` in the tracking script matches the created website's ID
- Check browser console for errors loading `analytics.js`

### Double pageviews in analytics
- Remove any manual `umami.track()` calls from React components
- Umami's script already auto-tracks SPA navigation via the History API

### Tracking script blocked by ad blockers
- Verify `TRACKER_SCRIPT_NAME` is set to `analytics.js` (or another non-obvious name) — NOT `umami.js` or `script.js`
- Verify `COLLECT_API_ENDPOINT` is set to `/api/collect` (not the default `/api/send`)
- After changing these, restart the stack: `sudo docker compose restart`

### TLS errors on first visit
- DNS for `analytics.esluce.com` must resolve before Caddy can provision Let's Encrypt certificates
- Wait a few minutes for DNS propagation, then check Caddy logs:
  ```bash
  sudo docker compose logs umami-caddy
  ```
- Caddy auto-provisions certificates on first HTTP request; the first few requests may fail with TLS warnings

---

> **Security notes:**
> - RDS port 5432 is restricted to EC2 security group only (not `0.0.0.0/0`)
> - All database connections use `sslmode=require` (TLS encryption)
> - Caddy auto-redirects HTTP to HTTPS with HSTS header
> - Default Umami admin password (`admin`/`umami`) must be changed on first login
> - Server version header is stripped via Caddy's `-Server` directive
