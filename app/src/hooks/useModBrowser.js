import { useState, useCallback } from 'react'
import { modsApi } from '../api/templatesApi'

export function useModBrowser() {
  const [results, setResults] = useState([])
  const [total, setTotal] = useState(0)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState(null)

  const search = useCallback(async (query, version, loader, projectType, offset = 0) => {
    if (!query || query.trim().length < 2) {
      setResults([])
      setTotal(0)
      return
    }
    try {
      setLoading(true)
      setError(null)
      const params = { q: query }
      if (version) params.version = version
      if (loader) params.loader = loader
      if (projectType) params.project_type = projectType
      params.offset = offset.toString()
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
  }, [])

  return { results, total, loading, error, search }
}
