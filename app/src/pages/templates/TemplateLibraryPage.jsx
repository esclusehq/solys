import { useState, useEffect } from 'react'
import { Link } from 'react-router-dom'
import { templatesApi } from '../../api/templatesApi'
import { useAuthStore } from '../../store/authStore'
import TemplateCard from '../../components/TemplateCard'
import { EscluseSpinner } from '../../components/SkeletonLoader'

export default function TemplateLibraryPage() {
  const { user } = useAuthStore()
  const [templates, setTemplates] = useState([])
  const [loading, setLoading] = useState(true)
  const [gameFilter, setGameFilter] = useState('all')
  const [searchQuery, setSearchQuery] = useState('')

  useEffect(() => {
    loadTemplates()
  }, [gameFilter])

  const loadTemplates = async () => {
    try {
      setLoading(true)
      const params = gameFilter !== 'all' ? { game_type: gameFilter } : {}
      const data = await templatesApi.list(params)
      setTemplates(Array.isArray(data) ? data : [])
    } catch (err) {
      console.error('Failed to load templates:', err)
    } finally {
      setLoading(false)
    }
  }

  const handleDelete = async (id) => {
    if (!confirm('Delete this template?')) return
    try {
      await templatesApi.delete(id)
      setTemplates(prev => prev.filter(t => t.id !== id))
    } catch (err) {
      console.error('Failed to delete template:', err)
    }
  }

  // Filter by search
  const filtered = templates.filter(t =>
    !searchQuery || t.display_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    t.game_type?.toLowerCase().includes(searchQuery.toLowerCase()) ||
    t.category?.toLowerCase().includes(searchQuery.toLowerCase())
  )

  // Separate built-in (featured) from user templates
  const featured = filtered.filter(t => t.is_builtin)
  const userTemplates = filtered.filter(t => !t.is_builtin)

  // Unique game types for filter dropdown
  const gameTypes = [...new Set(templates.map(t => t.game_type).filter(Boolean))]

  if (loading) return <div className="p-6"><EscluseSpinner /></div>

  return (
    <div className="p-6">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold text-white">Template Library</h2>
        <Link to="/templates/create"
              className="px-4 py-2 text-sm rounded-lg bg-[var(--color-cosmic-cyan)]/10
                         text-[var(--color-cosmic-cyan)] hover:bg-[var(--color-cosmic-cyan)]/20
                         border border-[var(--color-cosmic-cyan)]/30 font-bold">
          + Create Template
        </Link>
      </div>

      {/* Filters */}
      <div className="flex gap-3 mb-6">
        <input type="text" value={searchQuery} onChange={e => setSearchQuery(e.target.value)}
               placeholder="Search templates..."
               className="flex-1 px-4 py-2.5 rounded-lg text-sm bg-[var(--color-cosmic-card)]/60
                          border border-[var(--color-cosmic-border)] text-white" />
        <select value={gameFilter} onChange={e => setGameFilter(e.target.value)}
                className="px-4 py-2.5 rounded-lg text-sm bg-[var(--color-cosmic-card)]/60
                           border border-[var(--color-cosmic-border)] text-white">
          <option value="all">All Games</option>
          {gameTypes.map(gt => (
            <option key={gt} value={gt}>{gt.charAt(0).toUpperCase() + gt.slice(1)}</option>
          ))}
        </select>
      </div>

      {/* Featured (built-in) templates */}
      {featured.length > 0 && (
        <section className="mb-8">
          <h3 className="text-lg font-bold text-white mb-3">Featured Templates</h3>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {featured.map(t => (
              <TemplateCard key={t.id} template={t} user={user} onDelete={handleDelete} />
            ))}
          </div>
        </section>
      )}

      {/* User-created templates */}
      {userTemplates.length > 0 && (
        <section>
          <h3 className="text-lg font-bold text-white mb-3">Your Templates</h3>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {userTemplates.map(t => (
              <TemplateCard key={t.id} template={t} user={user} onDelete={handleDelete} />
            ))}
          </div>
        </section>
      )}

      {/* Empty state */}
      {filtered.length === 0 && !loading && (
        <div className="text-center py-12">
          <p className="text-[var(--color-text-muted)]">No templates found.</p>
        </div>
      )}
    </div>
  )
}
