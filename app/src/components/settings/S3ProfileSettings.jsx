import { useState, useEffect } from 'react'
import { useUIStore } from '../../store/uiStore'
import { s3ProfilesApi } from '../../lib/api'

export default function S3ProfileSettings() {
  const { addToast } = useUIStore()

  const [profiles, setProfiles] = useState([])
  const [loading, setLoading] = useState(false)
  const [saving, setSaving] = useState(false)
  const [showForm, setShowForm] = useState(false)
  const [editingId, setEditingId] = useState(null)

  const [name, setName] = useState('')
  const [endpoint, setEndpoint] = useState('')
  const [region, setRegion] = useState('')
  const [bucket, setBucket] = useState('')
  const [accessKey, setAccessKey] = useState('')
  const [secretKey, setSecretKey] = useState('')
  const [isDefault, setIsDefault] = useState(false)

  useEffect(() => {
    loadProfiles()
  }, [])

  const loadProfiles = async () => {
    setLoading(true)
    try {
      const data = await s3ProfilesApi.list()
      setProfiles(data || [])
    } catch (err) {
      console.error('Failed to load S3 profiles:', err)
      addToast({ type: 'error', message: 'Failed to load S3 profiles' })
    } finally {
      setLoading(false)
    }
  }

  const resetForm = () => {
    setName('')
    setEndpoint('')
    setRegion('')
    setBucket('')
    setAccessKey('')
    setSecretKey('')
    setIsDefault(false)
    setEditingId(null)
    setShowForm(false)
  }

  const handleEdit = (profile) => {
    setName(profile.name)
    setEndpoint(profile.endpoint)
    setRegion(profile.region || '')
    setBucket(profile.bucket)
    setAccessKey(profile.access_key)
    setSecretKey('')
    setIsDefault(profile.is_default || false)
    setEditingId(profile.id)
    setShowForm(true)
  }

  const handleDelete = async (id, profileName) => {
    if (!confirm(`Delete S3 profile "${profileName}" permanently?`)) return
    try {
      await s3ProfilesApi.delete(id)
      setProfiles(prev => prev.filter(p => p.id !== id))
      addToast({ type: 'success', message: `S3 profile "${profileName}" deleted` })
    } catch (err) {
      addToast({ type: 'error', message: err.message })
    }
  }

  const handleSubmit = async (e) => {
    e.preventDefault()
    setSaving(true)
    try {
      const payload = {
        name,
        endpoint,
        region: region || null,
        bucket,
        access_key: accessKey,
        is_default: isDefault || null,
      }
      if (!editingId || secretKey) {
        payload.secret_key = secretKey
      }

      if (editingId) {
        await s3ProfilesApi.update(editingId, payload)
        addToast({ type: 'success', message: `S3 profile "${name}" updated` })
      } else {
        await s3ProfilesApi.create(payload)
        addToast({ type: 'success', message: `S3 profile "${name}" created` })
      }
      resetForm()
      loadProfiles()
    } catch (err) {
      addToast({ type: 'error', message: err.message })
    } finally {
      setSaving(false)
    }
  }

  if (loading) {
    return <div className="text-gray-400">Loading S3 profiles...</div>
  }

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-medium text-white mb-2">S3 Storage Profiles</h3>
        <p className="text-gray-400 text-sm mb-4">
          Configure S3-compatible storage providers for backup destinations.
          Supports AWS S3, Cloudflare R2, MinIO, DigitalOcean Spaces.
        </p>
      </div>

      {profiles.length === 0 && !showForm && (
        <div className="p-4 bg-gray-700 rounded-lg text-gray-400 text-sm">
          No S3 profiles configured. Click "Add Profile" to create one.
        </div>
      )}

      {profiles.length > 0 && (
        <div className="space-y-3">
          {profiles.map(profile => (
            <div key={profile.id} className="p-4 bg-gray-700 rounded-lg">
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <span className="text-white font-medium">{profile.name}</span>
                    {profile.is_default && (
                      <span className="text-xs bg-blue-600 text-white px-2 py-0.5 rounded">Default</span>
                    )}
                  </div>
                  <p className="text-gray-400 text-xs mt-1">
                    {profile.endpoint}{profile.region ? ` | ${profile.region}` : ''} | {profile.bucket}
                  </p>
                </div>
                <div className="flex gap-2 ml-4">
                  <button
                    onClick={() => handleEdit(profile)}
                    className="px-3 py-1 bg-blue-600 text-white text-xs rounded hover:bg-blue-700"
                  >
                    Edit
                  </button>
                  <button
                    onClick={() => handleDelete(profile.id, profile.name)}
                    className="px-3 py-1 bg-red-600 text-white text-xs rounded hover:bg-red-700"
                  >
                    Delete
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {!showForm && (
        <button
          onClick={() => { resetForm(); setShowForm(true) }}
          className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 text-sm"
        >
          Add Profile
        </button>
      )}

      {showForm && (
        <form onSubmit={handleSubmit} className="space-y-4 p-4 bg-gray-700 rounded-lg">
          <div>
            <label className="block text-gray-400 mb-1">Profile Name</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="My S3 Backup"
              required
              className="w-full px-4 py-2 bg-gray-800 text-white rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          <div>
            <label className="block text-gray-400 mb-1">Endpoint</label>
            <input
              type="text"
              value={endpoint}
              onChange={(e) => setEndpoint(e.target.value)}
              placeholder="https://s3.amazonaws.com"
              required
              className="w-full px-4 py-2 bg-gray-800 text-white rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          <div>
            <label className="block text-gray-400 mb-1">Region</label>
            <input
              type="text"
              value={region}
              onChange={(e) => setRegion(e.target.value)}
              placeholder="us-east-1"
              className="w-full px-4 py-2 bg-gray-800 text-white rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          <div>
            <label className="block text-gray-400 mb-1">Bucket</label>
            <input
              type="text"
              value={bucket}
              onChange={(e) => setBucket(e.target.value)}
              placeholder="my-backups"
              required
              className="w-full px-4 py-2 bg-gray-800 text-white rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          <div>
            <label className="block text-gray-400 mb-1">Access Key ID</label>
            <input
              type="text"
              value={accessKey}
              onChange={(e) => setAccessKey(e.target.value)}
              placeholder="AKIAIOSFODNN7EXAMPLE"
              required
              className="w-full px-4 py-2 bg-gray-800 text-white rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          <div>
            <label className="block text-gray-400 mb-1">
              Secret Access Key
              {editingId && <span className="text-gray-500 text-xs ml-2">(leave empty to keep existing)</span>}
            </label>
            <input
              type="password"
              value={secretKey}
              onChange={(e) => setSecretKey(e.target.value)}
              placeholder={editingId ? 'Leave empty to keep existing' : 'wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY'}
              required={!editingId}
              className="w-full px-4 py-2 bg-gray-800 text-white rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          <div className="flex items-center gap-2">
            <input
              type="checkbox"
              id="isDefault"
              checked={isDefault}
              onChange={(e) => setIsDefault(e.target.checked)}
              className="rounded bg-gray-800 border-gray-600"
            />
            <label htmlFor="isDefault" className="text-gray-400 text-sm">Set as default profile</label>
          </div>

          <div className="flex gap-3 pt-2">
            <button
              type="submit"
              disabled={saving}
              className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50 text-sm"
            >
              {saving ? 'Saving...' : (editingId ? 'Update Profile' : 'Create Profile')}
            </button>
            <button
              type="button"
              onClick={resetForm}
              className="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-500 text-sm"
            >
              Cancel
            </button>
          </div>
        </form>
      )}
    </div>
  )
}
