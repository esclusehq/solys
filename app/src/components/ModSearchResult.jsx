export default function ModSearchResult({ mod, onAdd, onViewVersions }) {
  return (
    <div className="glass-panel p-4 rounded-xl border border-[var(--color-cosmic-border)] flex gap-4">
      {mod.icon_url && (
        <img src={mod.icon_url} alt={mod.title} className="w-12 h-12 rounded-lg object-cover" />
      )}
      <div className="flex-1 min-w-0">
        <h4 className="text-sm font-bold text-white truncate">{mod.title}</h4>
        <p className="text-xs text-[var(--color-text-muted)] line-clamp-2">{mod.description}</p>
        <div className="flex items-center gap-3 mt-2">
          <span className="text-[10px] text-[var(--color-text-muted)]">
            ⬇ {mod.downloads?.toLocaleString() || 0} downloads
          </span>
        </div>
      </div>
      <div className="flex flex-col gap-2">
        <button onClick={() => onViewVersions?.(mod)}
                className="px-3 py-1 text-xs rounded bg-[var(--color-cosmic-cyan)]/10
                           text-[var(--color-cosmic-cyan)] hover:bg-[var(--color-cosmic-cyan)]/20">
          Versions
        </button>
        <button onClick={() => onAdd?.(mod)}
                className="px-3 py-1 text-xs rounded bg-emerald-500/10 text-emerald-400
                           hover:bg-emerald-500/20">
          Add
        </button>
      </div>
    </div>
  )
}
