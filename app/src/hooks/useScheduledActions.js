import { useState, useEffect, useCallback } from 'react';
import { fetchApi } from '../api/client';

export function useScheduledActions(serverId) {
    const [schedules, setSchedules] = useState([]);
    const [loading, setLoading] = useState(true);
    const [saving, setSaving] = useState(false);

    const refresh = useCallback(async () => {
        try {
            const data = await fetchApi(`/servers/${serverId}/tasks`);
            setSchedules(data || []);
        } catch (err) {
            console.error('Failed to fetch schedules:', err);
        } finally {
            setLoading(false);
        }
    }, [serverId]);

    useEffect(() => { refresh(); }, [refresh]);

    const createSchedule = useCallback(async (data) => {
        try {
            setSaving(true);
            await fetchApi(`/servers/${serverId}/tasks`, {
                method: 'POST',
                body: JSON.stringify(data),
            });
            await refresh();
        } catch (err) { throw err; }
        finally { setSaving(false); }
    }, [serverId, refresh]);

    const updateSchedule = useCallback(async (taskId, data) => {
        try {
            setSaving(true);
            await fetchApi(`/servers/${serverId}/tasks/${taskId}`, {
                method: 'PATCH',
                body: JSON.stringify(data),
            });
            await refresh();
        } catch (err) { throw err; }
        finally { setSaving(false); }
    }, [serverId, refresh]);

    const toggleSchedule = useCallback(async (taskId, enabled) => {
        // Optimistic UI update (UI-SPEC: "Optimistic toggling")
        setSchedules(prev => prev.map(s =>
            s.id === taskId ? { ...s, enabled } : s
        ));
        try {
            await fetchApi(`/servers/${serverId}/tasks/${taskId}`, {
                method: 'PATCH',
                body: JSON.stringify({ enabled }),
            });
        } catch (err) {
            // Rollback on error
            setSchedules(prev => prev.map(s =>
                s.id === taskId ? { ...s, enabled: !enabled } : s
            ));
            throw err;
        }
    }, [serverId]);

    const deleteSchedule = useCallback(async (taskId) => {
        await fetchApi(`/servers/${serverId}/tasks/${taskId}`, { method: 'DELETE' });
        setSchedules(prev => prev.filter(s => s.id !== taskId));
    }, [serverId]);

    return { schedules, loading, saving, createSchedule, updateSchedule, toggleSchedule, deleteSchedule, refresh };
}
