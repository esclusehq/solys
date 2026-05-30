import { useState, useEffect, useMemo } from 'react';
import { useBackupConfig } from '../hooks/useBackupConfig';
import { fetchApi } from '../api/client';
import { useUIStore } from '../store/uiStore';
import { useNavigate } from 'react-router-dom';

const SCHEDULE_PRESETS = [
  { value: '0 */6 * * *', label: 'Every 6 hours' },
  { value: '0 */12 * * *', label: 'Every 12 hours' },
  { value: '0 0 * * *', label: 'Daily (midnight)' },
  { value: '0 0 * * 0', label: 'Weekly (Sunday)' },
  { value: '0 0 1 * *', label: 'Monthly (1st)' },
  { value: '__custom__', label: 'Custom...' },
];

const RETENTION_PRESETS = [
  { value: 'none', label: 'None (count only)', daily: 0, weekly: 0, monthly: 0 },
  { value: 'moderate', label: 'Keep 7 daily, 4 weekly, 3 monthly', daily: 7, weekly: 4, monthly: 3 },
  { value: 'balanced', label: 'Keep 14 daily, 6 weekly, 6 monthly', daily: 14, weekly: 6, monthly: 6 },
  { value: 'aggressive', label: 'Keep 30 daily, 8 weekly, 12 monthly', daily: 30, weekly: 8, monthly: 12 },
];

function derivePresetFromCron(cron) {
  const match = SCHEDULE_PRESETS.find(p => p.value === cron);
  return match ? match.value : '__custom__';
}

function deriveRetentionPreset(daily, weekly, monthly) {
  const match = RETENTION_PRESETS.find(
    p => p.daily === daily && p.weekly === weekly && p.monthly === monthly
  );
  return match ? match.value : 'none';
}

export default function ServerBackupConfig({ serverId, freePlan = false }) {
  const { addToast } = useUIStore();
  const navigate = useNavigate();
  const { config, loading, saving, saveConfig } = useBackupConfig(serverId);

  const [autoBackupEnabled, setAutoBackupEnabled] = useState(false);
  const [scheduleCron, setScheduleCron] = useState('0 0 * * *');
  const [selectedPreset, setSelectedPreset] = useState('__custom__');
  const [customCron, setCustomCron] = useState('');
  const [backupProvider, setBackupProvider] = useState('local');
  const [s3ProfileId, setS3ProfileId] = useState(null);
  const [maxRetained, setMaxRetained] = useState(10);
  const [retentionPreset, setRetentionPreset] = useState('moderate');
  const [s3Profiles, setS3Profiles] = useState([]);
  const [s3ProfilesLoading, setS3ProfilesLoading] = useState(false);

  useEffect(() => {
    if (!loading && config) {
      setAutoBackupEnabled(config.auto_backup_enabled);
      setScheduleCron(config.schedule_cron || '0 0 * * *');
      setSelectedPreset(derivePresetFromCron(config.schedule_cron));
      setCustomCron(config.schedule_cron);
      setBackupProvider(config.backup_provider || 'local');
      setS3ProfileId(config.s3_profile_id || null);
      setMaxRetained(config.max_retained_backups ?? 10);
      setRetentionPreset(
        deriveRetentionPreset(
          config.retention_daily ?? 7,
          config.retention_weekly ?? 4,
          config.retention_monthly ?? 3
        )
      );
    }
  }, [loading, config]);

  useEffect(() => {
    if (backupProvider === 's3') {
      setS3ProfilesLoading(true);
      fetchApi('/settings/s3/profiles')
        .then(data => setS3Profiles(data || []))
        .catch(() => setS3Profiles([]))
        .finally(() => setS3ProfilesLoading(false));
    }
  }, [backupProvider]);

  const deriveCron = () => {
    if (selectedPreset === '__custom__') return customCron;
    const preset = SCHEDULE_PRESETS.find(p => p.value === selectedPreset);
    return preset ? preset.value : '0 0 * * *';
  };

  const handleSave = async () => {
    const retention = RETENTION_PRESETS.find(p => p.value === retentionPreset) || RETENTION_PRESETS[0];
    const payload = {
      auto_backup_enabled: autoBackupEnabled,
      schedule_cron: deriveCron(),
      backup_provider: backupProvider,
      s3_profile_id: backupProvider === 's3' ? s3ProfileId : null,
      max_retained_backups: maxRetained,
      retention_daily: retention.daily,
      retention_weekly: retention.weekly,
      retention_monthly: retention.monthly,
    };
    try {
      await saveConfig(payload);
      addToast({ type: 'success', message: 'Backup configuration saved' });
    } catch (err) {
      addToast({ type: 'error', message: `Could not save backup configuration. ${err.message}` });
    }
  };

  if (loading) {
    return (
      <div className="glass-panel p-6 mb-8 space-y-5">
        <div className="skeleton-pulse h-5 w-64 rounded-lg bg-[var(--color-cosmic-card)]/40" />
        <div className="skeleton-pulse h-4 w-96 rounded-lg bg-[var(--color-cosmic-card)]/40" />
        <div className="skeleton-pulse h-[42px] w-full rounded-lg bg-[var(--color-cosmic-card)]/40" />
        <div className="skeleton-pulse h-[42px] w-full rounded-lg bg-[var(--color-cosmic-card)]/40" />
        <div className="skeleton-pulse h-[42px] w-full rounded-lg bg-[var(--color-cosmic-card)]/40" />
        <div className="skeleton-pulse h-[42px] w-full rounded-lg bg-[var(--color-cosmic-card)]/40" />
      </div>
    );
  }

  return (
    <div className="glass-panel p-6 mb-8">
      {freePlan && (
        <div className="bg-yellow-500/10 border border-yellow-500/30 rounded-lg p-4 mb-6">
          <div className="flex items-start gap-3">
            <span className="text-2xl">🔒</span>
            <div className="flex-1">
              <h4 className="text-sm font-semibold text-yellow-400">Scheduled Backups Require Upgrade</h4>
              <p className="text-xs text-[var(--color-text-muted)] mt-1">
                Scheduled backups are only available for Hobby and Pro plans. Upgrade to enable automatic backup scheduling.
              </p>
              <button
                onClick={() => navigate('/billing')}
                className="mt-3 px-4 py-2 bg-gradient-to-r from-yellow-500 to-orange-500 text-black text-xs font-semibold rounded-lg hover:opacity-90 transition-all"
              >
                Upgrade Now →
              </button>
            </div>
          </div>
        </div>
      )}

      <h3 className="text-lg font-bold mb-1">Automated Backup Configuration</h3>
      <p className="text-xs text-[var(--color-text-muted)] mb-6">
        Configure automatic server backups to local storage or S3-compatible cloud.
      </p>

      <div className="flex items-center justify-between mb-6">
        <div>
          <p className="text-sm font-semibold text-[var(--color-text-main)]">Enable Auto Backup</p>
          <p className="text-[10px] text-[var(--color-text-muted)] mt-0.5">Run backups on a schedule automatically.</p>
        </div>
        <button
          onClick={() => setAutoBackupEnabled(!autoBackupEnabled)}
          className={`w-12 h-6 rounded-full transition-all flex items-center px-0.5 ${
            autoBackupEnabled
              ? 'bg-[var(--color-cosmic-cyan)]'
              : 'bg-[var(--color-cosmic-border)]'
          }`}
        >
          <div className={`w-5 h-5 rounded-full bg-white shadow transition-transform ${
            autoBackupEnabled ? 'translate-x-6' : 'translate-x-0'
          }`} />
        </button>
      </div>

      <div className={`space-y-5 transition-opacity ${!autoBackupEnabled ? 'opacity-40 pointer-events-none' : ''}`}>
        {/* Schedule Preset */}
        <div>
          <label className="text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider mb-1 block">
            Schedule
          </label>
          <select
            value={selectedPreset}
            onChange={(e) => {
              const val = e.target.value;
              setSelectedPreset(val);
              if (val !== '__custom__') {
                setScheduleCron(val);
              }
            }}
            className="w-full px-4 py-2.5 rounded-lg text-sm bg-[var(--color-cosmic-card)]/60 border border-[var(--color-cosmic-border)] text-[var(--color-text-main)] focus:outline-none focus:border-[var(--color-cosmic-cyan)] transition-all"
          >
            {SCHEDULE_PRESETS.map(p => (
              <option key={p.value} value={p.value}>{p.label}</option>
            ))}
          </select>
        </div>

        {/* Custom Cron Input */}
        {selectedPreset === '__custom__' && (
          <div>
            <label className="text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider mb-1 block">
              Cron Expression
            </label>
            <input
              type="text"
              value={customCron}
              onChange={(e) => { setCustomCron(e.target.value); setScheduleCron(e.target.value); }}
              placeholder="0 */6 * * *"
              className="w-full px-4 py-2.5 rounded-lg text-sm font-mono bg-[var(--color-cosmic-card)]/60 border border-[var(--color-cosmic-border)] text-[var(--color-text-main)] placeholder:text-[var(--color-text-muted)] focus:outline-none focus:border-[var(--color-cosmic-cyan)] transition-all"
            />
          </div>
        )}

        {/* Max Backup Count */}
        <div>
          <label className="text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider mb-1 block">
            Max Backup Count
          </label>
          <input
            type="number"
            min="1"
            max="100"
            value={maxRetained}
            onChange={(e) => setMaxRetained(parseInt(e.target.value) || 10)}
            className="w-full px-4 py-2.5 rounded-lg text-sm bg-[var(--color-cosmic-card)]/60 border border-[var(--color-cosmic-border)] text-[var(--color-text-main)] focus:outline-none focus:border-[var(--color-cosmic-cyan)] transition-all"
          />
          <p className="text-[10px] text-[var(--color-text-muted)] mt-1">
            Oldest backups are pruned when this limit is exceeded.
          </p>
        </div>

        {/* Retention Schedule */}
        <div>
          <label className="text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider mb-1 block">
            Retention Schedule
          </label>
          <select
            value={retentionPreset}
            onChange={(e) => setRetentionPreset(e.target.value)}
            className="w-full px-4 py-2.5 rounded-lg text-sm bg-[var(--color-cosmic-card)]/60 border border-[var(--color-cosmic-border)] text-[var(--color-text-main)] focus:outline-none focus:border-[var(--color-cosmic-cyan)] transition-all"
          >
            {RETENTION_PRESETS.map(p => (
              <option key={p.value} value={p.value}>{p.label}</option>
            ))}
          </select>
          <p className="text-[10px] text-[var(--color-text-muted)] mt-1">
            Keep labeled backups beyond the max count limit.
          </p>
        </div>

        {/* Storage Provider */}
        <div>
          <label className="text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider mb-1 block">
            Storage Provider
          </label>
          <select
            value={backupProvider}
            onChange={(e) => setBackupProvider(e.target.value)}
            className="w-full px-4 py-2.5 rounded-lg text-sm bg-[var(--color-cosmic-card)]/60 border border-[var(--color-cosmic-border)] text-[var(--color-text-main)] focus:outline-none focus:border-[var(--color-cosmic-cyan)] transition-all"
          >
            <option value="local">💾 Local Storage</option>
            <option value="s3">☁️ S3-Compatible</option>
          </select>
        </div>

        {/* S3 Profile */}
        {backupProvider === 's3' && (
          <div>
            <label className="text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider mb-1 block">
              S3 Profile
            </label>
            {s3ProfilesLoading ? (
              <div className="skeleton-pulse h-[42px] rounded-lg bg-[var(--color-cosmic-card)]/40" />
            ) : s3Profiles.length === 0 ? (
              <select
                disabled
                className="w-full px-4 py-2.5 rounded-lg text-sm bg-[var(--color-cosmic-card)]/60 border border-[var(--color-cosmic-border)] text-[var(--color-text-muted)] opacity-50 cursor-not-allowed"
              >
                <option>No S3 profiles configured</option>
              </select>
            ) : (
              <select
                value={s3ProfileId || ''}
                onChange={(e) => setS3ProfileId(e.target.value || null)}
                className="w-full px-4 py-2.5 rounded-lg text-sm bg-[var(--color-cosmic-card)]/60 border border-[var(--color-cosmic-border)] text-[var(--color-text-main)] focus:outline-none focus:border-[var(--color-cosmic-cyan)] transition-all"
              >
                <option value="">Select a profile...</option>
                {s3Profiles.map(p => (
                  <option key={p.id} value={p.id}>
                    {p.name}{p.is_default ? ' (Default)' : ''}
                  </option>
                ))}
              </select>
            )}
            <p className="text-[10px] text-[var(--color-text-muted)] mt-1">
              Manage profiles in{' '}
              <a href="/settings?tab=storage" className="text-[var(--color-cosmic-cyan)] hover:underline">
                Settings → Storage
              </a>
            </p>
          </div>
        )}

        {/* Save Button */}
        <button
          onClick={handleSave}
          disabled={saving}
          className="w-full py-2.5 rounded-lg text-sm font-bold bg-[var(--color-cosmic-cyan)]/10 text-[var(--color-cosmic-cyan)] hover:bg-[var(--color-cosmic-cyan)]/20 border border-[var(--color-cosmic-cyan)]/30 disabled:opacity-50 transition-all"
        >
          {saving ? 'Saving...' : '💾 Save Changes'}
        </button>
      </div>
    </div>
  );
}
