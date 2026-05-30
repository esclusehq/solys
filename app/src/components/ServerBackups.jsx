import { useState } from 'react';
import { useBackups } from '../hooks/useBackups';
import ServerBackupConfig from './ServerBackupConfig';

function formatBytes(bytes) {
    if (!bytes || bytes === 0) return '—';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}

function formatDate(iso) {
    if (!iso) return '—';
    const d = new Date(iso);
    return d.toLocaleDateString('en-US', { day: 'numeric', month: 'short', year: 'numeric' }) + ', ' +
        d.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' });
}

function StatusBadge({ status }) {
    const map = {
        success: { bg: 'rgba(16,185,129,0.15)', border: 'rgba(16,185,129,0.4)', color: 'var(--color-cosmic-green)', icon: '✓', label: 'Success' },
        in_progress: { bg: 'rgba(245,158,11,0.15)', border: 'rgba(245,158,11,0.4)', color: 'var(--color-cosmic-orange)', icon: '⟳', label: 'Running' },
        failed: { bg: 'rgba(239,68,68,0.15)', border: 'rgba(239,68,68,0.4)', color: 'var(--color-cosmic-red)', icon: '✕', label: 'Failed' },
    };
    const s = map[status] || map.failed;
    return (
        <span
            className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[11px] font-bold border"
            style={{ background: s.bg, borderColor: s.border, color: s.color }}
        >
            <span className={status === 'in_progress' ? 'animate-spin inline-block' : ''}>{s.icon}</span>
            {s.label}
        </span>
    );
}

function ProviderBadge({ provider }) {
    const isLocal = provider === 'local';
    return (
        <span className={`inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-bold uppercase tracking-wider border
            ${isLocal
                ? 'bg-[rgba(99,102,241,0.1)] border-[rgba(99,102,241,0.3)] text-indigo-400'
                : 'bg-[rgba(245,158,11,0.1)] border-[rgba(245,158,11,0.3)] text-amber-400'
            }`}
        >
            {isLocal ? '💾' : '☁️'} {provider}
        </span>
    );
}

export default function ServerBackups({ serverId }) {
    const { backups, loading, triggering, restoring, triggerBackup, deleteBackup, restoreBackup } = useBackups(serverId);
    const [deletingId, setDeletingId] = useState(null);
    const [toast, setToast] = useState(null);

    const handleTrigger = async () => {
        try {
            await triggerBackup();
            setToast({ type: 'success', message: '🚀 Backup started! It will appear below shortly.' });
        } catch (err) {
            setToast({ type: 'error', message: `❌ ${err.message}` });
        }
        setTimeout(() => setToast(null), 5000);
    };

    const handleDelete = async (id) => {
        if (!confirm('Delete this backup permanently?')) return;
        try {
            setDeletingId(id);
            await deleteBackup(id);
            setToast({ type: 'success', message: '🗑️ Backup deleted.' });
        } catch (err) {
            setToast({ type: 'error', message: `❌ ${err.message}` });
        } finally {
            setDeletingId(null);
            setTimeout(() => setToast(null), 4000);
        }
    };

    const handleRestore = async (id) => {
        if (!confirm('⚠️ This will restore the server from this backup. The current server data will be replaced. Continue?')) return;
        try {
            await restoreBackup(id);
            setToast({ type: 'success', message: '♻️ Restore started! Server will restart with restored data.' });
        } catch (err) {
            setToast({ type: 'error', message: `❌ ${err.message}` });
        }
        setTimeout(() => setToast(null), 5000);
    };

    if (loading) {
        return (
            <div className="flex items-center justify-center py-20 text-[var(--color-text-muted)]">
                <span className="animate-spin mr-2">⟳</span> Loading backups...
            </div>
        );
    }

    return (
        <>
            <ServerBackupConfig serverId={serverId} />
            <div className="max-w-5xl">
            {/* Header */}
            <div className="flex items-center justify-between mb-6">
                <div>
                    <h3 className="text-lg font-bold flex items-center gap-2">
                        🗄️ Backup History
                    </h3>
                    <p className="text-xs text-[var(--color-text-muted)] mt-1">
                        {backups.length} backup{backups.length !== 1 ? 's' : ''} on record
                    </p>
                </div>
                <button
                    onClick={handleTrigger}
                    disabled={triggering}
                    className="px-5 py-2.5 rounded-xl text-sm font-bold flex items-center gap-2 transition-all border
                               bg-[var(--color-cosmic-cyan)]/10 text-[var(--color-cosmic-cyan)]
                               hover:bg-[var(--color-cosmic-cyan)]/20 border-[var(--color-cosmic-cyan)]/30
                               disabled:opacity-50"
                >
                    {triggering ? (
                        <><span className="animate-spin">⟳</span> Starting...</>
                    ) : (
                        <>📦 Backup Now</>
                    )}
                </button>
            </div>

            {/* Toast */}
            {toast && (
                <div className={`mb-4 px-4 py-3 rounded-lg text-sm font-medium border ${toast.type === 'success'
                    ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-400'
                    : 'bg-red-500/10 border-red-500/30 text-red-400'
                    }`}>
                    {toast.message}
                </div>
            )}

            {/* Table */}
            {backups.length === 0 ? (
                <div className="glass-panel p-12 text-center">
                    <div className="text-4xl mb-3">📦</div>
                    <p className="text-[var(--color-text-muted)] text-sm">No backups yet.</p>
                    <p className="text-[var(--color-text-muted)] text-xs mt-1">Click <strong>"Backup Now"</strong> to create your first backup.</p>
                </div>
            ) : (
                <div className="glass-panel overflow-hidden">
                    <table className="w-full">
                        <thead>
                            <tr className="border-b border-[var(--color-cosmic-border)]">
                                <th className="text-left text-[10px] font-bold uppercase tracking-widest text-[var(--color-text-muted)] px-5 py-3">File</th>
                                <th className="text-left text-[10px] font-bold uppercase tracking-widest text-[var(--color-text-muted)] px-5 py-3">Provider</th>
                                <th className="text-left text-[10px] font-bold uppercase tracking-widest text-[var(--color-text-muted)] px-5 py-3">Date</th>
                                <th className="text-left text-[10px] font-bold uppercase tracking-widest text-[var(--color-text-muted)] px-5 py-3">Size</th>
                                <th className="text-left text-[10px] font-bold uppercase tracking-widest text-[var(--color-text-muted)] px-5 py-3">Status</th>
                                <th className="text-right text-[10px] font-bold uppercase tracking-widest text-[var(--color-text-muted)] px-5 py-3">Actions</th>
                            </tr>
                        </thead>
                        <tbody>
                            {backups.map(b => (
                                <tr key={b.id}
                                    className="border-b border-[var(--color-cosmic-border)] last:border-b-0 hover:bg-[rgba(255,255,255,0.02)] transition-colors">
                                    <td className="px-5 py-3.5">
                                        <span className="text-sm font-mono text-[var(--color-text-main)]">{b.file_name}</span>
                                    </td>
                                    <td className="px-5 py-3.5">
                                        <ProviderBadge provider={b.provider} />
                                    </td>
                                    <td className="px-5 py-3.5 text-xs text-[var(--color-text-muted)]">
                                        {formatDate(b.created_at)}
                                    </td>
                                    <td className="px-5 py-3.5 text-xs font-mono text-[var(--color-text-muted)]">
                                        {formatBytes(b.size_bytes)}
                                    </td>
                                    <td className="px-5 py-3.5">
                                        <StatusBadge status={b.status} />
                                    </td>
                                    <td className="px-5 py-3.5 text-right">
                                        <div className="flex items-center justify-end gap-2">
                                            <button
                                                onClick={() => handleRestore(b.id)}
                                                disabled={restoring === b.id || b.status === 'in_progress'}
                                                className="px-3 py-1.5 rounded-lg text-xs font-bold
                                                           text-cyan-400 hover:bg-cyan-500/10 border border-cyan-500/20
                                                           hover:border-cyan-500/40 transition-all disabled:opacity-30"
                                                title="Restore from this backup"
                                            >
                                                {restoring === b.id ? '⟳' : '♻️'}
                                            </button>
                                            <button
                                                onClick={() => handleDelete(b.id)}
                                                disabled={deletingId === b.id || b.status === 'in_progress'}
                                                className="px-3 py-1.5 rounded-lg text-xs font-bold
                                                           text-red-400 hover:bg-red-500/10 border border-red-500/20
                                                           hover:border-red-500/40 transition-all disabled:opacity-30"
                                            >
                                                {deletingId === b.id ? '...' : '🗑️'}
                                            </button>
                                        </div>
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            )}
        </div>
        </>
    );
}
