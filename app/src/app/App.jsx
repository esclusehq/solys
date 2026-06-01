import { Routes, Route, Navigate } from 'react-router-dom'
import { useEffect } from 'react'
import { useAuthStore } from '../store/authStore'
import { useUIStore } from '../store/uiStore'
import ProtectedRoute from '../components/ProtectedRoute'
import Onboarding from '../components/Onboarding'
import ToastContainer from '../components/ToastContainer'
import NotificationCenter from '../components/NotificationCenter'
import EmailVerificationBanner from '../components/EmailVerificationBanner'
import VerifiedRoute from '../components/VerifiedRoute'
import { signOut } from '../lib/supabase'

import LoginPage from '../pages/auth/LoginPage'
import RegisterPage from '../pages/auth/RegisterPage'
import AuthCallback from '../pages/auth/AuthCallback'
import ForgotPasswordPage from '../pages/auth/ForgotPasswordPage'
import ResetPasswordPage from '../pages/auth/ResetPasswordPage'
import VerifyEmailPage from '../pages/auth/VerifyEmailPage'
import MfaVerifyPage from '../pages/auth/MfaVerifyPage'
import DashboardPage from '../pages/dashboard/DashboardPage'
import ServerManagerPage from '../pages/servers/ServerManagerPage'
import Nodes from '../pages/Nodes'
import ServerDetailsPage from '../pages/servers/ServerDetailsPage'
import ScheduledTasksPage from '../features/scheduling/ScheduledTasksPage'
import BillingPage from '../pages/billing/BillingPage'
import SettingsPage from '../pages/settings/SettingsPage'

import TemplateLibraryPage from '../pages/templates/TemplateLibraryPage'
import TemplateCreatePage from '../pages/templates/TemplateCreatePage'
import ModBrowserPage from '../pages/templates/ModBrowserPage'

export default function App() {
  const { isAuthenticated, logout } = useAuthStore()
  const { isOnboarded, sidebarOpen, toggleSidebar, theme, setTheme } = useUIStore()
  
  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme)
  }, [theme])
  
  const handleLogout = async () => {
    try {
      await signOut()
    } catch (e) {
      console.error('SignOut error:', e)
    }
    logout()
  }

  return (
    <>
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route path="/register" element={<RegisterPage />} />
        <Route path="/auth/callback" element={<AuthCallback />} />
        <Route path="/forgot-password" element={<ForgotPasswordPage />} />
        <Route path="/reset-password" element={<ResetPasswordPage />} />
        <Route path="/verify-email" element={<VerifyEmailPage />} />
        <Route path="/mfa-verify" element={<MfaVerifyPage />} />
        
        <Route
          path="/*"
          element={
            <ProtectedRoute>
              {!isOnboarded && <Onboarding />}
              <EmailVerificationBanner />
              <div className="flex min-h-screen bg-gray-900">
                <aside className={`${sidebarOpen ? 'w-64' : 'w-16'} bg-gray-800 border-r border-gray-700 transition-all`}>
                  <div className="p-4 flex items-center justify-between">
                    {sidebarOpen && <><img src="/logo.svg" alt="Escluse" className="w-16 h-16 inline-block mr-1" /><span className="font-bold text-white">Escluse</span></>}
                    <button onClick={toggleSidebar} className="text-gray-400 hover:text-white">
                      ☰
                    </button>
                  </div>
                  
                  {sidebarOpen && (
                    <nav className="px-4 space-y-2">
                      <a href="/" className="block py-2 text-gray-400 hover:text-white">Dashboard</a>
                      <a href="/servers" className="block py-2 text-gray-400 hover:text-white">Servers</a>
                      <a href="/nodes" className="block py-2 text-gray-400 hover:text-white">Nodes</a>
                      <a href="/templates" className="block py-2 text-gray-400 hover:text-white">Templates</a>
                      <a href="/mods" className="block py-2 text-gray-400 hover:text-white">Mod Browser</a>
                      <a href="/billing" className="block py-2 text-gray-400 hover:text-white">Billing</a>
                      <a href="/settings" className="block py-2 text-gray-400 hover:text-white">Settings</a>
                    </nav>
                  )}
                </aside>
                
                <main className="flex-1">
                  <header className="bg-gray-800 border-b border-gray-700 p-4 flex justify-between items-center">
                    <div className="flex items-center gap-3">
                      <h1 className="text-white font-semibold">Escluse Dashboard</h1>
                      <span className="px-2 py-0.5 rounded-full bg-yellow-500/20 text-yellow-400 text-xs font-medium">Alpha</span>
                    </div>
                    <div className="flex items-center gap-4">
                      <button
                        onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
                        className="text-xl cursor-pointer"
                        title={`Switch to ${theme === 'dark' ? 'light' : 'dark'} mode`}
                      >
                        {theme === 'dark' ? '☀️' : '🌙'}
                      </button>
                      <NotificationCenter />
                      <button 
                        onClick={handleLogout}
                        className="text-gray-400 hover:text-white text-sm"
                      >
                        Logout
                      </button>
                    </div>
                  </header>
                  
                  <Routes>
                    <Route path="/" element={<DashboardPage />} />
                    <Route path="/dashboard" element={<DashboardPage />} />
                    <Route path="/servers" element={<ServerManagerPage />} />
                    <Route path="/nodes" element={<Nodes />} />
                    <Route path="/servers/:id" element={<ServerDetailsPage />} />
                    <Route path="/servers/:id/tasks" element={<ScheduledTasksPage />} />
                    <Route path="/billing" element={<VerifiedRoute><BillingPage /></VerifiedRoute>} />
                    <Route path="/settings" element={<SettingsPage />} />
                    <Route path="/templates" element={<TemplateLibraryPage />} />
                    <Route path="/templates/create" element={<TemplateCreatePage />} />
                    <Route path="/templates/:id/edit" element={<TemplateCreatePage />} />
                    <Route path="/mods" element={<ModBrowserPage />} />
                  </Routes>
                </main>
              </div>
            </ProtectedRoute>
          }
        />
      </Routes>
      
      <ToastContainer />
    </>
  )
}