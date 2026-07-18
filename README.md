# Solys — Escluse Agent

## Installation

```bash
curl -fsSL https://get.esluce.com/latest/install.sh | bash
```

The `escluse-agent` binary will be installed to `/usr/local/bin` (or `$PREFIX/bin` on Termux/Android).

### Manual

| Platform | Format | Download |
|----------|--------|----------|
| Linux x86_64 | tarball | [solys-linux-x86_64.tar.gz](https://get.esluce.com/latest/solys-linux-x86_64.tar.gz) |
| Linux aarch64 | tarball | [solys-linux-aarch64.tar.gz](https://get.esluce.com/latest/solys-linux-aarch64.tar.gz) |
| Linux amd64 | deb | [escluse-agent_amd64.deb](https://get.esluce.com/latest/escluse-agent_amd64.deb) |
| Linux arm64 | deb | [escluse-agent_arm64.deb](https://get.esluce.com/latest/escluse-agent_arm64.deb) |
| Linux x86_64 | rpm | [escluse-agent-0.1.0-1.x86_64.rpm](https://get.esluce.com/latest/escluse-agent-0.1.0-1.x86_64.rpm) |
| Linux aarch64 | rpm | [escluse-agent-0.1.0-1.aarch64.rpm](https://get.esluce.com/latest/escluse-agent-0.1.0-1.aarch64.rpm) |
| Windows x86_64 | zip | [solys-windows-x86_64.zip](https://get.esluce.com/latest/solys-windows-x86_64.zip) |
| Android aarch64 (Termux) | tarball | [solys-android-aarch64.tar.gz](https://get.esluce.com/latest/solys-android-aarch64.tar.gz) |
| Android armv7 (Termux) | tarball | [solys-android-armv7.tar.gz](https://get.esluce.com/latest/solys-android-armv7.tar.gz) |

#### Tarball (Linux)

x86_64:

```bash
tar -xzf solys-linux-x86_64.tar.gz
sudo install -m 755 escluse-agent /usr/local/bin/
```

aarch64:

```bash
tar -xzf solys-linux-aarch64.tar.gz
sudo install -m 755 escluse-agent /usr/local/bin/
```

#### DEB (Debian/Ubuntu)

amd64:

```bash
sudo dpkg -i escluse-agent_amd64.deb
```

arm64:

```bash
sudo dpkg -i escluse-agent_arm64.deb
```

#### RPM (Fedora/RHEL)

x86_64:

```bash
sudo dnf install ./escluse-agent-0.1.0-1.x86_64.rpm
```

aarch64:

```bash
sudo dnf install ./escluse-agent-0.1.0-1.aarch64.rpm
```

Atau:

```bash
sudo rpm -ivh escluse-agent-0.1.0-1.x86_64.rpm
```

#### ZIP (Windows)

Extract `solys-windows-x86_64.zip` and run `escluse-agent.exe`.

Or using PowerShell:

```powershell
curl.exe -fsSL https://get.esluce.com/latest/install.ps1 | powershell -c -
```

#### Tarball (Android/Termux)

aarch64 (most devices):

```bash
tar -xzf solys-android-aarch64.tar.gz
install -m 755 escluse-agent $PREFIX/bin/
```

armv7 (older devices):

```bash
tar -xzf solys-android-armv7.tar.gz
install -m 755 escluse-agent $PREFIX/bin/
```

---

### Android (Termux)

Run the agent on your Android phone or tablet directly via Termux.

#### Prerequisites

1. Install [Termux](https://f-droid.org/en/packages/com.termux/) from F-Droid (recommended) or GitHub Releases
2. Update packages:
   ```bash
   pkg update && pkg upgrade
   ```

#### Quick Install

```bash
curl -fsSL https://get.esluce.com/latest/install.sh | bash
```

Or using wget:

```bash
wget -qO- https://get.esluce.com/latest/install.sh | bash
```

The script auto-detects Termux, installs the binary to `$PREFIX/bin`, and prompts for your API key.

#### Manual Install

Download the tarball matching your device architecture (see table above), then:

```bash
tar -xzf solys-android-aarch64.tar.gz
install -m 755 escluse-agent $PREFIX/bin/
```

#### Running

```bash
escluse-agent
```

The agent automatically detects Termux on first run and installs Java 17 if missing. When you create a Minecraft server, it picks the right Java version for your MC version — upgrading to Java 21 on the fly if you're running 1.21+.

---

## Configuration

Create the config file:

**Linux:**
```bash
mkdir -p ~/.config/escluse-agent && cat > ~/.config/escluse-agent/config.toml << 'EOF'
[server]
api_key = "esk_your_api_key_here"
backend_url = "wss://app.esluce.com/api/ws/node"
EOF
```

**Windows (PowerShell):**
```powershell
New-Item -Type Directory -Force "$env:APPDATA\escluse-agent"
@"
[server]
api_key = "esk_your_api_key_here"
backend_url = "wss://app.esluce.com/api/ws/node"
"@ | Set-Content "$env:APPDATA\escluse-agent\config.toml"
```

**Android (Termux):**
```bash
mkdir -p ~/.config/escluse-agent && cat > ~/.config/escluse-agent/config.toml << 'EOF'
[server]
api_key = "esk_your_api_key_here"
backend_url = "wss://app.esluce.com/api/ws/node"
EOF
chmod 600 ~/.config/escluse-agent/config.toml
```

Get your API key from the [Escluse Dashboard](https://app.esluce.com) → **Nodes** → **Add Node**.

Alternatively, set the `ESCLUSE_AGENT_API_KEY` environment variable:

```bash
export ESCLUSE_AGENT_API_KEY="esk_your_api_key_here"
escluse-agent
```

### Signed Checksums

All files are verified with SHA256. `SHA256SUMS.txt` is signed with Cosign (keyless):

```bash
cosign verify-blob --bundle SHA256SUMS.txt.bundle --certificate-identity-regexp 'esclusehq/solys' --certificate-oidc-issuer https://token.actions.githubusercontent.com SHA256SUMS.txt
```
