# Fix HTTP 525 — Backend `api.esluce.com` Tidak Reachable

## Ringkasan Diagnostik

| Pemeriksaan | Hasil |
|------------|-------|
| DNS `api.esluce.com` | ✅ Resolve ke Cloudflare (`172.67.171.10`, `104.21.63.158`) |
| Relay VPS → Cloudflare (TLS) | ✅ Handshake sukses, certificate Let's Encrypt valid |
| Relay VPS → Cloudflare (HTTP) | ✅ Response **525** dalam 39ms |
| Main app EC2 (`100.121.160.102`) — ping | ✅ Reachable via Tailscale, 0% loss |
| Main app EC2 — port **80** | ✅ **nginx/1.31.1** — serving landing page |
| Main app EC2 — port **443** | ❌ TLS handshake gagal — `tlsv1 alert internal error` |
| Main app EC2 — port **3000** | ✅ API server (axum) running, tapi error `ConnectInfo` |
| Main app EC2 — port **2375** | ❌ Docker API tidak ter ekspos |

## Root Cause

Dua masalah:

### 1. Cloudflare SSL/TLS mode salah

Cloudflare saat ini diset ke **Full** atau **Full (strict)**, tapi port 443 di origin server **gagal SSL handshake** (internal error). Ini menyebabkan **HTTP 525**.

### 2. Nginx di port 80 belum proxy `/internal/relay/authorize`

Bahkan kalau pake Flexible mode (HTTP ke port 80), nginx return **405 Not Allowed** untuk POST ke `/internal/relay/authorize`. Berarti endpoint `/internal/relay/authorize` belum ditambahkan ke konfigurasi nginx.

---

## Step-by-Step Perbaikan

### Step 1: SSH ke origin server (main app EC2)

```bash
ssh -i ~/Downloads/sqrt1mReyhan.pem ec2-user@100.121.160.102
```

### Step 2: Cek nginx config dan tambah proxy untuk /internal/

```bash
# Cek lokasi file config nginx
sudo nginx -t 2>&1
# atau
sudo ls -la /etc/nginx/conf.d/
sudo ls -la /etc/nginx/sites-enabled/

# Lihat config yang ada
sudo cat /etc/nginx/conf.d/default.conf
# atau
sudo cat /etc/nginx/sites-enabled/default
```

**Tambahkan location block ini ke config nginx** (server block untuk `api.esluce.com` atau port 80):

```nginx
location /internal/ {
    proxy_pass http://127.0.0.1:3000;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
}
```

```bash
# Test config
sudo nginx -t

# Reload nginx
sudo nginx -s reload
```

### Step 3: Cek apakah `ConnectInfo` extension terdaftar di API

Di kode backend (main app), cari file `main.rs` atau `server.rs` atau `router.rs`:

```bash
# Cari file yang setup axum router
grep -r "ConnectInfo\|into_make_service_with_connect_info\|add_extension" /opt/escluse/ --include="*.rs"
```

Pastikan ada `layer(ConnectInfo::<SocketAddr>::new(...))` atau cara setup yang sesuai.

### Step 4: Set Cloudflare SSL/TLS ke **Flexible**

Buka **[dash.cloudflare.com](https://dash.cloudflare.com)** → pilih domain **esluce.com** → **SSL/TLS** → **Overview** → pilih **Flexible**.

Ini adalah solusi **paling cepat** karena:
- Cloudflare akan connect ke origin via **HTTP di port 80** (bukan HTTPS di port 443)
- Tidak perlu memperbaiki SSL di port 443
- Traffic antara client dan Cloudflare tetap terenkripsi

> ⚠️ Flexible berarti traffic antara Cloudflare dan origin server TIDAK terenkripsi. Ini aman karena relay VPS dan main app EC2 berada di VPC yang sama via Tailscale.

### Step 5: Verifikasi dari relay VPS

```bash
# Setelah Flexible + nginx fix, test dari local
ssh -i ~/Downloads/EsluceRelay\(1\).pem ec2-user@100.94.90.104 \
  'curl -sS -w "\nHTTP: %{http_code}\n" -X POST \
    -H "Content-Type: application/json" \
    -d "{\"relay_token\":\"0c127f92-d2e5-4bc3-8cac-74c470a44ef5\"}" \
    https://api.esluce.com/internal/relay/authorize'
```

Harusnya return **HTTP 200** atau **HTTP 401/403** (bukan 525).

### Step 6: Restart relay gateway

```bash
ssh -i ~/Downloads/EsluceRelay\(1\).pem ec2-user@100.94.90.104 \
  'cd ~/escluse && docker compose -f opt/relay/docker-compose.yml restart relay-gateway'
```

### Step 7: Cek tunnel connect

```bash
# Tunggu ~30s, lalu cek
sleep 30 && \
ssh -i ~/Downloads/EsluceRelay\(1\).pem ec2-user@100.94.90.104 \
  'docker logs relay-gateway 2>&1 | grep -E "TunnelConnect|Tunnel registered|\[AUTH\]"'
```

---

## Ringkasan Perubahan yang Sudah Saya Lakukan (Code)

| File | Perubahan |
|------|-----------|
| `opt/relay/src/tunnel.rs` | Fix tunnel hang — drain DATA frame setelah `session.next()` |
| | Allow empty subdomain |
| `opt/relay/src/auth.rs` | Redis cache auth (TTL 5 menit) |
| | Retry 3× exponential backoff (1s, 2s, 4s) |
| | Stale cache fallback |
| `opt/relay/src/state.rs` | Redis connection pakai `Mutex` untuk shared access |
| `FIX-BACKEND-525.md` | Guide troubleshooting + step-by-step fix |

---

## Yang Perlu Kamu Lakukan

| No | Task | Server | Waktu |
|----|------|--------|-------|
| 1 | SSH ke main app EC2 (`100.121.160.102`) | Main app EC2 | 1 menit |
| 2 | Cek nginx config, tambah proxy `/internal/` | Main app EC2 | 5 menit |
| 3 | Buka Cloudflare Dashboard → set SSL ke **Flexible** | Cloudflare | 2 menit |
| 4 | Verifikasi dari relay VPS | Local | 1 menit |
| 5 | Restart relay gateway (saya bisa lakukan) | Relay VPS | 10 detik |

Kamu mulai dari **Step 1** sampai **Step 4**, lalu bilang saya untuk **Step 5** (restart gateway + verifikasi).
