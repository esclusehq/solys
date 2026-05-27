# Solys — Escluse Agent

## Installation

```bash
curl -fsSL https://get.esluce.com/latest/install.sh | bash
```

The `escluse-agent` binary will be installed to `/usr/local/bin`.

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

#### Tarball (Linux)

```bash
tar -xzf solys-linux-x86_64.tar.gz
sudo install -m 755 escluse-agent /usr/local/bin/
```

#### DEB (Debian/Ubuntu)

```bash
sudo dpkg -i escluse-agent_amd64.deb
```

#### RPM (Fedora/RHEL)

```bash
sudo dnf install ./escluse-agent-0.1.0-1.x86_64.rpm
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

## Configuration

Create `~/.config/escluse-agent/config.toml`:

```toml
api_key = "esk_your_api_key_here"
backend_url = "wss://app.esluce.com/api/ws/node"
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
