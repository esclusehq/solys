import { useState, useEffect, useCallback } from 'react'
import { templatesApi } from '../api/templatesApi'

export function useTemplateLibrary() {
  const [templates, setTemplates] = useState([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState(null)

  const refetch = useCallback(async (params = {}) => {
    try {
      setLoading(true)
      const data = await templatesApi.list(params)
      setTemplates(Array.isArray(data) ? data : [])
      setError(null)
    } catch (err) {
      setError(err.message)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => { refetch() }, [refetch])

  return { templates, loading, error, refetch }
}

// Separate async helpers for CRUD operations (not tied to hook lifecycle)
export async function createTemplate(data) {
  return templatesApi.create(data)
}

export async function updateTemplate(id, data) {
  return templatesApi.update(id, data)
}

export async function deleteTemplate(id) {
  return templatesApi.delete(id)
}

export async function createServerFromTemplate(templateId, data) {
  return templatesApi.createServer(templateId, data)
}
