import { api } from '../lib/api'

export const templatesApi = {
  list: (params) => api.get('/templates', { params }),
  get: (id) => api.get(`/templates/${id}`),
  create: (data) => api.post('/templates', data),
  update: (id, data) => api.put(`/templates/${id}`, data),
  delete: (id) => api.delete(`/templates/${id}`),
  createServer: (id, data) => api.post(`/templates/${id}/create-server`, data),
}

export const modsApi = {
  search: (params) => api.get('/plugins/search', { params }),
  getVersions: (projectId, params) => api.get(`/plugins/${projectId}/versions`, { params }),
}
