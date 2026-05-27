# Strategi Escluse

## Overview

Escluse adalah platform untuk deploy game server dengan mudah dari VPS, rumah, atau laptop sendiri. Fokus: **simplicity, bukan fitur sebanyak mungkin**.

**Visi**: "Tailscale + Vercel + game server panel"

---

## Market Positioning

### ❌ Jangan vs Calagopus di:

- Benchmark performance
- Rust performance (user tidak peduli 30000% lebih cepat)
- Extension API
- Egg system

### ✅ Lawan di:

- **Simplicity** — deploy 1 klik, server langsung online
- **No port forwarding** — NAT traversal
- **Domain otomatis** — `play.user.esluce.app`
- **UX mainstream** — bukan hanya untuk sysadmin

---

## Perbandingan Market

| Aspek | Escluse | Calagopus |
|-------|---------|-----------|
| **Target User** | mainstream / non-teknikal | teknikal / sysadmin |
| **Pain Point** | "cara bikin server online tanpa ribet" | "Pterodactyl replacement" |
| **Setup Required** | install agent → klik deploy | config node, wings, docker, proxy |
| **Port Forwarding** | tidak perlu | perlu |
| **Domain/DNS** | otomatis | manual |
| **License** | Proprietary (SaaS) | MIT (Open Source) |
| **Billing** | Built-in | Tidak ada |

---

## Fitur Kunci (Differentiatior)

### 1. NAT Traversal

Senjata utama. Agent connect outbound → tidak perlu port forwarding.

```
User install agent → klik deploy → server online
```

Tidak ada panel lain yang focus di sini.

### 2. Domain Otomatis

```
play.user.esluce.app
```

Langsung playable, tidak perlu konfigurasi DNS.

### 3. One-Click Templates

- Paper Minecraft
- Fabric
- Forge
- Modpack (Modrinth integration)
- Palworld
- Bedrock Edition

### 4. Desktop App

Calagopus = panel-centric.
Escluse = **desktop-first** self-hosting platform.

- Install wizard
- System tray
- Auto-start on boot
- Local server discovery

### 5. Lightweight Agent

Sudah ada di arsitektur:
- WebSocket outbound
- NAT-safe
- Docker/Podman orchestration

---

## Pain Terbesar User

```
"cara bikin server online tanpa ribet"
```

Bukan:
- "cara optimasi CPU usage"
- "bagaimana extension API work"
- "Pterodactyl vs Calagopus"

---

## Fitur Priority

### Tier 1 - Must Have (MVP)

- [ ] NAT traversal (outbound connection)
- [ ] One-click Minecraft deploy
- [ ] Domain otomatis
- [ ] Agent installation wizard
- [ ] Basic console

### Tier 2 - Differentiation

- [ ] Desktop app (Electron / Tauri)
- [ ] Modpack templates (Modrinth)
- [ ] Mobile app (Flutter)
- [ ] Team sharing / invite

### Tier 3 - Growth

- [ ] Palworld, Bedrock support
- [ ] Backup system
- [ ] Plugin/mod manager
- [ ] Performance monitoring

---

## Positioning Statement

### ❌ Jangan:

```
"Rust game server panel"
"Modern game panel"
"Open source Pterodactyl replacement"
```

### ✅ Gunakan:

```
"Deploy Minecraft server from home without port forwarding"

atau:

"Cloudflare Tunnel for game servers"
```

---

## Revenue Model

### Private Users
- Free: 1 server
- Starter ($5/mo): 3 servers
- Pro ($15/mo): 10 servers, modpacks

### Hosting Providers (White-label)
- API access
- Custom branding
- Bulk pricing
- Partner dashboard

---

## Kenapa 500M Masih Realistis?

Untuk SaaS niche:
```
300-500 paying users
```
sudah cukup besar.

Yang sulit bukan teknologi — tapi:
- Marketing
- Distribution
- Retention

---

## Competitors

| Product | Market | Kelemahan |
|---------|--------|-----------|
| Calagopus | Self-hosted technical | Setup kompleks |
| Pterodactyl | Self-hosted | PHP, legacy |
| Pelican | Self-hosted | Fork only |
| AMP | Commercial | Free tier terbatas |
| Escluse | SaaS mainstream | Baru |

---

## Kesimpulan

**Jangan jadi "panel game server lagi".**

Jadilah **platform termudah untuk meng-online-kan game server pribadi**.

Kalau berhasil di simplicity?
- Calagopus tetap jadi pilihan teknikal
- Escluse jadi pilihan mainstream
- Market berbeda, tidak perlu saing langsung

---

*Terakhir diupdate: 2026-05-11*