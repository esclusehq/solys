import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { templatesApi } from '../../api/templatesApi'
import { useUIStore } from '../../store/uiStore'

export default function TemplateCreatePage() {
  const { addToast } = useUIStore()
  const navigate = useNavigate()
  const [saving, setSaving] = useState(false)
  const [form, setForm] = useState({
    game_type: 'minecraft',
    category: 'vanilla',
    display_name: '',
    description: '',
    visibility: 'private',
    config: {
      docker_image: 'itzg/minecraft-server:latest',
      default_port: 25565,
      env: { MAX_PLAYERS: '20' },
    },
  })

  const updateField = (field, value) => {
    setForm(prev => ({ ...prev, [field]: value }))
  }

  const updateConfig = (key, value) => {
    setForm(prev => ({
      ...prev,
      config: { ...prev.config, [key]: value },
    }))
  }

  const updateEnv = (key, value) => {
    setForm(prev => ({
      ...prev,
      config: { ...prev.config, env: { ...prev.config.env, [key]: value } },
    }))
  }

  const handleSave = async (e) => {
    e.preventDefault()
    setSaving(true)
    try {
      await templatesApi.create(form)
      addToast({ type: 'success', message: 'Template created!' })
      navigate('/templates')
    } catch (err) {
      addToast({ type: 'error', message: err.message })
    } finally {
      setSaving(false)
    }
  }

  return (
    <div className="p-6 max-w-3xl mx-auto">
      <div className="flex items-center gap-3 mb-6">
        <button onClick={() => navigate('/templates')}
                className="text-gray-400 hover:text-white text-sm">&larr; Back</button>
        <h2 className="text-2xl font-bold text-white">Create Template</h2>
      </div>

      <form onSubmit={handleSave}>
        {/* Basic Info Section */}
        <section className="glass-panel p-6">
          <h3 className="text-lg font-bold mb-1">Basic Info</h3>
          <p className="text-xs text-[var(--color-text-muted)] mb-5">
            Configure the template name, game type, and category.
          </p>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-bold text-[var(--color-text-muted)] mb-2">Display Name</label>
              <input type="text" value={form.display_name}
                     onChange={e => updateField('display_name', e.target.value)} required
                     className="w-full px-4 py-2.5 rounded-lg text-sm bg-[var(--color-cosmic-card)]/60
                                border border-[var(--color-cosmic-border)] text-white" />
            </div>

            <div>
              <label className="block text-xs font-bold text-[var(--color-text-muted)] mb-2">Game Type</label>
              <select value={form.game_type} onChange={e => updateField('game_type', e.target.value)}
                      className="w-full px-4 py-2.5 rounded-lg text-sm bg-[var(--color-cosmic-card)]/60
                                 border border-[var(--color-cosmic-border)] text-white">
                <option value="minecraft">Minecraft</option>
                <option value="palworld">Palworld</option>
                <option value="rust">Rust</option>
                <option value="valheim">Valheim</option>
                <option value="bedrock">Minecraft Bedrock</option>
              </select>
            </div>

            <div>
              <label className="block text-xs font-bold text-[var(--color-text-muted)] mb-2">Category</label>
              <input type="text" value={form.category}
                     onChange={e => updateField('category', e.target.value)} required
                     placeholder="e.g. vanilla, paper, forge"
                     className="w-full px-4 py-2.5 rounded-lg text-sm bg-[var(--color-cosmic-card)]/60
                                border border-[var(--color-cosmic-border)] text-white" />
            </div>

            <div>
              <label className="block text-xs font-bold text-[var(--color-text-muted)] mb-2">Visibility</label>
              <select value={form.visibility} onChange={e => updateField('visibility', e.target.value)}
                      className="w-full px-4 py-2.5 rounded-lg text-sm bg-[var(--color-cosmic-card)]/60
                                 border border-[var(--color-cosmic-border)] text-white">
                <option value="private">Private</option>
                <option value="public">Public</option>
              </select>
            </div>
          </div>

          <div className="mt-4">
            <label className="block text-xs font-bold text-[var(--color-text-muted)] mb-2">Description</label>
            <textarea value={form.description} onChange={e => updateField('description', e.target.value)}
                      rows={3} placeholder="Describe what this template includes..."
                      className="w-full px-4 py-2.5 rounded-lg text-sm bg-[var(--color-cosmic-card)]/60
                                 border border-[var(--color-cosmic-border)] text-white" />
          </div>
        </section>

        {/* Configuration Section */}
        <section className="glass-panel p-6 mt-6">
          <h3 className="text-lg font-bold mb-1">Configuration</h3>
          <p className="text-xs text-[var(--color-text-muted)] mb-5">
            Docker image, port, and environment variables for the server.
          </p>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-bold text-[var(--color-text-muted)] mb-2">Docker Image</label>
              <input type="text" value={form.config.docker_image}
                     onChange={e => updateConfig('docker_image', e.target.value)} required
                     placeholder="e.g. itzg/minecraft-server:latest"
                     className="w-full px-4 py-2.5 rounded-lg text-sm bg-[var(--color-cosmic-card)]/60
                                border border-[var(--color-cosmic-border)] text-white" />
            </div>

            <div>
              <label className="block text-xs font-bold text-[var(--color-text-muted)] mb-2">Default Port</label>
              <input type="number" value={form.config.default_port}
                     onChange={e => updateConfig('default_port', parseInt(e.target.value) || 25565)}
                     className="w-full px-4 py-2.5 rounded-lg text-sm bg-[var(--color-cosmic-card)]/60
                                border border-[var(--color-cosmic-border)] text-white" />
            </div>
          </div>

          <div className="mt-4">
            <label className="block text-xs font-bold text-[var(--color-text-muted)] mb-2">Environment Variables (JSON)</label>
            <textarea value={JSON.stringify(form.config.env, null, 2)}
                      onChange={e => {
                        try { updateConfig('env', JSON.parse(e.target.value)) }
                        catch { /* allow editing, parse on save */ }
                      }}
                      rows={4}
                      className="w-full px-4 py-2.5 rounded-lg text-sm bg-[var(--color-cosmic-card)]/60
                                 border border-[var(--color-cosmic-border)] text-white font-mono" />
          </div>
        </section>

        {/* Save Button */}
        <button disabled={saving} type="submit"
                className="mt-6 w-full py-2.5 rounded-lg text-sm font-bold
                           bg-[var(--color-cosmic-cyan)]/10 text-[var(--color-cosmic-cyan)]
                           hover:bg-[var(--color-cosmic-cyan)]/20 border border-[var(--color-cosmic-cyan)]/30
                           disabled:opacity-50 transition-all">
          {saving ? 'Saving...' : 'Create Template'}
        </button>
      </form>
    </div>
  )
}
