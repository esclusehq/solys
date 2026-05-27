# Phase 32: Server Templates for Hobby and Pro plans - Context

**Gathered:** 2026-05-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Pre-configured server templates untuk berbagai game types dan variants yang bisa dipilih user saat membuat server.

</domain>

<decisions>
## Implementation Decisions

### Template Type
- **D-01:** Game type + variant templates (Vanilla, Forge, Fabric, Paper, dll)
- Tiap game type punya beberapa variant options

### Games Included
- **D-02:** Popular games: Minecraft, Palworld, Rust, Valheim, dll
- Multiple variants per game type

### UI Location
- **D-03:** Dropdown di create server form
- User pilih game type → вариант dropdown muncul
- Pre-selected defaults berdasarkan game type

### Integration with Plans
- **D-04:** Templates bekerja dengan plan limits (Phase 15)
- Tidak ada perubahan ke plan limits

</decisions>

<canonical_refs>
## References

- Phase 15 CONTEXT.md — Plan limits (Starter=5, Pro=15, Enterprise=unlimited)
- `app/src/pages/servers/CreateServerPage.jsx` — Create server form
- `api/src/domain/server/template/` — (belum ada, akan dibuat)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Create server form sudah ada
- useServers hook untuk server operations

### Integration Points
- CreateServerPage.jsx — tempat dropdown template
- API endpoint untuk templates (belum ada)

</code_context>

<specifics>
## Specific Details

1. Create server flow:
   - User klik "Add Server"
   - Pilih Game Type (dropdown)
   - Pilih Variant (Vanilla/Forge/Fabric/Paper)
   - Isi server name
   - Server dibuat dengan template properties

2. Template includes:
   - Default container image
   - Default port
   - Default environment variables
   - Default server.properties
   - Default start command

</specifics>

<deferred>
## Deferred Ideas

- Custom user templates (future)
- Community-shared templates

</deferred>

---

## ▶ Next Up

**Phase 32: Server Templates** — Pre-configured templates per game type + variant

`/clear` then:

`/gsd-plan-phase 32 v1.0`