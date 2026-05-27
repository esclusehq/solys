# Phase 36: Menambahkan fungsi untuk server untuk Bedrock/Pocket - Context

**Gathered:** 2026-05-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Menambahkan Bedrock/Pocket Edition server support sebagai game type baru. Saat ini sudah support Java Edition, Bedrock punya requirements berbeda.

</domain>

<decisions>
## Implementation Decisions

### Game Type
- **D-01:** Game type baru "bedrock" (Minecraft: Bedrock Edition)
- Bukan proxy atau combined container
- Separate Bedrock-specific image dan config

### Bedrock Specifics
- **D-02:** Default port: 19132 (bukan 25565 seperti Java)
- **D-03:** Bedrock container image: itzg/minecraft-bedrock-server atau similar
- **D-04:** Different start command (bedrock-server)

### Reuse Existing
- **D-05:** Gunakan pattern dari Phase 32 (template system)
- game_type field sudah ada di server config
- Template system dari Phase 32 bisa reuse

### UI
- **D-06:** Bedrock muncul di game type dropdown di create server form
- User pilih "Bedrock" sebagai game type

</decisions>

<canonical_refs>
## References

- Phase 32: Template system pattern (api/src/domain/server/template/)
- app/src/features/server/CreateServerModal.jsx — game type dropdown
- api/src/domain/server/model.rs — game_type field exists
- Existing template game_types: minecraft, palworld, rust, valheim

</canonical_refs>

<specifics>
## Specific Details

1. Bedrock container config:
   - docker_image: "itzg/minecraft-bedrock-server"
   - default_port: 19132
   - Bedrock-specific environment variables
   - Different startup command

2. Flow:
   - User pilih "Bedrock" dari game type dropdown
   - Template defaults: bedrock, default variant
   - Server dibuat dengan Bedrock config

3. Tidak include:
   - Geyser/Floodgate (cross-play) — defer ke future phase

</specifics>

<deferred>
## Deferred Ideas

- GeyserMC proxy for Java-Bedrock cross-play
- Bedrock-to-Java conversion
- Pocketmine-MP alternative

</deferred>

---

## ▶ Next Up

**Phase 36: Bedrock Server Support** — Add bedrock as game type

`/clear` then:

`/gsd-plan-phase 36 v1.0`