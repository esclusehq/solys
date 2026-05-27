# Phase 51 — Automasi DNS Cloudflare: CONTEXT

## Vision
Agent secara otomatis menghubungkan domain `*.esluce.com` ke IP client via Cloudflare API agar Minecraft (dan game server lainnya) bisa online ke public tanpa user perlu ribet setup DNS manual.

## Flow
- **Hybrid trigger**: DNS record bisa dibuat dari dashboard (backend kirim instruksi ke agent) atau langsung dari agent config
- **API Token**: Disimpan di database backend, user input via dashboard
- **Domain Source**: Wildcard `*.esluce.com` (kita provisioning penuh)

## Domain Structure
- **Internal/otomatis**: `node1.esluce.com:25565` — per node, port per server
- **Custom (optional)**: `survival.esluce.com` — custom name diarahkan ke node tertentu

## Auto-refresh (DDNS-like)
- Agent detect IP change → otomatis update DNS record via Cloudflare API
- Berjalan background, periodic check

## Scope Termasuk
1. Provisioning wildcard zone `*.esluce.com` di Cloudflare (zone, DNS records, SSL/TLS)
2. Backend: API endpoint untuk manage Cloudflare token, domain mapping, kirim instruksi ke agent
3. Agent: Cloudflare API client untuk create/update/delete DNS records, auto-refresh IP
4. Dashboard: UI input Cloudflare token, domain management, status DNS records

## Key Decisions
- `*.esluce.com` sebagai wildcard domain utama
- Full automasi provisioning wildcard
- Auto-refresh IP (DDNS-like)
- Hybrid flow (dashboard + agent config)
- API token di database backend
