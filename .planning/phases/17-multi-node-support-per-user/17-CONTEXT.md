# Phase 17: multi-node-support-per-user - Context

**Gathered:** 2026-04-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Allow users to own and manage multiple nodes (agent instances) for their game server infrastructure.

</domain>

<decisions>
## Implementation Decisions

### Node Assignment Model (D-01)
- **Decision:** Direct ownership model
- Users "own" specific nodes they have registered
- Full control over owned nodes (view stats, manage API keys, delete)

### User Quota/Limits (D-02)
- **Decision:** Tier-based limit based on subscription plan
- Starter plan: 1 node
- Pro plan: 3 nodes
- (Can be extended in future based on billing integration)

### Server Placement (D-03)
- **Decision:** Auto-placement
- When creating a server, system automatically selects best available node from user's owned nodes
- Factors: node health, available resources, current server load

### Node Visibility (D-04)
- **Decision:** Owned nodes only
- Users only see and interact with nodes they own
- Nodes page filters to user's owned nodes only

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing Patterns
- `api/migrations/20260324000004_create_plans_table.sql` — Existing plans (starter, pro) with server limits
- `api/migrations/20260309000001_create_nodes_tables.sql` — Node table structure
- `api/src/presentation/handlers/webhook_handlers.rs` — Example of user-scoped query (find_by_user_id)
- `app/src/hooks/useNodes.js` — Node data fetching patterns

### Integration Points
- Billing/subscription tier → node quota mapping
- Node ownership stored in nodes table (user_id field already exists)
- Auto-placement logic in create_server flow

</canonical_refs>

<specifics>
## Specific Ideas

- Node quota check happens at node registration time
- Auto-placement picks node with lowest current server count
- User can still manually select node in "advanced" server creation options
- Add "node_quota" field to plans table or use existing server limit

</specifics>

<deferred>
## Deferred Ideas

- Multi-node load balancing (auto-scale across nodes) — future phase
- Node groups/clusters — future phase

</deferred>

---

*Phase: 17-multi-node-support-per-user*
*Context gathered: 2026-04-16*