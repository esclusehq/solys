import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import { fetchApi } from '../api/client'
import { useUIStore } from './uiStore'
import * as authApi from '../api/auth'

export const useAuthStore = create(
  persist(
    (set, get) => ({
      user: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,

      checkAuth: async () => {
        set({ isLoading: true })
        try {
          const user = await authApi.getMe()
          set({ 
            user, 
            isAuthenticated: true, 
            isLoading: false 
          })
          return true
        } catch (err) {
          set({ 
            user: null, 
            isAuthenticated: false, 
            isLoading: false 
          })
          return false
        }
      },

      login: async (email, password) => {
        set({ isLoading: true, error: null })
        try {
          await authApi.login(email, password)
          const user = await authApi.getMe()
          set({ 
            user, 
            isAuthenticated: true, 
            isLoading: false 
          })
          useUIStore.getState().setOnboarded(true)
        } catch (err) {
          set({ error: err.message, isLoading: false })
          throw err
        }
      },

      register: async (email, password, name) => {
        set({ isLoading: true, error: null })
        try {
          await authApi.register(email, password)
          const user = await authApi.getMe()
          set({ 
            user: { email, name }, 
            isAuthenticated: true, 
            isLoading: false 
          })
          useUIStore.getState().setOnboarded(true)
        } catch (err) {
          set({ error: err.message, isLoading: false })
          throw err
        }
      },

      logout: async () => {
        set({ isLoading: true })
        try {
          await authApi.logout()
        } catch (err) {
          console.error('Logout error:', err)
        } finally {
          set({
            user: null,
            isAuthenticated: false,
            isLoading: false,
            error: null,
          })
        }
      },

      forgotPassword: async (email) => {
        set({ isLoading: true, error: null })
        try {
          await authApi.forgotPassword(email)
          set({ isLoading: false })
        } catch (err) {
          set({ error: err.message, isLoading: false })
          throw err
        }
      },

      resetPassword: async (token, newPassword) => {
        set({ isLoading: true, error: null })
        try {
          await authApi.resetPassword(token, newPassword)
          set({ isLoading: false })
        } catch (err) {
          set({ error: err.message, isLoading: false })
          throw err
        }
      },

      setUser: (user) => set({ user }),
      
      setAuth: (user) => set({
        user,
        isAuthenticated: true,
        isLoading: false,
      }),

      refreshAccessToken: async () => {
        try {
          await authApi.refreshToken()
          const user = await authApi.getMe()
          set({ user, isAuthenticated: true })
          return true
        } catch (err) {
          console.error('Token refresh failed:', err)
          set({ user: null, isAuthenticated: false })
          return false
        }
      },

      updateProfile: async (data) => {
        set({ isLoading: true })
        try {
          const result = await fetchApi('/auth/profile', {
            method: 'PUT',
            body: JSON.stringify(data),
          })
          set((state) => ({
            user: { ...state.user, ...result },
            isLoading: false,
          }))
          return result
        } catch (err) {
          set({ isLoading: false })
          throw err
        }
      },

      fetchLoginHistory: async () => {
        try {
          return await fetchApi('/auth/login-history')
        } catch (err) {
          console.error('Failed to fetch login history:', err)
          throw err
        }
      },

      requestAccountDeletion: async (password, confirmText) => {
        try {
          return await fetchApi('/auth/account/delete', {
            method: 'POST',
            body: JSON.stringify({ password, confirm_text: confirmText }),
          })
        } catch (err) {
          throw err
        }
      },

      cancelAccountDeletion: async () => {
        try {
          return await fetchApi('/auth/account/cancel-delete', {
            method: 'POST',
          })
        } catch (err) {
          throw err
        }
      },

      transferOwnership: async (targetEmail) => {
        try {
          return await fetchApi('/auth/account/transfer', {
            method: 'POST',
            body: JSON.stringify({ target_email: targetEmail }),
          })
        } catch (err) {
          throw err
        }
      },

      clearError: () => set({ error: null }),
    }),
    {
      name: 'escluse-auth',
      partialize: (state) => ({
        user: state.user,
        isAuthenticated: state.isAuthenticated,
      }),
    }
  )
)
