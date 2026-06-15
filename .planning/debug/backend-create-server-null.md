---
status: diagnosed
trigger: "perbaiki backend yang mengembalikan nilai null saat create server bedrock"
created: 2026-06-12
updated: 2026-06-13
---

## Symptoms

1. **Expected behavior**: Creating a Bedrock server should return the created server object
2. **Actual behavior**: Backend returns null (`[CreateServerModal] Server created: null`)
3. **Error messages**: No visible error toast or error message in console logs - just null response
4. **Timeline**: Started after adding PocketMine-MP and Nukkit variants (quick task 260613-5a9)
5. **Reproduction**: Open Create Server modal, select "Minecraft Bedrock" game type, select any variant (default/pocketmine/nukkit), fill form fields, click "Create Server"

### Full Logs
```
[CreateServerModal] Loaded nodes: Array [ {…} ]
[CreateServerModal] Loaded templates: Array(11) [ {…}, {…}, {…}, {…}, {…}, {…}, {…}, {…}, {…}, {…}, … ]
[CreateServerModal] Game type changed: bedrock variants: 3
[CreateServerModal] Template selected: Minecraft Bedrock
[CreateServerModal] handleSubmit called Object { name: "testke2", existingServers: 0 }
[CreateServerModal] Creating server with data: Object { name: "testke2", port: "19132", nodeId: "" }
[CreateServerModal] Calling serversApi.create
[CreateServerModal] Server created: null
```

## Root Cause Analysis

### Primary: Quota limits are 0 for hobby plan

Backend logs from `docker logs escluse_backend` confirm both `testke1` and `testke2` failed at quota check:

```
[quota] Subscription found: true
[quota] Plan found: name=hobby
Quota check passed: allowed=false
QUOTA EXCEEDED: reasons=[
  QuotaViolation { resource: "ram_mb", current: 0, requested: 2048, limit: 0 },
  QuotaViolation { resource: "cpu_cores", current: 0, requested: 1, limit: 0 },
  QuotaViolation { resource: "disk_gb", current: 0, requested: 5, limit: 0 }
]
```

All limits are **0** for the hobby plan (ram=0, cpu=0, disk=0). The handler returns `Err(format!("QUOTA_EXCEEDED: ..."))` which Axum converts to HTTP 500 with a non-JSON body (`String`).

**Why frontend shows `null` instead of an error**: The API client in `app/src/lib/api.js:53-58` calls `response.json()` which fails (body is plain text), `.catch(() => null)` returns null for `data`. Then `!response.ok` is true (500), so it throws `Error("Request failed: 500")`. This SHOULD show a toast error — the user may have seen it and focused on the `Server created: null` log line instead of the error.

### Secondary: `server_type` always `undefined` for bedrock

In `CreateServerModal.jsx:308`:
```js
server_type: gameType === 'bedrock' ? undefined : serverType,
```

This means no variant template info reaches the backend. However, the domain DTO (`api/src/domain/server/model.rs`) doesn't have a `server_type` field at all — it has `#[serde(default)]` which silently drops unknown fields. So even if `server_type` were set, it would be ignored.

### Key Observations
1. The **hobby plan has zero resource limits** in the database — this is the blocker
2. The frontend `server_type: undefined` for bedrock is a secondary issue
3. Even `testke1` (same payload) also fails with quota exceeded in the same backend session
4. The error manifests as `null` because the 500 error body is plain text, not JSON

**Bottleneck**: `api/src/domain/usage/service.rs` — `QuotaService` checks plan limits which return 0 for hobby

**Evidence files**:
- `api/src/presentation/handlers/server_handlers.rs` — quota check at lines ~490-516
- `api/src/domain/usage/service.rs` — QuotaService logic
- `api/src/domain/usage/model.rs` — Plan/Quota models and DB queries
- `app/src/features/server/CreateServerModal.jsx` — handleSubmit line 308
- `app/src/lib/api.js` — API client response parsing lines 53-58
