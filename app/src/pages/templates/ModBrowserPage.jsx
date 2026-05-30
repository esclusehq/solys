import { useState, useRef, useEffect } from 'react'
import { modsApi } from '../../api/templatesApi'
import ModSearchResult from '../../components/ModSearchResult'
import { EscluseSpinner } from '../../components/SkeletonLoader'

export default function ModBrowserPage() {
  const [results, setResults] = useState([])
  const [total, setTotal] = useState(0)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState(null)
  const [query, setQuery] = useState('')
  const [version, setVersion] = useState('')
  const [loader, setLoader] = useState('')
  const [offset, setOffset] = useState(0)
  const pageSize = 25
  const debounceRef = useRef(null)

  const executeSearch = async (q, v, l, off) => {
    if (!q || q.trim().length < 2) {
      setResults([])
      setTotal(0)
      return
    }
    try {
      setLoading(true)
      setError(null)
      const params = { q }
      if (v) params.version = v
      if (l) params.loader = l
      params.offset = off.toString()
      params.sort = 'downloads'
      const data = await modsApi.search(params)
      setResults(data.plugins || data.hits || [])
      setTotal(data.total || 0)
    } catch (e) {
      setError(e.message)
      setResults([])
    } finally {
      setLoading(false)
    }
  }

  const handleSearchChange = (e) => {
    const val = e.target.value
    setQuery(val)
    setOffset(0)
    clearTimeout(debounceRef.current)
    debounceRef.current = setTimeout(() => {
      executeSearch(val, version, loader, 0)
    }, 300)
  }

  useEffect(() => {
    return () => clearTimeout(debounceRef.current)
  }, [])

  const handleFilterChange = () => {
    clearTimeout(debounceRef.current)
    setOffset(0)
    debounceRef.current = setTimeout(() => {
      executeSearch(query, version, loader, 0)
    }, 300)
  }

  const handleVersionChange = (e) => {
    setVersion(e.target.value)
    handleFilterChange()
  }

  const handleLoaderChange = (e) => {
    setLoader(e.target.value)
    handleFilterChange()
  }

  const loadMore = () => {
    const newOffset = offset + pageSize
    setOffset(newOffset)
    executeSearch(query, version, loader, newOffset)
  }

  const handleViewVersions = async (mod) => {
    try {
      const versions = await modsApi.getVersions(mod.project_id)
      console.log('Versions for', mod.title, ':', versions)
      alert(`Versions for ${mod.title}:\n${(versions || []).slice(0, 5).map(v => `${v.name} (${v.version_number})`).join('\n') || 'No versions found'}`)
    } catch (e) {
      console.error('Failed to load versions:', e)
    }
  }

  const handleAdd = (mod) => {
    alert(`Add "${mod.title}" to collection — Coming soon!`)
  }

  return (
    <div className="p-6">
      <h2 className="text-2xl font-bold text-white mb-6">Mod Browser</h2>
      <p className="text-xs text-[var(--color-text-muted)] mb-5">
        Browse and search mods from Modrinth. Click "Versions" to see available versions, or "Add" to include in a collection.
      </p>

      {/* Search + Filters */}
      <div className="flex gap-3 mb-6">
        <input type="text" value={query} onChange={handleSearchChange}
               placeholder="Search mods (min 2 characters)..."
               className="flex-1 px-4 py-2.5 rounded-lg text-sm bg-[var(--color-cosmic-card)]/60
                          border border-[var(--color-cosmic-border)] text-white" />
        <select value={version} onChange={handleVersionChange}
                className="px-4 py-2.5 rounded-lg text-sm bg-[var(--color-cosmic-card)]/60
                           border border-[var(--color-cosmic-border)] text-white">
          <option value="">All Versions</option>
          <option value="1.21">1.21</option>
          <option value="1.20.4">1.20.4</option>
          <option value="1.20.1">1.20.1</option>
          <option value="1.19.4">1.19.4</option>
          <option value="1.18.2">1.18.2</option>
          <option value="1.16.5">1.16.5</option>
        </select>
        <select value={loader} onChange={handleLoaderChange}
                className="px-4 py-2.5 rounded-lg text-sm bg-[var(--color-cosmic-card)]/60
                           border border-[var(--color-cosmic-border)] text-white">
          <option value="">All Loaders</option>
          <option value="forge">Forge</option>
          <option value="fabric">Fabric</option>
          <option value="quilt">Quilt</option>
          <option value="neoforge">NeoForge</option>
          <option value="paper">Paper</option>
          <option value="spigot">Spigot</option>
          <option value="purpur">Purpur</option>
        </select>
      </div>

      {/* Error */}
      {error && (
        <div className="mb-4 px-4 py-3 rounded-lg text-sm font-medium bg-red-500/10 text-red-400 border border-red-500/20">
          {error}
        </div>
      )}

      {/* Loading */}
      {loading && <EscluseSpinner />}

      {/* Results */}
      {!loading && results.length > 0 && (
        <>
          <p className="text-xs text-[var(--color-text-muted)] mb-4">{total} results found</p>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {results.map((mod, i) => (
              <ModSearchResult key={mod.project_id || i} mod={mod}
                               onViewVersions={handleViewVersions}
                               onAdd={handleAdd} />
            ))}
          </div>
          {results.length < total && (
            <button onClick={loadMore}
                    className="mt-6 w-full py-2.5 rounded-lg text-sm font-bold
                               bg-[var(--color-cosmic-cyan)]/10 text-[var(--color-cosmic-cyan)]
                               hover:bg-[var(--color-cosmic-cyan)]/20 border border-[var(--color-cosmic-cyan)]/30">
              Load More ({results.length} / {total})
            </button>
          )}
        </>
      )}

      {/* Empty state */}
      {!loading && !error && results.length === 0 && query.length >= 2 && (
        <div className="text-center py-12">
          <p className="text-[var(--color-text-muted)]">No mods found matching your search.</p>
        </div>
      )}

      {/* Initial state */}
      {!loading && !error && query.length < 2 && (
        <div className="text-center py-12">
          <p className="text-[var(--color-text-muted)]">Enter at least 2 characters to search mods.</p>
        </div>
      )}
    </div>
  )
}
