---
phase: 28
plan: 01
type: execute
wave: 1
status: complete
completed: 2026-04-20
---

# Phase 28 Plan 01: Download Plugin/Mod/Datapack - Summary

## Completed Tasks

| Task | Status | Notes |
|------|--------|-------|
| Task 1: Add Plugins/Datapacks tab to ServerDetailsPage.jsx | ✅ Complete | Added conditional tab based on mc_loader |

## Changes Made

### ServerDetailsPage.jsx

1. **Added PluginManager import:**
   ```javascript
   import PluginManager from '../../components/PluginManager'
   ```

2. **Added conditional plugins tab:**
   ```javascript
   const moddedLoaders = ['PAPER', 'SPIGOT', 'BUKKIT', 'PURPUR', 'FORGE', 'FABRIC', 'NEOFORGE']
   const isModded = server?.config?.mc_loader && moddedLoaders.includes(server.config.mc_loader?.toUpperCase())
   const pluginsTab = isModded
     ? { id: 'plugins', label: 'Plugins', icon: '🧩' }
     : { id: 'plugins', label: 'Datapacks', icon: '📦' }

   const tabs = [
     { id: 'overview', label: 'Overview', icon: '📊' },
     { id: 'files', label: 'Files', icon: '📁' },
     pluginsTab,
     { id: 'logs', label: 'Logs', icon: '📋' },
   ]
   ```

3. **Added conditional rendering for plugins tab:**
   ```javascript
   {activeTab === 'plugins' && server && (
     <div className="flex-1 min-h-0">
       <PluginManager serverId={id} server={server} />
     </div>
   )}
   ```

## Key Files Modified

| File | Lines | Purpose |
|------|-------|---------|
| app/src/pages/servers/ServerDetailsPage.jsx | 339 (+8) | Added plugins tab |

## Verification Results

- [x] Build passes (npm run build)
- [x] PluginManager imported
- [x] Conditional tab based on mc_loader
- [x] Tab shows "Plugins" for modded loaders (Paper, Spigot, Forge, Fabric, etc.)
- [x] Tab shows "Datapacks" for vanilla servers

## Phase Coverage

| Decision | Covered By |
|----------|------------|
| D-01: Add Plugins tab to ServerDetailsPage | Task 1 |
| D-02: Use mc_loader field for detection | Task 1 |
| D-03: Minecraft only | Task 1 |

---

*Phase: 28*
*Plan: 28-01*
*Completed: 2026-04-20*