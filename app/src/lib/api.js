import { useAuthStore } from '../store/authStore'

const API_URL = '/api/v1'

class ApiClient {
  constructor() {
    this.baseUrl = API_URL
  }

  getToken() {
    return useAuthStore.getState().accessToken
  }

  async request(endpoint, options = {}) {
    const token = this.getToken()
    
    const headers = {
      'Content-Type': 'application/json',
      ...options.headers,
    }

    if (token) {
      headers['Authorization'] = `Bearer ${token}`
    }

    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      ...options,
      headers,
      credentials: 'include',  // Include cookies for authentication
    })

    if (response.status === 401) {
      const refreshed = await useAuthStore.getState().refreshAccessToken()
      if (refreshed) {
        return this.request(endpoint, options)
      }
      useAuthStore.getState().logout()
      throw new Error('Session expired')
    }

    const data = await response.json().catch(() => null)
    
    if (!response.ok) {
      throw new Error(data?.error?.message || data?.message || `Request failed: ${response.status}`)
    }

    return data?.data ?? data
  }

  get(endpoint, options = {}) {
    return this.request(endpoint, { ...options, method: 'GET' })
  }

  post(endpoint, body, options = {}) {
    return this.request(endpoint, { 
      ...options, 
      method: 'POST', 
      body: JSON.stringify(body) 
    })
  }

  put(endpoint, body, options = {}) {
    return this.request(endpoint, { 
      ...options, 
      method: 'PUT', 
      body: JSON.stringify(body) 
    })
  }

  patch(endpoint, body, options = {}) {
    return this.request(endpoint, { 
      ...options, 
      method: 'PATCH', 
      body: JSON.stringify(body) 
    })
  }

  delete(endpoint, options = {}) {
    return this.request(endpoint, { ...options, method: 'DELETE' })
  }

  templates(params = {}) {
    return this.get('/templates', { params })
  }
}

export const api = new ApiClient()

export const serversApi = {
  list: () => api.get('/servers'),
  get: (id) => api.get(`/servers/${id}`),
  create: (data) => api.post('/servers', data),
  update: (id, data) => api.put(`/servers/${id}`, data),
  delete: (id) => api.delete(`/servers/${id}`),
  start: (id) => api.post(`/servers/${id}/start`),
  stop: (id) => api.post(`/servers/${id}/stop`),
  restart: (id) => api.post(`/servers/${id}/restart`),
  getLogs: (id, params) => api.get(`/servers/${id}/logs`, { params }),
  streamLogs: (id, params) => api.get(`/servers/${id}/logs/stream`, { params }),
  getStats: (id) => api.get(`/servers/${id}/stats`),
  getServerProperties: (id) => api.get(`/servers/${id}/properties`),
  updateServerProperties: (id, properties) => api.patch(`/servers/${id}/properties`, properties),
}

export const nodesApi = {
  list: () => api.get('/nodes'),
  get: (id) => api.get(`/nodes/${id}`),
  getStats: () => api.get('/nodes/stats'),
}

export const billingApi = {
  getPlans: () => api.get('/billing/plans'),
  getPlan: (id) => api.get(`/billing/plans/${id}`),
  getInvoices: () => api.get('/billing/invoices'),
  getCurrentSubscription: () => api.get('/billing/subscription'),
  createCheckout: (planId, billingCycle) => api.post('/billing/checkout', { 
    plan_id: planId,
    billing_cycle: billingCycle 
  }),
  getRefundEligibility: (subscriptionId) => api.get(`/billing/refund/eligibility?subscription_id=${subscriptionId}`),
  requestRefund: (subscriptionId, reason) => api.post('/billing/refund', { subscription_id: subscriptionId, reason }),
  getRefunds: () => api.get('/billing/refunds'),
}

export const usersApi = {
  getProfile: () => api.get('/users/me'),
  updateProfile: (data) => api.put('/users/me', data),
  updatePassword: (data) => api.put('/users/me/password', data),
  getApiKey: () => api.get('/users/me/api-key'),
  regenerateApiKey: () => api.post('/users/me/api-key/regenerate'),
}

export const notificationsApi = {
  list: () => api.get('/notifications'),
  markRead: (id) => api.put(`/notifications/${id}/read`),
  markAllRead: () => api.put('/notifications/read-all'),
}

export const jobsApi = {
  list: (params) => api.get('/jobs', { params }),
  get: (id) => api.get(`/jobs/${id}`),
  cancel: (id) => api.delete(`/jobs/${id}`),
  retry: (id) => api.post(`/jobs/${id}/retry`),
  getByServer: (serverId) => api.get(`/jobs/by-server/${serverId}`),
}

export const webhooksApi = {
  list: () => api.get('/webhooks'),
  get: (id) => api.get(`/webhooks/${id}`),
  create: (data) => api.post('/webhooks', data),
  update: (id, data) => api.put(`/webhooks/${id}`, data),
  delete: (id) => api.delete(`/webhooks/${id}`),
  test: (id) => api.post(`/webhooks/${id}/test`),
  retry: (id) => api.post(`/webhooks/${id}/retry`),
}

export const tasksApi = {
  get: (externalId) => api.get(`/jobs/tasks/${externalId}`),
}

export const templatesApi = {
  list: (params) => api.get('/templates', { params }),
}

export const cloudflareApi = {
  getConfig: () => api.get('/settings/cloudflare'),
  saveConfig: (data) => api.put('/settings/cloudflare', data),
  testConnection: () => api.post('/settings/cloudflare/test'),
}

export const s3ProfilesApi = {
  list: () => api.get('/settings/s3/profiles'),
  get: (id) => api.get(`/settings/s3/profiles/${id}`),
  create: (data) => api.post('/settings/s3/profiles', data),
  update: (id, data) => api.put(`/settings/s3/profiles/${id}`, data),
  delete: (id) => api.delete(`/settings/s3/profiles/${id}`),
}