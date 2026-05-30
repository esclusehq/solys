import { useState, useEffect, useCallback } from 'react';
import { fetchApi } from '../api/client';

export function useBackupConfig(serverId) {
    const [config, setConfig] = useState({
        auto_backup_enabled: false,
        schedule_cron: '',
        backup_provider: 'local',
        s3_profile_id: null,
        max_retained_backups: 10,
        retention_daily: 7,
        retention_weekly: 4,
        retention_monthly: 3,
    });
    const [loading, setLoading] = useState(true);
    const [saving, setSaving] = useState(false);
    const [error, setError] = useState(null);

    const refresh = useCallback(async () => {
        setLoading(true);
        setError(null);
        try {
            const data = await fetchApi(`/servers/${serverId}/backup-config`);
            if (data) {
                setConfig(prev => ({
                    ...prev,
                    auto_backup_enabled: data.auto_backup_enabled ?? false,
                    schedule_cron: data.schedule_cron ?? '',
                    backup_provider: data.backup_provider ?? 'local',
                    s3_profile_id: data.s3_profile_id ?? null,
                    max_retained_backups: data.max_retained_backups ?? 10,
                    retention_daily: data.retention_daily ?? 7,
                    retention_weekly: data.retention_weekly ?? 4,
                    retention_monthly: data.retention_monthly ?? 3,
                }));
            }
        } catch (err) {
            setError(err);
            console.error('Failed to fetch backup config:', err);
        } finally {
            setLoading(false);
        }
    }, [serverId]);

    useEffect(() => {
        refresh();
    }, [refresh]);

    const saveConfig = useCallback(async (newConfig) => {
        setSaving(true);
        setError(null);
        try {
            const data = await fetchApi(`/servers/${serverId}/backup-config`, {
                method: 'PUT',
                body: JSON.stringify(newConfig),
            });
            if (data) {
                setConfig(prev => ({
                    ...prev,
                    ...data,
                }));
            }
            return true;
        } catch (err) {
            setError(err);
            throw err;
        } finally {
            setSaving(false);
        }
    }, [serverId]);

    return { config, loading, saving, error, saveConfig, refresh };
}
