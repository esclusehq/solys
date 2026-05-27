# Phase 45: OBSERVABILITY - Context

**Gathered:** 2026-05-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Enhance observability - alerting thresholds, Grafana integration, distributed tracing.
</domain>

<decisions>
## Implementation Decisions

### Current State
- ✅ metrics.report handler exists
- ✅ /metrics endpoint with Prometheus format
- ✅ Heartbeat includes metrics
- ✅ agent-metrics crate

### New Features (D-01 to D-04)

- **D-01:** Add alerting thresholds - CPU > 80%, Memory > 85%, Disk > 90% trigger alerts
- **D-02:** Grafana integration - dashboard JSON + prometheus scrape config
- **D-03:** Distributed tracing - trace ID per request through handler chain
- **D-04:** Alert payload sent via existing heartbeat or new topic

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

- `web-agent/src/handlers/metrics.rs` - existing metrics handler
- `web-agent/src/api/routes.rs` - /metrics endpoint
- `web-agent/src/agent_connection.rs` - heartbeat with metrics
- `agent-core/crates/agent-metrics/src/` - metrics collection

</canonical_refs>

<specifics>
## Specific Ideas

Want full enhancement:
1. Alerting thresholds (CPU, Memory, Disk)
2. Grafana integration (dashboard + scrape config)
3. Distributed tracing (trace ID per request)

</specifics>

<deferred>
## Deferred Ideas

None

</deferred>

---

*Phase: 45-observability*
*Context gathered: 2026-05-03*