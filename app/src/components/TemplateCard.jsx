import { Link } from 'react-router-dom'

export default function TemplateCard({ template, user, onDelete, onClone }) {
  const isAdmin = user?.role === 'admin' || user?.role === 'owner' || user?.role === 'founder'
  const canEdit = !template.is_builtin || isAdmin

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
        <div className="flex gap-1.5">
          {template.is_builtin && (
            <span className="px-2 py-0.5 text-[10px] rounded-full bg-blue-500/20 text-blue-400">
              Official
            </span>
          )}
          {template.is_active === false && (
            <span className="px-2 py-0.5 text-[10px] rounded-full bg-yellow-500/20 text-yellow-400">
              Coming Soon
            </span>
          )}
        </div>
      </div>

      {template.description && (
        <p className="text-xs text-[var(--color-text-muted)] mb-3 line-clamp-2">
          {template.description}
        </p>
      )}

      <div className="flex items-center justify-between mt-4">
        <div className="flex gap-2">
          {template.is_active === false ? (
            <span className="px-3 py-1.5 text-xs rounded-lg bg-gray-600/20 text-gray-500 cursor-not-allowed">
              Coming Soon
            </span>
          ) : (
            <Link to={`/templates/${template.id}`}
                  className="px-3 py-1.5 text-xs rounded-lg bg-[var(--color-cosmic-cyan)]/10
                             text-[var(--color-cosmic-cyan)] hover:bg-[var(--color-cosmic-cyan)]/20">
              Create Server
            </Link>
          )}
        </div>
        {canEdit && (
          <div className="flex gap-2">
            <Link to={`/templates/${template.id}/edit`}
                  className="text-xs text-yellow-400 hover:text-yellow-300">
              Edit
            </Link>
            <button onClick={() => onDelete?.(template.id)}
                    className="text-xs text-red-400 hover:text-red-300">
              Delete
            </button>
          </div>
        )}
      </div>
    </div>
  )
}
