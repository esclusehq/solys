# Phase 34: Modpacks Templates for Hobby and Pro plans - Context

**Gathered:** 2026-05-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Modpack download integration menggunakan CurseForge/Modrinth API untuk download modpacks saat server creation.

</domain>

<decisions>
## Implementation Decisions

### Modpack Source
- **D-01:** CurseForge/Modrinth API integration
- Download modpacks saat server creation berdasarkan pilihan user
- Phase 28 sudah punya PluginManager - bisa reusable pattern

### Games
- **D-02:** Minecraft modpacks (modifiers seperti server-side mods)
- Tidak semua game support mods

### UI Integration
- **D-03:** Di server creation form, setelah game type dipilih
- Tampilkan modpack selector jika game type support mods
- Reuse Pattern dari Phase 28 dan 32

### Fallback
- **D-04:** Curated list jika API tidak available
- Cache modpack metadata locally

</decisions>

<canonical_refs>
## References

- Phase 28: PluginManager component pattern (app/src/components/PluginManager.jsx)
- Phase 32: Template system (api/src/domain/server/template/)
- `app/src/components/PluginManager.jsx` — PluginManager component
- API: CurseForge/Modrinth documentation

</canonical_refs>

<specifics>
## Specific Details

1. Flow:
   - User pilih game type (Minecraft)
   - Jika support mods, muncul modpack selector
   - User pilih modpack dari list atau search
   - System download mods saat server provision

2. API integration:
   - Search modpacks by game
   - Get modpack versions/files
   - Download URL generation

3. Reuse from Phase 28:
   - PluginManager UI pattern
   - Download progress display

</specifics>

<deferred>
## Deferred Ideas

- Custom modpack upload
- Modpack versioning
- Modpack config presets

</deferred>

---

## ▶ Next Up

**Phase 34: Modpacks Templates** — CurseForge/Modrinth integration

`/clear` then:

`/gsd-plan-phase 34 v1.0`