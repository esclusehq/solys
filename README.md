# Solys — Escluse Agent

## Install

```bash
curl -fsSL https://get.esluce.com/latest/install.sh | bash
```

Binary `escluse-agent` akan terinstall ke `/usr/local/bin`.

### Manual

| Platform | Format | URL |
|----------|--------|-----|
| Linux x86_64 | tarball | `https://get.esluce.com/latest/solys-linux-x86_64.tar.gz` |
| Linux aarch64 | tarball | `https://get.esluce.com/latest/solys-linux-aarch64.tar.gz` |
| Windows x86_64 | zip | `https://get.esluce.com/latest/solys-windows-x86_64.zip` |
| Linux amd64 | deb | `https://get.esluce.com/latest/escluse-agent_amd64.deb` |
| Linux arm64 | deb | `https://get.esluce.com/latest/escluse-agent_arm64.deb` |
| Linux x86_64 | rpm | `https://get.esluce.com/latest/escluse-agent-0.1.0-1.x86_64.rpm` |
| Linux aarch64 | rpm | `https://get.esluce.com/latest/escluse-agent-0.1.0-1.aarch64.rpm` |

### Windows

```powershell
curl.exe -fsSL https://get.esluce.com/latest/install.ps1 | powershell -c -
```

Atau download `solys-windows-x86_64.zip`, extract, jalankan `escluse-agent.exe`.

### Signed Checksums

Semua file diverifikasi dengan SHA256. `SHA256SUMS.txt` ditandatangani dengan Cosign (keyless):

```bash
cosign verify-blob --bundle SHA256SUMS.txt.bundle --certificate-identity-regexp 'esclusehq/solys' --certificate-oidc-issuer https://token.actions.githubusercontent.com SHA256SUMS.txt
```
