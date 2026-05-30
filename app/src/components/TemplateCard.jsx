import { Link } from 'react-router-dom'

export default function TemplateCard({ template, onDelete, onClone }) {
  return (
    <div className="glass-panel p-4 rounded-xl border border-[var(--color-cosmic-border)]
                    hover:border-[var(--color-cosmic-cyan)]/50 transition-all">
      <div className="flex items-start justify-between mb-3">
        <div>
          <h3 className="text-white font-bold">{template.display_name}</h3>
          <p className="text-xs text-[var(--color-text-muted)]">
            {template.game_type} / {template.category}
          </p>
        </div>
        {template.is_builtin && (
          <span className="px-2 py-0.5 text-[10px] rounded-full bg-blue-500/20 text-blue-400">
            Official
          </span>
        )}
      </div>

      {template.description && (
        <p className="text-xs text-[var(--color-text-muted)] mb-3 line-clamp-2">
          {template.description}
        </p>
      )}

      <div className="flex items-center justify-between mt-4">
        <div className="flex gap-2">
          <Link to={`/templates/${template.id}`}
                className="px-3 py-1.5 text-xs rounded-lg bg-[var(--color-cosmic-cyan)]/10
                           text-[var(--color-cosmic-cyan)] hover:bg-[var(--color-cosmic-cyan)]/20">
            Create Server
          </Link>
        </div>
        {!template.is_builtin && (
          <button onClick={() => onDelete?.(template.id)}
                  className="text-xs text-red-400 hover:text-red-300">
            Delete
          </button>
        )}
      </div>
    </div>
  )
}
