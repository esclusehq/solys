import { useState, useEffect, useCallback } from 'react'
import { useAuthStore } from '../../store/authStore'
import { useUIStore } from '../../store/uiStore'
import { supabase, signOut } from '../../lib/supabase'
import { webhooksApi, cloudflareApi } from '../../lib/api'
import { useProfile, uploadAvatar } from '../../hooks/useProfile'
import CloudflareSettings from '../../components/settings/CloudflareSettings'

export default function SettingsPage() {
  const { user, logout, refreshUser } = useAuthStore()
  const { addToast } = useUIStore()
  
  const [name, setName] = useState('')
  const [email, setEmail] = useState('')
  const [activeTab, setActiveTab] = useState('profile')
  const [isLoading, setIsLoading] = useState(false)

  // Password change states
  const [currentPassword, setCurrentPassword] = useState('')
  const [newPassword, setNewPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [passwordLoading, setPasswordLoading] = useState(false)

  // 2FA states - read from database
  const [mfaEnabled, setMfaEnabled] = useState(false)
  const [mfaLoading, setMfaLoading] = useState(false)
  const [showMfaSetup, setShowMfaSetup] = useState(false)
  const [mfaQrCode, setMfaQrCode] = useState('')
  const [mfaSecret, setMfaSecret] = useState('')
  const [mfaFactorId, setMfaFactorId] = useState('')
  const [mfaChallengeId, setMfaChallengeId] = useState('')
  const [mfaCode, setMfaCode] = useState('')

  // Profile section states
  const [displayName, setDisplayName] = useState('')
  const [avatarUrl, setAvatarUrl] = useState(user?.avatar_url || '')
  const [savingProfile, setSavingProfile] = useState(false)
  const [uploadingAvatar, setUploadingAvatar] = useState(false)
  const [dragOver, setDragOver] = useState(false)

  // Login history state
  const [loginHistory, setLoginHistory] = useState([])
  const [loginHistoryLoading, setLoginHistoryLoading] = useState(false)
  const [loginHistoryError, setLoginHistoryError] = useState(null)

  // Delete account state
  const [deletePassword, setDeletePassword] = useState('')
  const [deleteConfirmText, setDeleteConfirmText] = useState('')
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false)
  const [deletingAccount, setDeletingAccount] = useState(false)

  // Transfer ownership state
  const [transferEmail, setTransferEmail] = useState('')
  const [showTransfer, setShowTransfer] = useState(false)

  // API Key states
  const [apiKeys, setApiKeys] = useState([])
  const [showNewApiKey, setShowNewApiKey] = useState('')
  const [apiLoading, setApiLoading] = useState(false)

  const getUserRole = () => {
    return user?.role || user?.user_metadata?.role || 'member'
  }

  useEffect(() => {
    if (user) {
      setName(user.name || user.user_metadata?.full_name || '')
      setEmail(user.email || '')
      setDisplayName(user.display_name || '')
      setAvatarUrl(user.avatar_url || '')
    }
  }, [user])

  useEffect(() => {
    checkMfaStatus()
  }, [])

  const checkMfaStatus = async () => {
    try {
      const { data, error } = await supabase.rpc('get_user_security')
      if (error) {
        console.error('Error checking 2FA status:', error)
        setMfaEnabled(false)
        return
      }
      
      const security = data?.[0]
      const enabled = security?.is_2fa_enabled || false
      setMfaEnabled(enabled)
      console.log('2FA status from DB:', enabled)
    } catch (err) {
      console.error('Failed to check 2FA status:', err)
      setMfaEnabled(false)
    }
  }

  // Password Change
  const handlePasswordChange = async (e) => {
    e.preventDefault()
    if (newPassword !== confirmPassword) {
      addToast({ type: 'error', message: 'Passwords do not match' })
      return
    }
    if (newPassword.length < 6) {
      addToast({ type: 'error', message: 'Password must be at least 6 characters' })
      return
    }

    setPasswordLoading(true)
    try {
      const { error } = await supabase.auth.updateUser({
        password: newPassword
      })
      if (error) throw error
      setCurrentPassword('')
      setNewPassword('')
      setConfirmPassword('')
      addToast({ type: 'success', message: 'Password updated successfully!' })
    } catch (err) {
      addToast({ type: 'error', message: err.message })
    } finally {
      setPasswordLoading(false)
    }
  }

  // Profile handlers
  const handleAvatarUpload = async (file) => {
    if (!file) return
    try {
      setUploadingAvatar(true)
      const url = await uploadAvatar(file, user.id)
      await useAuthStore.getState().updateProfile({ avatar_url: url })
      setAvatarUrl(url)
      addToast({ type: 'success', message: 'Avatar updated' })
    } catch (err) {
      addToast({ type: 'error', message: err.message })
    } finally {
      setUploadingAvatar(false)
    }
  }

  const handleAvatarClick = () => {
    const input = document.createElement('input')
    input.type = 'file'
    input.accept = 'image/jpeg,image/png,image/webp'
    input.onchange = (e) => {
      const file = e.target.files?.[0]
      if (file) handleAvatarUpload(file)
    }
    input.click()
  }

  const handleAvatarDrop = (e) => {
    e.preventDefault()
    const file = e.dataTransfer?.files?.[0]
    if (file) handleAvatarUpload(file)
  }

  const handleSaveDisplayName = async () => {
    try {
      setSavingProfile(true)
      await useAuthStore.getState().updateProfile({ display_name: displayName })
      addToast({ type: 'success', message: 'Display name updated' })
    } catch (err) {
      addToast({ type: 'error', message: err.message })
    } finally {
      setSavingProfile(false)
    }
  }

  // Login history handlers
  const fetchLoginHistory = useCallback(async () => {
    try {
      setLoginHistoryLoading(true)
      const data = await useAuthStore.getState().fetchLoginHistory()
      setLoginHistory(Array.isArray(data) ? data : [])
      setLoginHistoryError(null)
    } catch (err) {
      setLoginHistoryError(err.message)
    } finally {
      setLoginHistoryLoading(false)
    }
  }, [])

  useEffect(() => {
    fetchLoginHistory()
  }, [fetchLoginHistory])

  // Delete account handlers
  const handleDeleteAccount = async () => {
    if (deleteConfirmText !== 'DELETE') return
    try {
      setDeletingAccount(true)
      await useAuthStore.getState().requestAccountDeletion(deletePassword, deleteConfirmText)
      addToast({ type: 'success', message: 'Account deletion scheduled. You have 14 days to cancel.' })
      await useAuthStore.getState().logout()
    } catch (err) {
      addToast({ type: 'error', message: err.message || 'Could not process deletion. Please try again.' })
    } finally {
      setDeletingAccount(false)
    }
  }

  const handleTransfer = async () => {
    if (!transferEmail) return
    try {
      const result = await useAuthStore.getState().transferOwnership(transferEmail)
      addToast({ type: 'success', message: result.message || 'Servers transferred successfully' })
      setShowTransfer(false)
      setTransferEmail('')
    } catch (err) {
      addToast({ type: 'error', message: err.message })
    }
  }

  // 2FA Setup
  const handleEnable2FA = async () => {
    setMfaLoading(true)
    try {
      // Use Supabase MFA to generate QR code
      const { data, error } = await supabase.auth.mfa.enroll({
        factorType: 'totp',
        issuer: 'Esluce',
        friendlyName: `Authenticator-${Date.now()}`
      })
      if (error) {
        addToast({ type: 'error', message: error.message })
        return
      }
      setMfaQrCode(data.totp.qr_code)
      setMfaSecret(data.totp.secret)
      setMfaFactorId(data.id)
      setShowMfaSetup(true)
    } catch (err) {
      addToast({ type: 'error', message: err.message })
    } finally {
      setMfaLoading(false)
    }
  }

  const handleVerify2FA = async () => {
    if (!mfaCode) {
      addToast({ type: 'error', message: 'Please enter the code from your authenticator' })
      return
    }
    setMfaLoading(true)
    try {
      // First verify with Supabase MFA
      const { data: challengeData, error: challengeError } = await supabase.auth.mfa.challenge({
        factorId: mfaFactorId
      })
      if (challengeError) throw challengeError

      const { error: verifyError } = await supabase.auth.mfa.verify({
        factorId: mfaFactorId,
        code: mfaCode,
        challengeId: challengeData.id
      })
      if (verifyError) throw verifyError

      // If verified, save to database
      const { error: dbError } = await supabase.rpc('enable_2fa', {
        p_secret_encrypted: mfaSecret
      })
      if (dbError) {
        console.error('DB error:', dbError)
      }

      setMfaEnabled(true)
      setShowMfaSetup(false)
      setMfaCode('')
      setMfaFactorId('')
      addToast({ type: 'success', message: '2FA enabled successfully!' })
    } catch (err) {
      addToast({ type: 'error', message: err.message })
    } finally {
      setMfaLoading(false)
    }
  }

  const handleDisable2FA = async () => {
    // If 2FA is enabled, require code verification
    if (mfaEnabled && !mfaCode) {
      addToast({ type: 'error', message: 'Please enter your 2FA code to disable' })
      return
    }
    if (!confirm('Are you sure you want to disable 2FA?')) return
    
    setMfaLoading(true)
    try {
      // If 2FA is actually enabled, verify the code first
      if (mfaEnabled && mfaCode) {
        const { data: factorsData } = await supabase.auth.mfa.listFactors()
        const totpFactor = factorsData?.factors?.find(f => f.factor_type === 'totp')
        
        if (totpFactor) {
          const { data: challengeData } = await supabase.auth.mfa.challenge({ factorId: totpFactor.id })
          const { error: verifyError } = await supabase.auth.mfa.verify({
            factorId: totpFactor.id,
            code: mfaCode,
            challengeId: challengeData.id
          })
         
          if (verifyError) {
            addToast({ type: 'error', message: 'Invalid 2FA code' })
            setMfaLoading(false)
            return
          }
        }
      }

      // If verified, disable in database
      const { error: dbError } = await supabase.rpc('disable_2fa')
      if (dbError) {
        console.error('DB error:', dbError)
      }

      setMfaEnabled(false)
      setMfaCode('')
      addToast({ type: 'success', message: '2FA disabled!' })
    } catch (err) {
      addToast({ type: 'error', message: err.message })
    } finally {
      setMfaLoading(false)
    }
  }

  // API Keys
  const loadApiKeys = async () => {
    try {
      const { data, error } = await supabase.rpc('list_api_keys')
      if (error) throw error
      setApiKeys(data || [])
    } catch (err) {
      console.error('Error loading API keys:', err)
    }
  }

  useEffect(() => {
    loadApiKeys()
  }, [])

  const generateApiKey = async () => {
    setApiLoading(true)
    try {
      const { data, error } = await supabase.rpc('create_api_key', {
        p_name: 'API Key',
        p_scopes: ['read'],
        p_expires_at: null
      })
      if (error) throw error

      // data is an array with {id, full_key}
      if (data && data.length > 0) {
        setShowNewApiKey(data[0].full_key)
      }
      loadApiKeys()
      addToast({ type: 'success', message: 'API Key generated!' })
    } catch (err) {
      addToast({ type: 'error', message: err.message })
    } finally {
      setApiLoading(false)
    }
  }

  const copyApiKey = (key) => {
    navigator.clipboard.writeText(key)
    addToast({ type: 'success', message: 'Copied to clipboard!' })
  }

  const deleteApiKey = async (id) => {
    if (!confirm('Revoke this API key?')) return
    try {
      const { error } = await supabase.rpc('revoke_api_key', { p_key_id: id })
      if (error) throw error
      loadApiKeys()
      addToast({ type: 'success', message: 'API Key revoked!' })
    } catch (err) {
      addToast({ type: 'error', message: err.message })
    }
  }

  const getRoleBadge = (role) => {
    const roles = {
      owner: { bg: 'bg-purple-600', text: 'Owner' },
      admin: { bg: 'bg-red-600', text: 'Admin' },
      member: { bg: 'bg-blue-600', text: 'Member' },
      viewer: { bg: 'bg-gray-600', text: 'Viewer' },
      friend: { bg: 'bg-green-600', text: 'Friend' },
    }
    const r = roles[role] || roles.viewer
    return (
      <span className={`px-2 py-1 ${r.bg} text-white text-xs rounded`}>
        {r.text}
      </span>
    )
  }

  const handleLogoutAll = async () => {
    if (!confirm('Are you sure you want to log out from all devices?')) return
    
    setIsLoading(true)
    try {
      await signOut()
      logout()
      addToast({ type: 'success', message: 'Logged out from all devices!' })
    } catch (err) {
      addToast({ type: 'error', message: err.message })
    } finally {
      setIsLoading(false)
    }
  }

  const handleLogoutCurrent = async () => {
    if (!confirm('Log out from this device?')) return
    
    try {
      await signOut()
      logout()
      addToast({ type: 'success', message: 'Logged out!' })
    } catch (err) {
      addToast({ type: 'error', message: err.message })
    }
  }

  const renderProfileTab = () => (
    <div className="space-y-6">
      {/* Avatar Upload Section — D-01, D-02, D-03 */}
      <div>
        <div className="flex items-center gap-6">
          <div
            onClick={handleAvatarClick}
            onDrop={handleAvatarDrop}
            onDragOver={(e) => { e.preventDefault(); setDragOver(true) }}
            onDragLeave={() => setDragOver(false)}
            className={`
              relative w-24 h-24 rounded-full overflow-hidden cursor-pointer
              border-2 border-dashed transition-all duration-200
              ${dragOver ? 'border-[#0ddff2] bg-[rgba(13,223,242,0.1)]' : 'border-gray-600 hover:border-[#0ddff2]'}
            `}
          >
            {uploadingAvatar ? (
              <div className="w-full h-full flex items-center justify-center bg-gray-800">
                <svg className="animate-spin w-8 h-8 text-[#0ddff2]" viewBox="0 0 24 24" fill="none">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                </svg>
              </div>
            ) : avatarUrl ? (
              <>
                <img src={avatarUrl} alt="Avatar" className="w-full h-full object-cover" />
                <div className="absolute inset-0 bg-black/50 opacity-0 hover:opacity-100 transition-opacity flex items-center justify-center">
                  <span className="text-white text-xs">Change avatar</span>
                </div>
              </>
            ) : (
              <div className="w-full h-full flex flex-col items-center justify-center bg-gray-800 text-gray-400">
                <svg className="w-8 h-8 mb-1" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                  <polyline points="17 8 12 3 7 8" />
                  <line x1="12" y1="3" x2="12" y2="15" />
                </svg>
                <span className="text-[10px] text-center leading-tight px-1">Click or drag image here</span>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Display Name Section — D-10 */}
      <div>
        <h3 className="text-lg font-semibold text-white mb-4">Display Name</h3>
        <p className="text-xs text-gray-500 mb-2">Shown in the sidebar</p>
        <div className="flex gap-3">
          <input
            type="text"
            value={displayName}
            onChange={(e) => setDisplayName(e.target.value)}
            placeholder="Enter display name"
            className="flex-1 px-4 py-2 bg-gray-700 text-white rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <button
            onClick={handleSaveDisplayName}
            disabled={savingProfile}
            className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
          >
            {savingProfile ? 'Saving...' : 'Save Changes'}
          </button>
        </div>
      </div>

      <div className="pt-6 border-t border-gray-700">
        <h3 className="text-lg font-medium text-white mb-4">Profile Information</h3>
        <form className="space-y-4">
          <div>
            <label className="block text-gray-400 mb-1">Name</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full px-4 py-2 bg-gray-700 text-white rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          
          <div>
            <label className="block text-gray-400 mb-1">Email</label>
            <input
              type="email"
              value={email}
              disabled
              className="w-full px-4 py-2 bg-gray-600 text-gray-400 rounded cursor-not-allowed"
            />
            <p className="text-gray-500 text-xs mt-1">Email cannot be changed</p>
          </div>

          <button
            type="submit"
            className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
          >
            Save Changes
          </button>
        </form>
      </div>

      <div className="pt-6 border-t border-gray-700">
        <h3 className="text-lg font-medium text-white mb-4">Change Password</h3>
        <form onSubmit={handlePasswordChange} className="space-y-4">
          <div>
            <label className="block text-gray-400 mb-1">Current Password</label>
            <input
              type="password"
              value={currentPassword}
              onChange={(e) => setCurrentPassword(e.target.value)}
              className="w-full px-4 py-2 bg-gray-700 text-white rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          
          <div>
            <label className="block text-gray-400 mb-1">New Password</label>
            <input
              type="password"
              value={newPassword}
              onChange={(e) => setNewPassword(e.target.value)}
              className="w-full px-4 py-2 bg-gray-700 text-white rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          <div>
            <label className="block text-gray-400 mb-1">Confirm New Password</label>
            <input
              type="password"
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
              className="w-full px-4 py-2 bg-gray-700 text-white rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          <button
            type="submit"
            disabled={passwordLoading}
            className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
          >
            {passwordLoading ? 'Updating...' : 'Update Password'}
          </button>
        </form>
      </div>

      {/* Login History Section — D-04, D-05 */}
      <div className="pt-6 border-t border-gray-700">
        <h3 className="text-lg font-semibold text-white mb-4">Login History</h3>

        {loginHistoryLoading ? (
          <div className="space-y-3">
            {[1, 2, 3].map((i) => (
              <div key={i} className="h-10 bg-gray-700 rounded animate-pulse" />
            ))}
          </div>
        ) : loginHistoryError ? (
          <div className="text-center py-8">
            <p className="text-gray-400 mb-2">Could not load login history. Please try again.</p>
            <button
              onClick={fetchLoginHistory}
              className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
            >
              Retry
            </button>
          </div>
        ) : loginHistory.length === 0 ? (
          <div className="text-center py-8 text-gray-400">
            <svg className="w-12 h-12 mx-auto mb-2 opacity-50" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <rect x="3" y="4" width="18" height="18" rx="2" ry="2" />
              <line x1="16" y1="2" x2="16" y2="6" />
              <line x1="8" y1="2" x2="8" y2="6" />
              <line x1="3" y1="10" x2="21" y2="10" />
            </svg>
            <p>No login history found</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="text-gray-400 border-b border-gray-700">
                  <th className="text-left py-2 font-normal">Timestamp</th>
                  <th className="text-left py-2 font-normal">IP Address</th>
                  <th className="text-left py-2 font-normal">Device/Browser</th>
                  <th className="text-left py-2 font-normal">Provider</th>
                  <th className="text-left py-2 font-normal">Session ID</th>
                </tr>
              </thead>
              <tbody className="text-gray-300">
                {loginHistory.map((entry) => (
                  <tr key={entry.id} className="border-b border-gray-700 hover:bg-[rgba(255,255,255,0.03)]">
                    <td className="py-2">{new Date(entry.created_at).toLocaleString()}</td>
                    <td className="py-2 font-mono text-xs">{entry.ip_address || '-'}</td>
                    <td className="py-2">{[entry.device_info, entry.browser_info].filter(Boolean).join(' / ') || entry.user_agent?.substring(0, 50) || '-'}</td>
                    <td className="py-2">{entry.oauth_provider || 'Email/Password'}</td>
                    <td className="py-2 font-mono text-xs text-gray-500">{entry.session_id?.substring(0, 8) || '-'}...</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Delete Account Section — D-07, D-08, D-09 */}
      <div className="pt-6 border-t border-gray-700">
        <div className="border-l-4 border-red-500 pl-4">
          <h3 className="text-lg font-semibold text-red-400 mb-2">Danger Zone</h3>
          <p className="text-sm text-gray-400 mb-4">
            Once you delete your account, there is no going back. Your servers and data will be permanently deleted after a 14-day grace period.
          </p>

          {!showDeleteConfirm ? (
            <div className="space-y-3">
              <button
                onClick={() => setShowDeleteConfirm(true)}
                className="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700"
              >
                Delete Account
              </button>

              {/* Transfer ownership link — D-08 */}
              {!showTransfer ? (
                <button
                  onClick={() => setShowTransfer(true)}
                  className="ml-3 text-sm text-blue-400 hover:text-blue-300"
                >
                  Transfer ownership of your servers
                </button>
              ) : (
                <div className="mt-3 flex gap-3">
                  <input
                    type="email"
                    value={transferEmail}
                    onChange={(e) => setTransferEmail(e.target.value)}
                    placeholder="Target user email"
                    className="flex-1 px-4 py-2 bg-gray-700 text-white rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                  <button
                    onClick={handleTransfer}
                    className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
                  >
                    Transfer
                  </button>
                  <button
                    onClick={() => { setShowTransfer(false); setTransferEmail('') }}
                    className="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-500"
                  >
                    Cancel
                  </button>
                </div>
              )}
            </div>
          ) : (
            <div className="space-y-4 p-4 bg-gray-800 rounded border border-red-900">
              <p className="text-sm text-red-300">This action cannot be undone. Your account will be permanently deleted after 14 days.</p>

              <div>
                <label className="block text-gray-400 mb-1 text-sm">Enter your password to continue</label>
                <input
                  type="password"
                  value={deletePassword}
                  onChange={(e) => setDeletePassword(e.target.value)}
                  className="w-full px-4 py-2 bg-gray-700 text-white rounded focus:outline-none focus:ring-2 focus:ring-red-500"
                />
              </div>

              <div>
                <label className="block text-gray-400 mb-1 text-sm">Type DELETE to confirm</label>
                <input
                  type="text"
                  value={deleteConfirmText}
                  onChange={(e) => setDeleteConfirmText(e.target.value)}
                  className="w-full px-4 py-2 bg-gray-700 text-white rounded focus:outline-none focus:ring-2 focus:ring-red-500"
                />
              </div>

              <div className="flex gap-3">
                <button
                  onClick={handleDeleteAccount}
                  disabled={deletingAccount || deleteConfirmText !== 'DELETE' || !deletePassword}
                  className="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700 disabled:opacity-50"
                >
                  {deletingAccount ? 'Processing...' : 'Permanently Delete Account'}
                </button>
                <button
                  onClick={() => { setShowDeleteConfirm(false); setDeletePassword(''); setDeleteConfirmText('') }}
                  className="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-500"
                >
                  Cancel
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  )

  const renderSecurityTab = () => (
    <div className="space-y-6">
      <div>
        <div className="flex items-center justify-between">
          <h3 className="text-lg font-medium text-white mb-2">Two-Factor Authentication</h3>
          <button
            onClick={checkMfaStatus}
            className="text-gray-400 hover:text-white text-sm"
          >
            ↻ Refresh
          </button>
        </div>
        <p className="text-gray-400 text-sm mb-4">
          Add an extra layer of security using Google Authenticator or similar apps
        </p>
        
        {mfaEnabled ? (
          <div className="space-y-4">
            <div className="flex items-center gap-4">
              <div className="flex items-center gap-2">
                <div className="w-2 h-2 bg-green-500 rounded-full"></div>
                <span className="text-green-400 text-sm">2FA is enabled</span>
              </div>
            </div>
            <div>
              <label className="block text-gray-400 mb-1">Enter 2FA code to disable</label>
              <input
                type="text"
                value={mfaCode}
                onChange={(e) => setMfaCode(e.target.value)}
                placeholder="123456"
                maxLength={6}
                className="w-full px-4 py-2 bg-gray-700 text-white rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <button
              onClick={handleDisable2FA}
              disabled={mfaLoading || mfaCode.length < 6}
              className="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700 disabled:opacity-50"
            >
              {mfaLoading ? 'Disabling...' : 'Disable 2FA'}
            </button>
          </div>
        ) : showMfaSetup ? (
          <div className="space-y-4">
            <div className="p-4 bg-gray-700 rounded">
              <p className="text-white text-sm mb-2">Scan this QR code with your authenticator app:</p>
              <img src={mfaQrCode} alt="2FA QR Code" className="w-48 h-48 bg-white p-2 rounded" />
              <p className="text-gray-400 text-xs mt-2">Or enter this secret: {mfaSecret}</p>
            </div>
            <div>
              <label className="block text-gray-400 mb-1">Enter code from authenticator</label>
              <input
                type="text"
                value={mfaCode}
                onChange={(e) => setMfaCode(e.target.value)}
                placeholder="123456"
                className="w-full px-4 py-2 bg-gray-700 text-white rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <div className="flex gap-2">
              <button
                onClick={handleVerify2FA}
                disabled={mfaLoading}
                className="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700 disabled:opacity-50"
              >
                {mfaLoading ? 'Verifying...' : 'Verify & Enable'}
              </button>
              <button
                onClick={() => setShowMfaSetup(false)}
                className="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-500"
              >
                Cancel
              </button>
            </div>
          </div>
        ) : (
          <button
            onClick={handleEnable2FA}
            disabled={mfaLoading}
            className="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700 disabled:opacity-50"
          >
            {mfaLoading ? 'Setting up...' : 'Enable 2FA'}
          </button>
        )}
      </div>

      <div className="pt-6 border-t border-gray-700">
        <h3 className="text-lg font-medium text-white mb-2">Active Sessions</h3>
        <p className="text-gray-400 text-sm mb-4">Manage your active login sessions</p>
        
        <div className="space-y-3">
          <div className="flex items-center justify-between p-3 bg-gray-700 rounded">
            <div className="flex items-center gap-3">
              <div className="w-2 h-2 bg-green-500 rounded-full"></div>
              <div>
                <p className="text-white text-sm">Current Session</p>
                <p className="text-gray-400 text-xs">This device</p>
              </div>
            </div>
            <button
              onClick={handleLogoutCurrent}
              className="text-red-400 text-sm hover:text-red-300"
            >
              Logout
            </button>
          </div>
        </div>
      </div>

      <div className="pt-6 border-t border-gray-700">
        <h3 className="text-lg font-medium text-white mb-2">Logout All Devices</h3>
        <p className="text-gray-400 text-sm mb-4">
          This will log you out from all devices except this one.
        </p>
        <button
          onClick={handleLogoutAll}
          disabled={isLoading}
          className="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700 disabled:opacity-50"
        >
          {isLoading ? 'Logging out...' : 'Logout All Devices'}
        </button>
      </div>
    </div>
  )

  const renderApiTab = () => (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-medium text-white mb-2">API Keys</h3>
        <p className="text-gray-400 text-sm mb-4">
          Use API keys to access Esluce from the command line or scripts.
        </p>

        {showNewApiKey && (
          <div className="p-4 bg-green-900 border border-green-700 rounded mb-4">
            <p className="text-green-400 text-sm font-medium mb-2">Your new API key (copy now - won't show again):</p>
            <div className="flex gap-2">
              <code className="flex-1 p-2 bg-gray-800 text-green-400 rounded text-sm font-mono break-all">
                {showNewApiKey}
              </code>
              <button
                onClick={() => copyApiKey(showNewApiKey)}
                className="px-3 py-2 bg-green-700 text-white rounded hover:bg-green-600"
              >
                Copy
              </button>
            </div>
          </div>
        )}

        <button
          onClick={generateApiKey}
          disabled={apiLoading}
          className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
        >
          {apiLoading ? 'Generating...' : 'Generate New API Key'}
        </button>
      </div>

      {apiKeys.length > 0 && (
        <div className="pt-6 border-t border-gray-700">
          <h3 className="text-lg font-medium text-white mb-4">Your API Keys</h3>
          <div className="space-y-3">
            {apiKeys.map((apiKey) => (
              <div key={apiKey.id} className="flex items-center justify-between p-3 bg-gray-700 rounded">
                <div>
                  <p className="text-white text-sm">{apiKey.name}</p>
                  <p className="text-gray-400 text-xs">
                    {apiKey.key_prefix}... • Created: {new Date(apiKey.created_at).toLocaleDateString()}
                    {apiKey.expires_at && ` • Expires: ${new Date(apiKey.expires_at).toLocaleDateString()}`}
                  </p>
                  {apiKey.is_active === false && (
                    <span className="text-red-400 text-xs">Revoked</span>
                  )}
                </div>
                <div className="flex gap-2">
                  <button
                    onClick={() => deleteApiKey(apiKey.id)}
                    disabled={!apiKey.is_active}
                    className="px-3 py-1 bg-red-600 text-white text-sm rounded hover:bg-red-500 disabled:opacity-50"
                  >
                    Delete
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )

  const isAdmin = ['owner', 'admin'].includes(getUserRole())

  const tabs = [
    { id: 'profile', label: 'Profile' },
    { id: 'team', label: 'Team' },
    { id: 'security', label: 'Security' },
    { id: 'api', label: 'API Keys' },
    { id: 'webhooks', label: 'Webhooks' },
    ...(isAdmin ? [{ id: 'cloudflare', label: 'Cloudflare DNS' }] : []),
  ]

  // Mock team members (nanti bisa dari API)
  const [teamMembers, setTeamMembers] = useState([])

  useEffect(() => {
    if (user) {
      setTeamMembers([
        { id: 1, name: 'You', email: user.email || '', role: getUserRole() },
      ])
    }
  }, [user, getUserRole])

  const [inviteEmail, setInviteEmail] = useState('')
  const [inviteRole, setInviteRole] = useState('friend')

  const handleInvite = (e) => {
    e.preventDefault()
    if (!inviteEmail) return
    
    // Add to team (mock)
    const newMember = {
      id: Date.now(),
      name: inviteEmail.split('@')[0],
      email: inviteEmail,
      role: inviteRole,
      status: 'pending',
    }
    setTeamMembers([...teamMembers, newMember])
    setInviteEmail('')
    addToast({ type: 'success', message: 'Invitation sent!' })
  }

  const handleRemoveMember = (id) => {
    if (!confirm('Remove this member?')) return
    setTeamMembers(teamMembers.filter(m => m.id !== id))
    addToast({ type: 'success', message: 'Member removed!' })
  }

  const renderTeamTab = () => (
    <div className="space-y-6">
      {/* Current Role */}
      <div>
        <h3 className="text-lg font-medium text-white mb-2">Your Role</h3>
        <div className="flex items-center gap-3">
          {getRoleBadge(getUserRole())}
        <span className="text-gray-400 text-sm">
          {getUserRole() === 'owner' ? 'Full access to all features' :
           getUserRole() === 'admin' ? 'Can manage all servers and users' :
           getUserRole() === 'member' ? 'Can manage assigned servers' :
           getUserRole() === 'friend' ? 'Can view and join your servers' :
           'View-only access'}
        </span>
        </div>
      </div>

      {/* Invite Team Members */}
      {(getUserRole() === 'owner' || getUserRole() === 'admin') && (
        <div className="pt-6 border-t border-gray-700">
          <h3 className="text-lg font-medium text-white mb-4">Invite Team Members</h3>
          <form onSubmit={handleInvite} className="flex gap-3">
            <input
              type="email"
              value={inviteEmail}
              onChange={(e) => setInviteEmail(e.target.value)}
              placeholder="friend@email.com"
              className="flex-1 px-4 py-2 bg-gray-700 text-white rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <select
              value={inviteRole}
              onChange={(e) => setInviteRole(e.target.value)}
              className="px-4 py-2 bg-gray-700 text-white rounded focus:outline-none"
            >
              <option value="friend">Friend</option>
              <option value="viewer">Viewer</option>
              <option value="member">Member</option>
              <option value="admin">Admin</option>
            </select>
            <button
              type="submit"
              className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
            >
              Invite
            </button>
          </form>
        </div>
      )}

      {/* Team Members List */}
      <div className="pt-6 border-t border-gray-700">
        <h3 className="text-lg font-medium text-white mb-4">Team Members</h3>
        <div className="space-y-3">
          {teamMembers.map((member) => (
            <div key={member.id} className="flex items-center justify-between p-3 bg-gray-700 rounded">
              <div className="flex items-center gap-3">
                <div className="w-8 h-8 bg-gray-600 rounded-full flex items-center justify-center text-white text-sm">
                  {member.name[0].toUpperCase()}
                </div>
                <div>
                  <p className="text-white text-sm">
                    {member.name}
                    {member.id === 1 && <span className="text-gray-400 text-xs ml-2">(You)</span>}
                  </p>
                  <p className="text-gray-400 text-xs">{member.email}</p>
                </div>
              </div>
              <div className="flex items-center gap-2">
                {getRoleBadge(member.role)}
                {member.status === 'pending' && (
                  <span className="text-yellow-400 text-xs">Pending</span>
                )}
                {member.id !== 1 && (getUserRole() === 'owner' || getUserRole() === 'admin') && (
                  <button
                    onClick={() => handleRemoveMember(member.id)}
                    className="text-red-400 hover:text-red-300 text-sm ml-2"
                  >
                    Remove
                  </button>
                )}
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Role Permissions */}
      <div className="pt-6 border-t border-gray-700">
        <h3 className="text-lg font-medium text-white mb-4">Role Permissions</h3>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="text-gray-400 border-b border-gray-700">
                <th className="text-left py-2">Permission</th>
                <th className="text-center py-2">Owner</th>
                <th className="text-center py-2">Admin</th>
                <th className="text-center py-2">Member</th>
                <th className="text-center py-2">Friend</th>
                <th className="text-center py-2">Viewer</th>
              </tr>
            </thead>
            <tbody className="text-gray-300">
              <tr className="border-b border-gray-700">
                <td className="py-2">Manage all servers</td>
                <td className="text-center">✓</td>
                <td className="text-center">✓</td>
                <td className="text-center">-</td>
                <td className="text-center">-</td>
                <td className="text-center">-</td>
              </tr>
              <tr className="border-b border-gray-700">
                <td className="py-2">Create servers</td>
                <td className="text-center">✓</td>
                <td className="text-center">✓</td>
                <td className="text-center">✓</td>
                <td className="text-center">-</td>
                <td className="text-center">-</td>
              </tr>
              <tr className="border-b border-gray-700">
                <td className="py-2">Start/Stop servers</td>
                <td className="text-center">✓</td>
                <td className="text-center">✓</td>
                <td className="text-center">✓</td>
                <td className="text-center">✓</td>
                <td className="text-center">-</td>
              </tr>
              <tr className="border-b border-gray-700">
                <td className="py-2">View servers</td>
                <td className="text-center">✓</td>
                <td className="text-center">✓</td>
                <td className="text-center">✓</td>
                <td className="text-center">✓</td>
                <td className="text-center">✓</td>
              </tr>
              <tr className="border-b border-gray-700">
                <td className="py-2">Manage team</td>
                <td className="text-center">✓</td>
                <td className="text-center">✓</td>
                <td className="text-center">-</td>
                <td className="text-center">-</td>
                <td className="text-center">-</td>
              </tr>
              <tr className="border-b border-gray-700">
                <td className="py-2">Billing</td>
                <td className="text-center">✓</td>
                <td className="text-center">-</td>
                <td className="text-center">-</td>
                <td className="text-center">-</td>
                <td className="text-center">-</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  )

  // Webhooks Tab
  const [webhooks, setWebhooks] = useState([])
  const [webhooksLoading, setWebhooksLoading] = useState(false)
  const [testingWebhook, setTestingWebhook] = useState(null)
  const [retryingWebhook, setRetryingWebhook] = useState(null)

  const loadWebhooks = async () => {
    setWebhooksLoading(true)
    try {
      const data = await webhooksApi.list()
      setWebhooks(data || [])
    } catch (err) {
      console.error('Error loading webhooks:', err)
    } finally {
      setWebhooksLoading(false)
    }
  }

  useEffect(() => {
    if (activeTab === 'webhooks') {
      loadWebhooks()
    }
  }, [activeTab])

  const handleTestWebhook = async (id) => {
    setTestingWebhook(id)
    try {
      await webhooksApi.test(id)
      addToast({ type: 'success', message: 'Test webhook sent!' })
      loadWebhooks()
    } catch (err) {
      addToast({ type: 'error', message: err.message })
    } finally {
      setTestingWebhook(null)
    }
  }

  const handleRetryWebhook = async (id) => {
    setRetryingWebhook(id)
    try {
      await webhooksApi.retry(id)
      addToast({ type: 'success', message: 'Webhook retry initiated!' })
      loadWebhooks()
    } catch (err) {
      addToast({ type: 'error', message: err.message })
    } finally {
      setRetryingWebhook(null)
    }
  }

  const formatRelativeTime = (dateStr) => {
    if (!dateStr) return '-'
    const date = new Date(dateStr)
    const now = new Date()
    const diffMs = now - date
    const diffMins = Math.floor(diffMs / 60000)
    const diffHours = Math.floor(diffMins / 60)
    const diffDays = Math.floor(diffHours / 24)
    if (diffMins < 1) return 'Just now'
    if (diffMins < 60) return `${diffMins}m ago`
    if (diffHours < 24) return `${diffHours}h ago`
    if (diffDays < 7) return `${diffDays}d ago`
    return date.toLocaleDateString()
  }

  const renderWebhooksTab = () => (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-medium text-white mb-2">Webhooks</h3>
        <p className="text-gray-400 text-sm mb-4">
          Monitor webhook delivery status and retry failed deliveries.
        </p>
      </div>

      {webhooksLoading ? (
        <div className="text-gray-400">Loading...</div>
      ) : webhooks.length === 0 ? (
        <div className="text-gray-400 text-center py-8">
          No webhooks configured. Create one via API to get started.
        </div>
      ) : (
        <div className="space-y-4">
          {webhooks.map((webhook) => (
            <div key={webhook.id} className="p-4 bg-gray-700 rounded-lg">
              <div className="flex items-center justify-between mb-3">
                <div>
                  <h4 className="text-white font-medium">{webhook.name}</h4>
                  <p className="text-gray-400 text-sm font-mono truncate max-w-md">
                    {webhook.url}
                  </p>
                </div>
                <div className="flex items-center gap-2">
                  <span className={`px-2 py-1 rounded text-xs ${
                    webhook.failure_count > 0 
                      ? 'bg-red-600 text-white' 
                      : 'bg-green-600 text-white'
                  }`}>
                    {webhook.failure_count > 0 
                      ? `${webhook.failure_count} failures` 
                      : 'Healthy'}
                  </span>
                </div>
              </div>
              
              <div className="flex items-center gap-4 text-sm text-gray-400 mb-4">
                <span>Last delivery: {formatRelativeTime(webhook.last_delivery_at)}</span>
                {webhook.last_failure_at && (
                  <span className="text-red-400">Last failure: {formatRelativeTime(webhook.last_failure_at)}</span>
                )}
              </div>

              <div className="flex gap-2">
                <button
                  onClick={() => handleTestWebhook(webhook.id)}
                  disabled={testingWebhook === webhook.id}
                  className="px-3 py-1.5 bg-blue-600 text-white text-sm rounded hover:bg-blue-700 disabled:opacity-50"
                >
                  {testingWebhook === webhook.id ? 'Testing...' : 'Test'}
                </button>
                {webhook.failure_count > 0 && (
                  <button
                    onClick={() => handleRetryWebhook(webhook.id)}
                    disabled={retryingWebhook === webhook.id}
                    className="px-3 py-1.5 bg-orange-600 text-white text-sm rounded hover:bg-orange-700 disabled:opacity-50"
                  >
                    {retryingWebhook === webhook.id ? 'Retrying...' : 'Retry'}
                  </button>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold text-white mb-6">Settings</h1>
      
      {/* Tabs */}
      <div className="flex gap-4 border-b border-gray-700 mb-6">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`pb-2 px-1 text-sm font-medium ${
              activeTab === tab.id
                ? 'text-blue-400 border-b-2 border-blue-400'
                : 'text-gray-400 hover:text-white'
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Content */}
      {activeTab === 'profile' && renderProfileTab()}
      {activeTab === 'team' && renderTeamTab()}
      {activeTab === 'security' && renderSecurityTab()}
      {activeTab === 'api' && renderApiTab()}
      {activeTab === 'webhooks' && renderWebhooksTab()}
      {activeTab === 'cloudflare' && <CloudflareSettings />}
    </div>
  )
}