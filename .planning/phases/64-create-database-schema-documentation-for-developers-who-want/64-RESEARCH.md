# Phase 64: Create Database Schema Documentation — Research

**Researched:** 2026-05-31
**Domain:** PostgreSQL schema introspection, Rust CLI tooling, Mermaid ER diagram generation, developer documentation
**Confidence:** HIGH

## Summary

This phase delivers a `DATABASE_SCHEMA.md` at repo root and a Rust CLI generator (`tools/db-schema-gen/`) that produces it. The generator introspects a live PostgreSQL database via `information_schema`, reads rustdoc annotations from entity structs in `api/src/domain/`, and generates Mermaid ER diagrams + markdown column-level tables organized by business domain.

The schema has ~30+ tables across 8 business domains (Servers, Nodes, Users/Auth, Billing/Subscriptions, Backups, Settings/Config, Events/Logs, Jobs). The generator reads both the live database schema AND the Rust entity source code to produce a developer-facing document that explains not just what each column is, but why the schema is structured that way.

**Primary recommendation:** Build `tools/db-schema-gen/` as a standalone Rust CLI binary (Cargo workspace member) using `sqlx` + `tokio` for DB introspection + `clap` for CLI args. Parse rustdoc annotations via `regex` from source files in `api/src/domain/`. Generate a single `DATABASE_SCHEMA.md` following the existing repo-root doc style (ATX headings, GFM tables, Mermaid fenced blocks).

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Single `DATABASE_SCHEMA.md` at repo root. No docs site integration.
- **D-02:** **Mermaid ER diagrams** per relationship cluster + **markdown tables** per table (column name, type, constraints, description).
- **D-03:** ER diagrams grouped by natural FK relationship clusters (the agent identifies the clusters from the schema), not one big diagram nor per-domain rigidly.
- **D-04:** Tables grouped **by business domain** — Servers, Nodes, Billing/Subscriptions, Users/Auth, Backups, Settings/Config, Events/Logs, Jobs, and any others the agent identifies.
- **D-05:** Each domain section includes: brief domain description, ER diagram for relationship cluster, then markdown tables per table.
- **D-06:** Each domain section includes **common query patterns** and **design rationale** — why the schema is structured as it is.
- **D-07:** **Current schema snapshot only.** Document what exists now.
- **D-08:** **Rust CLI tool** (`tools/db-schema-gen/`) that: Connects to PostgreSQL and introspects the live schema via `information_schema`; Reads rustdoc annotations from Rust entity structs in `api/src/domain/` for narrative descriptions and query patterns; Generates `DATABASE_SCHEMA.md` with Mermaid ER diagrams and markdown tables.
- **D-09:** The generated `DATABASE_SCHEMA.md` is committed to the repo. Developers regenerate when schema changes.
- **D-10:** Tool name, location (`tools/db-schema-gen/` is presumed), and Rust dependencies are the agent's discretion.

### The Agent's Discretion
- Exact Mermaid ER diagram type (`erDiagram` syntax)
- Relationship cluster boundaries (what connects to what)
- Query patterns to include per domain section
- Rust crate dependencies for the generator tool
- Markdown formatting and heading levels
- Whether to include an index/table of contents
- Whether to include a "Schema Version" or "Last Generated" timestamp

### Deferred Ideas (OUT OF SCOPE)
- **Generator tool docs site integration** — cross-linking DATABASE_SCHEMA.md from docs.esluce.com was considered but declined. Repo-root only.
- **Full schema migration history inline** — current snapshot only was preferred; migration history remains in `api/migrations/`.
</user_constraints>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Schema introspection | CLI tool (generator) | — | Runs as developer tool, not a service; queries `information_schema` directly |
| Rustdoc annotation reading | CLI tool (generator) | — | Reads source files from `api/src/domain/` — no runtime DB needed for this part |
| Source of truth for schema shape | Database (PostgreSQL) | Migrations (api/migrations/) | The live DB is the ultimate truth; migrations define its evolution |
| Source of truth for narrative docs | Rust entity source files | — | Rustdoc annotations (`///` comments) on structs provide descriptions, rationale, query patterns |
| Output document | Repo root (`DATABASE_SCHEMA.md`) | — | Single committed file at repo root per D-01 |
| Mermaid rendering | GitHub (rendering engine) | — | No build step — GitHub renders Mermaid natively from fenced code blocks |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `sqlx` | 0.7 | PostgreSQL connectivity + introspection | Already the project's ORM; `information_schema` queries are standard SQL |
| `tokio` | 1 | Async runtime | Already used by all Rust services in the project |
| `clap` | 4 | CLI argument parsing | De-facto standard Rust CLI framework; handles `--db-url`, `--output`, `--help` |
| `regex` | 1 | Rustdoc annotation parsing | Extract `///` doc comments from Rust source files |
| `anyhow` | 1 | Error handling | Already used across the project |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `serde` / `serde_json` | 1 | JSON handling for query patterns in entity structs | If entity structs embed JSON config data |
| `walkdir` | 2 | Recursive file traversal | Finding all `.rs` files in `api/src/domain/` |
| `chrono` | 0.4 | Timestamp handling | For "Last Generated" timestamp in output |
| `indoc` | 2 | Multi-line string formatting | Clean template strings for Mermaid diagrams |
| `unicode-width` | 0.1 | Column alignment in markdown tables | If generating GFM tables with precise column widths |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `sqlx` | `tokio-postgres` | `tokio-postgres` is lower-level; `sqlx` keeps consistent with project stack |
| `regex` | `syn` (Rust AST parser) | `syn` is heavier and more complex; regex on doc comments is sufficient since we only need `///` lines |
| Standalone binary | Bash script + `psql` | Bash would be fragile; Rust gives structured data processing, type safety, and cross-platform |
| Standalone binary | Python script | Project is Rust-first; Rust avoids dependency on Python being installed |
| `clap` | `argh` | `clap` is more widely used and has better subcommand support |

**Installation:**
```bash
# Not installed as a dependency — it's a Cargo workspace member
cargo run --manifest-path tools/db-schema-gen/Cargo.toml -- --db-url "$DATABASE_URL"
```

**Version verification [VERIFIED: cargo registry — sqlx 0.7, clap 4.x, regex 1.x are current stable releases]:**
- `sqlx` 0.7 — confirmed in `api/Cargo.toml`
- `tokio` 1 — confirmed in `api/Cargo.toml`
- `clap` 4 — latest stable as of 2026
- `regex` 1 — standard Rust crate

## Architecture Patterns

### System Architecture

The generator is a standalone CLI tool — it has no runtime dependencies on the API. It reads two sources:

1. **Live PostgreSQL** (via `information_schema`): Tables, columns, types, nullability, defaults, constraints, foreign keys, indexes
2. **Rust source files** (via regex parsing): `///` doc comments on entity structs for descriptions, query patterns, design rationale

```
┌─────────────────────────────────────────────────────────────────────┐
│                     tools/db-schema-gen/                             │
│                                                                      │
│  ┌──────────────┐     ┌─────────────────────┐     ┌──────────────┐  │
│  │ CLI (clap)   │────▶│ Introspector        │────▶│ Markdown     │  │
│  │ --db-url     │     │                     │     │ Generator    │  │
│  │ --output     │     │ ┌─────────────────┐ │     │              │  │
│  │ --source-dir │     │ │ information_    │ │     │ ┌──────────┐ │  │
│  └──────────────┘     │ │ schema queries  │ │     │ │ Mermaid  │ │  │
│                        │ └─────────────────┘ │     │ │ erDiagram│ │  │
│  ┌──────────────┐     │ ┌─────────────────┐ │     │ └──────────┘ │  │
│  │ Rustdoc      │────▶│ │ rustdoc parser   │ │     │ ┌──────────┐ │  │
│  │ Reader        │     │ │ (regex)          │ │     │ │ GFM     │ │  │
│  │ (walkdir)    │     │ └─────────────────┘ │     │ │ tables   │ │  │
│  └──────────────┘     └─────────────────────┘     │ └──────────┘ │  │
│                                                     └──────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌─────────────────────┐
                    │  DATABASE_SCHEMA.md  │
                    │  (committed to repo  │
                    │   root, regenerated  │
                    │   on schema changes) │
                    └─────────────────────┘
```

### Recommended Project Structure

```
tools/db-schema-gen/
├── Cargo.toml
└── src/
    ├── main.rs                  # CLI entry point with clap
    ├── introspector.rs          # PostgreSQL information_schema queries
    ├── rustdoc_reader.rs        # Parse /// comments from .rs files
    ├── domain_classifier.rs     # Map table names → business domains
    ├── mermaid_generator.rs     # Build erDiagram blocks
    ├── markdown_builder.rs      # Compose sections, tables, diagrams
    └── types.rs                 # Internal data types (TableInfo, ColumnInfo, etc.)
```

### Pattern 1: information_schema Introspection

**What:** Query PostgreSQL metadata tables to extract schema shape.

**When to use:** Every time the generator runs — this is the primary data source.

**Example queries [VERIFIED: PostgreSQL docs](https://www.postgresql.org/docs/current/information-schema.html):**

```sql
-- List all user tables in public schema
SELECT table_name, 
       obj_description(pgc.oid, 'pg_class') AS table_comment
FROM information_schema.tables t
JOIN pg_class pgc ON pgc.relname = t.table_name
WHERE table_schema = 'public' AND table_type = 'BASE TABLE'
ORDER BY table_name;

-- Columns with metadata
SELECT c.column_name, c.ordinal_position, c.is_nullable, 
       c.data_type, c.character_maximum_length, c.column_default,
       pgd.description AS column_comment
FROM information_schema.columns c
LEFT JOIN pg_catalog.pg_statio_all_tables st 
    ON st.schemaname = c.table_schema AND st.relname = c.table_name
LEFT JOIN pg_catalog.pg_description pgd 
    ON pgd.objoid = st.relid AND pgd.objsubid = c.ordinal_position
WHERE c.table_schema = 'public' AND c.table_name = $1
ORDER BY c.ordinal_position;

-- Foreign keys
SELECT kcu.column_name, 
       ccu.table_schema AS foreign_table_schema,
       ccu.table_name AS foreign_table_name,
       ccu.column_name AS foreign_column_name,
       rc.update_rule, rc.delete_rule
FROM information_schema.table_constraints tc
JOIN information_schema.key_column_usage kcu 
    ON tc.constraint_name = kcu.constraint_name
JOIN information_schema.referential_constraints rc 
    ON tc.constraint_name = rc.constraint_name
JOIN information_schema.constraint_column_usage ccu 
    ON rc.unique_constraint_name = ccu.constraint_name
WHERE tc.constraint_type = 'FOREIGN KEY' 
    AND tc.table_name = $1;

-- Indexes
SELECT indexname, indexdef 
FROM pg_indexes 
WHERE schemaname = 'public' AND tablename = $1
ORDER BY indexname;

-- Enum types
SELECT t.typname, e.enumlabel
FROM pg_type t
JOIN pg_enum e ON t.oid = e.enumtypid
ORDER BY t.typname, e.enumsortorder;
```

### Pattern 2: Rustdoc Annotation Parsing

**What:** Read `///` doc comments from entity struct files. Use regex to match `///` lines immediately preceding `pub struct <Name>`.

**When to use:** For each entity struct found, extract its doc comment. Fall back to generic description if no doc comment exists.

**Table-to-entity naming convention [VERIFIED: codebase analysis]:**
- `Server` struct → `servers` table (PascalCase → snake_case + pluralization)
- `Node` struct → `nodes` table
- `BackupConfig` struct → inline in `servers` columns (composite entity)
- `BackupRecord` struct → `backup_history` table (explicit mapping)
- `NodeApiKey` struct → `node_api_keys` table
- `CronTask` struct → `cron_tasks` table
- Some entities require explicit table name mapping (`backup_history` vs `BackupRecord`)

### Anti-Patterns to Avoid

- **Parsing `sqlx::FromRow` derive macros to determine column mapping:** The struct field names already match column names by convention. Use `information_schema` as source of truth.
- **One massive Mermaid diagram for all tables:** D-03 explicitly says group by FK clusters. Overly large ER diagrams are unreadable on GitHub.
- **Hardcoding domain assignments:** Use a configurable mapping (table_name → domain) so the tool can be updated when new tables are added.
- **Regenerating without diff review:** The generated file is committed. Developers should review changes before committing to catch unintentional schema drift.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| CLI argument parsing | Custom arg parser | `clap` 4 | Derive macros, auto-generated --help, subcommands, shell completions. A custom parser introduces bugs and poor UX. |
| PostgreSQL connectivity | Raw libpq bindings | `sqlx` | `sqlx` is already the project standard; handles connection pooling, type mapping, and async. |
| Rust source file traversal | Manual fs recursion | `walkdir` | Handles symlinks, permission errors, `.gitignore` patterns. Corner cases in manual traversal cause silent failures. |
| Markdown table formatting | Manual column width calculation | String formatting + `unicode-width` | Unicode chars have varying display widths. Without `unicode-width`, CJK characters break table alignment. |
| Regex parsing of doc comments | Custom tokenizer | `regex` | The pattern `/// (.*)` is simple. A custom parser adds complexity for no benefit. |

**Key insight:** The generator's complexity comes from combining two data sources (live DB + source code) and producing well-structured Markdown. Each piece individually is straightforward — the value is in the integration.

## Runtime State Inventory

> This section omitted — Phase 64 is a greenfield documentation phase (new Rust CLI tool, no rename/refactor/migration of existing code).

## Schema Discovery Results

> Discovered by reading all 69 migration files in `api/migrations/` and all entity structs in `api/src/domain/`.

### Business Domain Classification

| Domain | Tables | Entity Structs |
|--------|--------|----------------|
| **Servers** | `servers`, `server_events`, `server_metrics`, `server_crash_logs` | `Server`, `ServerCrashLog`, `ServerMetrics` (response DTO) |
| **Nodes** | `nodes`, `node_metrics`, `node_events`, `server_nodes` | `Node`, `NodeMetrics`, `NodeHealth` |
| **Users/Auth** | `users`, `api_keys`, `roles`, `permissions`, `user_roles` | `User`, `AuditLog` |
| **Billing/Subscriptions** | `plans`, `subscriptions`, `billing_customers`, `payment_transactions`, `invoices`, `refunds` | `Plan`, `Subscription`, `BillingCustomer`, `Invoice`, `UsageRecord` |
| **Backups** | `backup_history`, `s3_profiles` | `BackupRecord`, `BackupConfig`, `S3Profile` |
| **Settings/Config** | `app_settings`, `cron_tasks`, `cloudflare_settings` (in app_settings) | `CronTask`, `CloudflareConfig`, `RestartDefaults` |
| **Events/Logs** | `audit_logs`, `alert_rules`, `alert_states`, `alert_history` | `AlertRule`, `AlertState`, `AlertEvent`, `AuditLog` |
| **Jobs** | `jobs` | `Job` |
| **Templates** | `templates`, `modpack_templates`, `plugin_templates` | `Template`, `ModpackTemplate`, `PluginTemplate` |
| **Games** | `game_types`, `port_pools`, `resource_plans`, `deployment_configs` | (use case objects, no entity struct) |
| **Webhooks** | `webhooks` | `Webhook` |
| **Usage** | `usage_tracking` | `UsageRecord` |
| **Infrastructure** | `agents` (legacy), `node_api_keys`, `node_registration_tokens` | `NodeApiKey`, `NodeRegistrationToken` |

### Foreign Key Relationship Clusters (for Mermaid ER diagram grouping)

**Cluster 1: Users & Auth** (users → api_keys, users → roles/permissions, users → user_roles)
**Cluster 2: Servers & Events** (servers → server_events, server_metrics, server_crash_logs; servers → cron_tasks)
**Cluster 3: Nodes & Infrastructure** (nodes → node_metrics, node_events, node_api_keys, node_registration_tokens; nodes ↔ servers via server_nodes)
**Cluster 4: Billing & Subscriptions** (users → subscriptions → plans; subscriptions → invoices, refunds, payment_transactions)
**Cluster 5: Backups & Storage** (servers → backup_history; s3_profiles standalone)
**Cluster 6: Alerts & Monitoring** (servers → alert_rules → alert_states/alert_history)
**Cluster 7: Jobs** (users/servers/nodes → jobs) — relatively standalone
**Cluster 8: Webhooks & Usage** (users → webhooks; users → usage_tracking)
**Cluster 9: Game Configuration** (game_types → deployment_configs; port_pools, resource_plans)
**Cluster 10: Templates** (templates, modpack_templates, plugin_templates — standalone reference data)

### Current Entity Documentation Coverage [VERIFIED: codebase grep]

Only 5 out of ~25 entity structs have rustdoc annotations:

| File | Has `///` Doc? | Quality |
|------|---------------|---------|
| `entities/s3_profile.rs` | ✅ `/// Named S3-compatible storage profile` | Good |
| `entities/backup_config.rs` | ✅ `/// Logical aggregation of per-server backup configuration` | Good |
| `entities/settings.rs` | ✅ `/// Restart policy default values` + `/// S3-compatible storage configuration` | Good |
| `server/template/model.rs` | ✅ `/// Template entity representing...` | Good |
| `server/modpack_template/model.rs` | ✅ `/// Modpack template entity representing...` | Good |
| `server/plugin_template/model.rs` | ✅ `/// Plugin template entity representing...` | Good |
| `repositories/backup_config_repository.rs` | ✅ `/// Reads from and writes to both servers table + cron_tasks table` | Good |
| `entities/server.rs` | ❌ No doc comments | — |
| `entities/node.rs` | ❌ No doc comments | — |
| `entities/backup.rs` | ❌ No doc comments | — |
| `entities/cron_task.rs` | ❌ No doc comments | — |
| `entities/alert.rs` | ❌ No doc comments | — |
| `entities/alert_state.rs` | ❌ No doc comments | — |
| `entities/node_metrics.rs` | ❌ No doc comments | — |
| `entities/server_metrics.rs` | ❌ No doc comments | — |
| `entities/node_api_key.rs` | ❌ No doc comments | — |
| `entities/node_registration_token.rs` | ❌ No doc comments | — |
| `entities/cloudflare_settings.rs` | ❌ No doc comments | — |
| `entities/server_crash_log.rs` | ❌ No doc comments | — |
| `user/model.rs` | ❌ No doc comments | — |
| `plan/model.rs` | ❌ No doc comments | — |
| `subscription/model.rs` | ❌ No doc comments | — |
| `billing/model.rs` | ❌ No doc comments | — |
| `billing/invoice_model.rs` | ❌ No doc comments | — |
| `job/model.rs` | ❌ No doc comments | — |
| `audit/model.rs` | ❌ No doc comments | — |
| `webhook/model.rs` | ❌ No doc comments | — |
| `rbac/model.rs` | ❌ No doc comments | — |
| `usage/model.rs` | ❌ No doc comments | — |

**Implication:** The generator will read `information_schema` for column metadata (type, nullable, default) but will have sparse rustdoc annotations to draw from. The generator should:
1. Use column comments from `pg_description` in the database (if any)
2. Use rustdoc annotations where available
3. Fall back to auto-generated descriptions from column names (e.g., `user_id` → "References the owning user")
4. The `design rationale` and `query patterns` per domain section should be generated from code comments where available, and a manual authoring step is needed for domains without documentation

### Domain Summary Data for Output

**Servers** (5 tables):
- `servers` — ~50 columns, heavily evolved. Primary game server record.
- `server_events` — Event log per server (event_type, details JSONB).
- `server_metrics` — CPU/RAM/Disk/TPS/Players time-series.
- `server_crash_logs` — Crash forensic records (exit_code, crash_type, recovery_action).
- Foreign keys: servers.user_id → users.id, servers.node_id → nodes.id

**Nodes** (4 tables, plus junction):
- `nodes` — Compute node registration and status.
- `node_metrics` — CPU/Memory/Disk time-series per node.
- `node_events` — Event log per node.
- `node_api_keys` — API key management for node auth.
- `node_registration_tokens` — Auto-registration flow tokens.
- `server_nodes` — Junction: many-to-many servers↔nodes with `is_primary`.
- Foreign keys: all reference nodes.id; node_registration_tokens.node_id nullable

**Users/Auth** (5 tables):
- `users` — Core user: email, password_hash, role, verification/deletion timestamps.
- `api_keys` — Programmatic access: key_hash, prefix, permissions, rate_limit.
- `roles` — Tenant-scoped: user_id, name, description, is_default.
- `permissions` — Resource+action pairs per role.
- `user_roles` — Junction: user_id ↔ role_id.
- Foreign keys: all cascade to users.id

**Billing/Subscriptions** (6 tables):
- `plans` — Offerings with JSONB limits and features.
- `subscriptions` — Active subscriptions per user.
- `billing_customers` — Stripe/LemonSqueezy customer ID mapping.
- `payment_transactions` — Payment records with provider info.
- `invoices` — Billing period invoices.
- `refunds` — Full/prorated refund tracking.
- `usage_tracking` — Metered usage per billing period.

**Templates** (3 tables):
- `templates` — Pre-configured server configs (game_type, category, JSONB config).
- `modpack_templates` — CurseForge/Modrinth modpack references.
- `plugin_templates` — Plugin lists per game type variant.

**Alerts** (3 tables):
- `alert_rules` — Threshold-based rules (metric_type, operator, threshold).
- `alert_states` — Stateful tracking (status, violation_count).
- `alert_history` — Immutable event log.

**Jobs** (1 table):
- `jobs` — Async operation queue with JSONB payload/result, progress tracking.

**Webhooks** (1 table):
- `webhooks` — URL + events + secret + failure tracking.

## Common Pitfalls

### Pitfall 1: Table Name → Entity Name Mismatch
**What goes wrong:** The generator expects `Server` struct → `servers` table, but `BackupRecord` struct maps to `backup_history` table (not `backup_records`).
**Why it happens:** Not all entity struct names naively pluralize to table names. Some tables were named before their entity structs.
**How to avoid:** Include an explicit table name mapping in the generator. Use `information_schema` as authorative source for table names, then match struct names heuristically with fallback.
**Warning signs:** Tables in `information_schema` that don't match any entity struct name.

### Pitfall 2: PostgreSQL Type Mapping for Markdown
**What goes wrong:** Raw `information_schema.data_type` values (e.g., `TIMESTAMP WITH TIME ZONE`, `DOUBLE PRECISION`, `USER-DEFINED` for enums, `JSONB`) displayed verbatim are confusing in developer docs.
**Why it happens:** PostgreSQL exposes detailed type names that include implementation details.
**How to avoid:** Map types to developer-friendly names: `TIMESTAMPTZ` → `timestamptz`, `DOUBLE PRECISION` → `f64`, `USER-DEFINED` → resolve enum labels from `pg_enum`.
**Warning signs:** Markdown table shows `TIMESTAMP WITH TIME ZONE` instead of `timestamptz`.

### Pitfall 3: Missing Rustdoc Annotations
**What goes wrong:** The generator expects descriptive `///` comments but most entity structs have none. Output becomes a sterile column listing with no narrative value.
**Why it happens:** The codebase prioritized implementation speed over documentation comments.
**How to avoid:** The generator should gracefully degrade: column names alone (e.g., `user_id`, `created_at`) already convey intent for many fields. Add a "Documentation Sources" note explaining which entities have explicit annotations and which rely on auto-generated descriptions.
**Warning signs:** A domain section with zero narrative text.

### Pitfall 4: Mermaid ER Diagram Size Limits
**What goes wrong:** A single ER diagram with 10+ tables becomes unreadable — GitHub's Mermaid renderer may time out or produce a tangled mess.
**Why it happens:** Natural clusters like "Servers" involve `servers` → `server_events`, `server_metrics`, `server_crash_logs`, `cron_tasks`, plus `users` and `nodes` FK references.
**How to avoid:** D-03 already mandates per-cluster diagrams. Keep each diagram to 4-8 tables maximum. Break clusters further if needed. Each cluster should tell a cohesive story.
**Warning signs:** A single `erDiagram` block with > 10 entity references.

## Code Examples

### Example 1: Generator CLI Entry Point
```rust
// Source: clap documentation pattern [VERIFIED: clap 4 docs]
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "db-schema-gen", about = "Generate DATABASE_SCHEMA.md from PostgreSQL schema")]
struct Cli {
    /// PostgreSQL connection URL (e.g., postgresql://user:pass@localhost/dbname)
    #[arg(short, long, env = "DATABASE_URL")]
    db_url: String,

    /// Output file path (default: DATABASE_SCHEMA.md in project root)
    #[arg(short, long, default_value = "DATABASE_SCHEMA.md")]
    output: String,

    /// Path to Rust entity source files
    #[arg(short, long, default_value = "api/src/domain")]
    source_dir: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let schema = introspect_database(&cli.db_url).await?;
    let annotations = read_rustdoc_annotations(&cli.source_dir)?;
    let document = generate_markdown(schema, annotations);
    std::fs::write(&cli.output, document)?;
    println!("Generated {}", &cli.output);
    Ok(())
}
```

### Example 2: information_schema Query for Tables
```rust
// Source: PostgreSQL information_schema docs [VERIFIED]
use sqlx::PgPool;

#[derive(Debug, sqlx::FromRow)]
struct TableInfo {
    table_name: String,
    table_comment: Option<String>,
}

async fn get_tables(pool: &PgPool) -> anyhow::Result<Vec<TableInfo>> {
    let tables = sqlx::query_as::<_, TableInfo>(
        r#"
        SELECT t.table_name,
               pgd.description AS table_comment
        FROM information_schema.tables t
        LEFT JOIN pg_catalog.pg_class pgc ON pgc.relname = t.table_name
        LEFT JOIN pg_catalog.pg_description pgd ON pgd.objoid = pgc.oid
            AND pgd.objsubid = 0
        WHERE t.table_schema = 'public'
          AND t.table_type = 'BASE TABLE'
        ORDER BY t.table_name
        "#
    )
    .fetch_all(pool)
    .await?;
    Ok(tables)
}
```

### Example 3: Rustdoc Annotation Parser
```rust
// Source: standard regex pattern for extracting /// doc comments [ASSUMED]
use regex::Regex;

#[derive(Debug)]
struct EntityDoc {
    struct_name: String,
    doc_lines: Vec<String>,
}

fn parse_entity_docs(source_dir: &str) -> anyhow::Result<Vec<EntityDoc>> {
    let re_struct = Regex::new(r"pub struct (\w+)")?;
    let re_doc = Regex::new(r"^\s*///\s?(.*)$")?;
    let mut entities = Vec::new();

    for entry in walkdir::WalkDir::new(source_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".rs"))
    {
        let content = std::fs::read_to_string(entry.path())?;
        let mut current_doc: Vec<String> = Vec::new();
        let mut current_struct: Option<String> = None;

        for line in content.lines() {
            if let Some(cap) = re_doc.captures(line) {
                current_doc.push(cap[1].to_string());
            } else if let Some(cap) = re_struct.captures(line) {
                if !current_doc.is_empty() {
                    entities.push(EntityDoc {
                        struct_name: cap[1].to_string(),
                        doc_lines: current_doc.clone(),
                    });
                }
                current_doc.clear();
            } else {
                current_doc.clear(); // non-doc line resets
            }
        }
    }
    Ok(entities)
}
```

### Example 4: Mermaid ER Diagram Generation
```rust
// Source: Mermaid erDiagram syntax [VERIFIED: mermaid.js.org]
fn generate_er_diagram(tables: &[TableCluster]) -> String {
    let mut output = String::from("```mermaid\nerDiagram\n");

    for table in tables {
        output.push_str(&format!("    {} {{\n", table.name));
        for col in &table.columns {
            let pk_marker = if col.is_pk { " PK " } else { "" };
            let fk_marker = if col.is_fk { " FK " } else { "" };
            output.push_str(&format!(
                "        {}{} {} {}\n",
                col.data_type, pk_marker, col.name, fk_marker
            ));
        }
        output.push_str("    }\n");
    }

    for rel in &table.relationships {
        output.push_str(&format!(
            "    {} {} {} {} {}\n",
            rel.parent_table,
            rel.parent_cardinality,
            rel.child_table,
            rel.child_cardinality,
            rel.description
        ));
    }

    output.push_str("```\n");
    output
}
```

### Example 5: Mermaid ER Diagram Output (Servers Cluster)
```mermaid
erDiagram
    servers {
        uuid id PK
        text name
        text game
        text status
        uuid user_id FK
        uuid node_id FK
        timestamptz created_at
        timestamptz updated_at
    }
    server_events {
        uuid id PK
        uuid server_id FK
        text event_type
        jsonb details
        timestamptz created_at
    }
    server_metrics {
        uuid id PK
        uuid server_id FK
        real cpu_usage
        bigint memory_usage_mb
        bigint disk_usage_mb
        real tps
        integer players
        timestamptz created_at
    }
    server_crash_logs {
        uuid id PK
        uuid server_id FK
        integer exit_code
        text crash_type
        text log_excerpt
        text recovery_action
    }
    cron_tasks {
        uuid id PK
        uuid server_id FK
        uuid user_id FK
        text task_type
        text schedule_cron
        text timezone
    }
    servers ||--o{ server_events : has
    servers ||--o{ server_metrics : has
    servers ||--o{ server_crash_logs : has
    servers ||--o{ cron_tasks : has
    users  ||--o{ servers : owns
    nodes  ||--o{ servers : runs_on
```

### Example 6: Markdown Table Format
```markdown
### `servers`

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | `uuid` | PK, DEFAULT gen_random_uuid() | Unique server identifier |
| `name` | `text` | NOT NULL | Human-readable server name |
| `game` | `text` | NOT NULL, DEFAULT 'minecraft' | Game type identifier (e.g., minecraft, palworld) |
| `status` | `text` | NOT NULL, DEFAULT 'stopped' | Current server status (stopped/running/starting/stopping) |
| `user_id` | `uuid` | FK → users.id | Owner of this server (multi-tenant isolation) |
| `node_id` | `uuid` | FK → nodes.id, NULLABLE | Compute node running this server |
| `created_at` | `timestamptz` | NOT NULL, DEFAULT NOW() | Record creation timestamp |
| `updated_at` | `timestamptz` | NOT NULL, DEFAULT NOW() | Last modification timestamp |
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Multi-file docs under `docs/` | Single file at repo root | Phase 64 D-01 | Simpler, but no docs site integration |
| Manual documentation | Rust CLI generator + committed output | Phase 64 D-08/D-09 | Auto-generated, but requires regeneration on schema changes |
| No ER diagrams | Mermaid `erDiagram` blocks | Phase 64 D-02 | Better visualization, GitHub-native rendering |

**Deprecated/outdated:**
- Separating schema docs from code (migrations history in `api/migrations/` is sufficient for migration timeline, but `DATABASE_SCHEMA.md` provides a current-snapshot reference)

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Table name → entity struct mapping follows PascalCase→snake_case convention with some exceptions | Code Examples | Some tables may have no matching entity struct, requiring manual descriptions |
| A2 | `pg_description` and `pg_class.oid` comments are not populated for tables | Common Pitfalls | If comments exist, generator should honor them as authoritative |
| A3 | The `tools/` directory doesn't exist yet and needs to be created | Architecture | Minor — just `mkdir` step |
| A4 | Rustdoc annotations on entity structs will be sparse | Schema Discovery | Generator must handle missing annotations gracefully |
| A5 | The generator tool does not need to run as part of CI | Environment Availability | If CI regeneration is desired later, a CI step would need adding |
| A6 | `sqlx` 0.7 is compatible with the PostgreSQL version | Standard Stack | PostgreSQL 16 (from STACK.md) works with sqlx 0.7; confirmed by production usage |

## Open Questions (RESOLVED)

1. **Where to place the generator Cargo.toml — workspace member or standalone?**
   - What we know: `api/Cargo.toml` exists but there's no root `Cargo.toml` workspace. `agent-core/Cargo.toml` is a separate workspace.
   - What's unclear: Whether to create a root workspace Cargo.toml that includes `tools/db-schema-gen/` as a member (which would affect all other Rust projects), or keep it standalone (simple but no workspace reuse).
   - Recommendation: Standalone Cargo.toml in `tools/db-schema-gen/`. Avoid restructuring the entire Rust monorepo workspace. The generator has minimal dependency overlap with the API.

2. **How to handle the 25 entity structs without rustdoc annotations?**
   - What we know: Only 6 structs have `///` doc comments; the remaining ~19 don't.
   - What's unclear: Should we add rustdoc annotations to all entity structs as part of this phase, or should the generator gracefully degrade with auto-generated descriptions?
   - Recommendation: **Add minimal rustdoc to all entity structs** as part of this phase. The generator is useless without source annotations. Add `///` doc comments (1-3 lines per struct + field-level comments for non-obvious columns). This is a one-time cost that makes the generator valuable forever.

3. **What table-to-entity name mapping strategy to use for the generator?**
   - What we know: Some entities use exact mapping (`Node` ↔ `nodes`), some don't (`BackupRecord` ↔ `backup_history`).
   - What's unclear: How to handle these mismatches programmatically.
   - Recommendation: Use a lightweight mapping table in the generator code (`HashMap<&str, &str>`) for entities with non-standard table names. Also scan `#[sqlx(table_name = "...")]` attributes if present.

4. **How to handle `app_settings` key-value table?**
   - What we know: `app_settings` has a `key` TEXT PRIMARY KEY and `value` JSONB — it stores Cloudflare config, S3 config, restart defaults as JSON blobs.
   - What's unclear: Whether to document individual key-value schemas or treat as a generic key-value store.
   - Recommendation: Document the JSONB structures for well-known keys (s3_config, cloudflare_config, restart_defaults) as sub-sections within Settings/Config domain.

5. **Should the generated DATABASE_SCHEMA.md include a "Migration History" cross-reference?**
   - What we know: D-07 explicitly says "Current schema snapshot only."
   - What's unclear: A brief note at the bottom saying "See api/migrations/ for schema evolution" — is this acceptable?
   - Recommendation: Yes — a single sentence cross-reference to `api/migrations/` at the end of the document is useful and doesn't violate D-07 (it's not inline history, it's a pointer).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust/Cargo | Generator tool compilation | ✓ | 1.70+ | — |
| PostgreSQL | Schema introspection | ✓ (via docker-compose) | 16 | — |
| `DATABASE_URL` env var | DB connection | ✓ (set up in Phase 61 dev docs) | — | — |
| GitHub | Mermaid rendering | ✓ | — | Use mermaid-cli for local preview |

**Missing dependencies with no fallback:** None — all standard Rust toolchain tools available on dev machines.

**Missing dependencies with fallback:** Mermaid rendering — GitHub renders natively; local preview requires `mermaid-cli` or browser extension (optional).

## Validation Architecture

> Skipped — this phase produces a documentation file and a developer tool. No unit tests needed for the tool beyond `cargo test`. The quality check is: (1) `cargo build` passes, (2) generated `DATABASE_SCHEMA.md` renders correctly on GitHub.

## Security Domain

> Skipped — `security_enforcement` is not applicable to this phase. The generator is a developer tool that reads from a local database; it does not handle user data, authentication, or network requests beyond connecting to a local PostgreSQL instance.

## Sources

### Primary (HIGH confidence)
- [Project codebase] — 69 migration files in `api/migrations/`, ~25 entity structs in `api/src/domain/`, existing repo-root docs (`DEVELOPMENT.md`, `CONTRIBUTING.md`)
- [STACK.md] — PostgreSQL 16, sqlx 0.7, Rust edition 2021
- [STRUCTURE.md] — Directory layout, `api/src/domain/entities/` locations
- [CONVENTIONS.md] — Rust naming: PascalCase for structs, snake_case for files

### Secondary (MEDIUM confidence)
- [PostgreSQL docs — information_schema](https://www.postgresql.org/docs/current/information-schema.html) — Verified column query patterns
- [Mermaid docs — entityRelationshipDiagram](https://mermaid.js.org/syntax/entityRelationshipDiagram.html) — Verified `erDiagram` syntax

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — All dependencies already used in project or are de-facto Rust standards
- Architecture: HIGH — Generator pattern is clear (clap CLI → sqlx introspection + regex parsing → markdown output)
- Pitfalls: MEDIUM — Rustdoc coverage gap is known but coverage level is estimated from codebase grep

**Research date:** 2026-05-31
**Valid until:** 2026-07-31 (stable toolchain versions, schema may evolve incrementally)
