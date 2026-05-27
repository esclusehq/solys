# Phase 20: streamline-agent-installation-di-vps - Context

**Gathered:** 2026-04-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Streamline agent installation on VPS through better UI/UX in node registration flow.

</domain>

<decisions>
## Implementation Decisions

### UI Improvements (D-01)
- **Token Copy:** One-click copy button in registration modal
- **Download Link:** Link to download agent binary/docker-compose
- **Installation Guide:** Simple 3-step guide in modal

### Existing Backend (Already Working)
- Registration tokens: `/api/v1/nodes/{id}/tokens` (POST)
- Token validation: `/api/v1/nodes/register` (POST)
- Expiry: Configurable (default 24h)

### Implementation Details
1. Add "Get Installation Token" button to NodeDetails  
2. Modal shows: token, copy button, download link, quick guide
3. Copy button: single click to copy to clipboard
4. Download link: points to agent download (or docker-compose.yml)
5. Install guide: 3 lines max

</decisions>

<canonical_refs>
## Reference

**Backend Already Exists:**
- `node_registration_token_handlers.rs` - Token generation
- `NodeDetails` component in Nodes.jsx - Where to add button

**Frontend Patterns:**
- Modal component (already in use for API keys, tokens)
- Copy to clipboard use: `navigator.clipboard.writeText()`

</canonical_refs>

<specifics>
## Specific Ideas

**Installation Modal Content:**
```
1. Copy registration token below
2. Download agent: [Download Link]
3. Run: curl -sL <agent-url> | bash -s --token <token>
```

Or for Docker:
```
docker run -e TOKEN=<token> -v /var/run/docker.sock:/var/run/docker.sock eclipsesolo/agent
```

</specifics>

<deferred>
## Deferred Ideas

- Auto-detection of VPS environment (CPU, RAM, Docker)
- One-click install script generation
- Cloud-init integration for managed VPS

</deferred>

---

*Phase: 20-streamline-agent-installation-di-vps*
*Context gathered: 2026-04-18*