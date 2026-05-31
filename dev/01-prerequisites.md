# Prerequisites

Before setting up the Esluce development environment, ensure the following tools are installed on your system.

## Tool Prerequisites

| Tool            | Minimum Version | Purpose                                        | Verification Command      |
|-----------------|-----------------|------------------------------------------------|---------------------------|
| Docker          | 24+             | Container runtime for PostgreSQL & Redis        | `docker --version`        |
| Docker Compose  | v2 (standalone or plugin) | Multi-container orchestration         | `docker compose version`  |
| Node.js         | 20+             | Frontend build and dev server                  | `node --version`          |
| npm             | 10+             | Frontend dependency management                 | `npm --version`           |
| Rust            | 1.70+ (edition 2021) | Backend API, Worker, Agent                | `rustc --version`         |
| Cargo           | 1.70+           | Rust package manager                           | `cargo --version`         |
| rustup          | latest          | Rust toolchain installer                       | `rustup --version`        |
| Supabase CLI    | latest          | Local Supabase Auth & DB                       | `supabase --version`      |

## OS-Specific Install Commands

#### Linux (apt — Debian/Ubuntu)

```bash
# Docker (if not installed)
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER
# Log out and back in for group changes to take effect

# Node.js via nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
nvm install 20

# Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable

# Supabase CLI
npm install -g supabase
# Or via brew (if brew is installed): brew install supabase/tap/supabase
```

#### macOS (Homebrew)

```bash
# Docker Desktop
brew install --cask docker

# Node.js
brew install node@20

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Supabase CLI
brew install supabase/tap/supabase
```

#### Windows (winget)

```powershell
# Docker Desktop
winget install Docker.DockerDesktop

# Node.js
winget install OpenJS.NodeJS.LTS

# Rust
# Download from https://rustup.rs
# Or: winget install Rustlang.Rustup

# Supabase CLI
npm install -g supabase
```

> **Note:** After installing Docker Desktop on Windows, ensure WSL2 is configured and Docker is running (look for the whale icon in the system tray).

## Version Verification

Copy and paste the following block to verify all tools are installed correctly:

```bash
echo "=== Docker ===" && docker --version && docker compose version
echo "=== Node.js ===" && node --version
echo "=== Rust ===" && rustc --version && cargo --version
echo "=== Supabase ===" && supabase --version
echo "=== All tools verified ==="
```

## Troubleshooting

If any command fails, see [05-troubleshooting.md](05-troubleshooting.md) or refer to the tool's official installation guide.
