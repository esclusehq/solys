import { useState, useCallback, useRef, useEffect, useMemo } from 'react';
import { useParams, Link, useNavigate } from 'react-router-dom';
import { fetchApi } from '../api/client';
import { startServer, stopServer, sendCommand, updateServer } from '../hooks/useServers';
import { useWebSocket } from '../hooks/useWebSocket';
import { useAlertHistory } from '../hooks/useAlerts';
import StatusBadge from '../components/StatusBadge';
import FileManager from '../components/FileManager';
import PluginManager from '../components/PluginManager';
import ServerBackups from '../components/ServerBackups';
import SourceControl from '../components/SourceControl';
import WebIDE from '../components/WebIDE';
import Terminal from '../components/Terminal';

const MAX_POINTS = 60;
const CHART_W = 800;
const CHART_H = 200;

// Convert data array to a smooth SVG path using monotone cubic interpolation
function buildPath(data, maxVal, close = false) {
    if (data.length === 0) return '';
    if (data.length === 1) {
        const y = CHART_H - (Math.min(data[0], maxVal) / maxVal) * (CHART_H - 10) - 5;
        const d = `M0,${y} L${CHART_W},${y}`;
        return close ? d + ` L${CHART_W},${CHART_H} L0,${CHART_H} Z` : d;
    }
    const pts = data.map((v, i) => ({
        x: (i / (MAX_POINTS - 1)) * CHART_W,
        y: CHART_H - (Math.min(v, maxVal) / maxVal) * (CHART_H - 10) - 5,
    }));
    let d = `M${pts[0].x},${pts[0].y}`;
    for (let i = 1; i < pts.length; i++) {
        const cp = (pts[i].x - pts[i - 1].x) / 3;
        d += ` C${pts[i - 1].x + cp},${pts[i - 1].y} ${pts[i].x - cp},${pts[i].y} ${pts[i].x},${pts[i].y}`;
    }
    if (close) d += ` L${pts[pts.length - 1].x},${CHART_H} L${pts[0].x},${CHART_H} Z`;
    return d;
}

export default function ServerDetails() {
    const { id } = useParams();
    const [server, setServer] = useState(null);
    const [metrics, setMetrics] = useState(null);
    const [loading, setLoading] = useState(true);
    const [logs, setLogs] = useState([]);
    const [cmdInput, setCmdInput] = useState('');
    const [tpsHistory, setTpsHistory] = useState([]);
    const [cpuHistory, setCpuHistory] = useState([]);
    const { history } = useAlertHistory(id);
    const logEndRef = useRef(null);
    const [activeTab, setActiveTab] = useState('dashboard');
    const [webhookUrl, setWebhookUrl] = useState('');
    const [webhookSaving, setWebhookSaving] = useState(false);
    const [webhookToast, setWebhookToast] = useState(null);

    // Backup config state
    const [backupConfig, setBackupConfig] = useState({
        auto_backup_enabled: false,
        backup_cron: '',
        backup_provider: 'local',
        backup_path: '',
        max_retained_backups: 5,
    });
    const [backupSaving, setBackupSaving] = useState(false);
    const [userPlan, setUserPlan] = useState('free');
    const [planLoading, setPlanLoading] = useState(true);
    const navigate = useNavigate();
    
    const canUseBackup = userPlan !== 'free';

    useEffect(() => {
        fetchApi(`/servers/${id}`).then(data => {
            setServer(data);
            setWebhookUrl(data.discord_webhook_url || '');
            setBackupConfig({
                auto_backup_enabled: data.auto_backup_enabled || false,
                backup_cron: data.backup_cron || '',
                backup_provider: data.backup_provider || 'local',
                backup_path: data.backup_path || '',
                max_retained_backups: data.max_retained_backups ?? 5,
            });
            setLoading(false);
        }).catch(() => setLoading(false));

        // Load user plan for backup access check
        fetchApi('/billing/subscription').then(data => {
            const planName = data?.plan?.name || 'free';
            setUserPlan(planName);
        }).catch(() => {
            setUserPlan('free');
        }).finally(() => {
            setPlanLoading(false);
        });

        // Fetch metrics history to seed the chart immediately
        fetchApi(`/servers/${id}/metrics-history`).then(historyData => {
            if (Array.isArray(historyData) && historyData.length > 0) {
                // API returns newest first, reverse for chart (oldest → newest)
                const ordered = [...historyData].reverse();
                setTpsHistory(ordered.map(m => m.tps ?? 0));
                setCpuHistory(ordered.map(m => m.cpu_usage ?? 0));
                // Set current metrics to the latest entry
                const latest = historyData[0];
                setMetrics({ tps: latest.tps, cpu_usage: latest.cpu_usage, memory_usage_mb: latest.memory_usage_mb, players: latest.players });
            }
        }).catch(() => { /* metrics history not available yet */ });
    }, [id]);

    const onWsMessage = useCallback((msg) => {
        if (msg.type === 'snapshot') {
            console.log('[WS] Received snapshot, status:', msg.data.status);
            setServer(prev => prev ? { ...prev, status: msg.data.status } : prev);
            if (msg.data.latest_metrics) {
                const m = msg.data.latest_metrics;
                setMetrics(m);
                if (m.tps != null) setTpsHistory(prev => prev.length === 0 ? [m.tps] : prev);
                if (m.cpu_usage != null) setCpuHistory(prev => prev.length === 0 ? [m.cpu_usage] : prev);
            }
        } else if (msg.type === 'event') {
            const { type, payload } = msg.data;
            console.log('[WS] Received event:', type, 'payload:', payload);
            if (type === 'StatusChanged') {
                console.log('[WS] StatusChanged:', payload.status);
                setServer(prev => prev ? { ...prev, status: payload.status } : prev);
            }
            if (type === 'MetricsUpdated') {
                setMetrics({ tps: payload.tps, cpu_usage: payload.cpu_usage, memory_usage_mb: payload.memory_usage_mb, players: payload.players });
                if (payload.tps != null) setTpsHistory(prev => [...prev.slice(-(MAX_POINTS - 1)), payload.tps]);
                if (payload.cpu_usage != null) setCpuHistory(prev => [...prev.slice(-(MAX_POINTS - 1)), payload.cpu_usage]);
            }
            if (type === 'LogLine') {
                setLogs(prev => [...prev.slice(-200), { time: new Date().toLocaleTimeString(), text: payload.line }]);
            }
        }
    }, []);

    const { connected } = useWebSocket([id], onWsMessage);

    useEffect(() => {
        logEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    }, [logs]);

    const handleToggle = async () => {
        console.log('[handleToggle] Current status:', server.status, 'Server ID:', id);
        try {
            if (server.status === 'running') {
                console.log('[handleToggle] Calling stopServer...');
                await stopServer(id);
            } else {
                console.log('[handleToggle] Calling startServer...');
                await startServer(id);
            }
        } catch (err) { alert(err.message); }
    };

    const handleCommand = async (e) => {
        e.preventDefault();
        if (!cmdInput.trim()) return;
        try {
            const result = await sendCommand(id, cmdInput);
            setLogs(prev => [...prev, { time: new Date().toLocaleTimeString(), text: `> ${cmdInput}`, isCmd: true }]);
            if (result?.output) setLogs(prev => [...prev, { time: new Date().toLocaleTimeString(), text: result.output }]);
            setCmdInput('');
        } catch (err) {
            setLogs(prev => [...prev, { time: new Date().toLocaleTimeString(), text: `Error: ${err.message}`, isError: true }]);
        }
    };

    const isTransitional = ['starting', 'container_running', 'stopping'].includes(server?.status);

    // Build chart paths from history
    const tpsLine = useMemo(() => buildPath(tpsHistory, 20), [tpsHistory]);
    const tpsFill = useMemo(() => buildPath(tpsHistory, 20, true), [tpsHistory]);
    const cpuLine = useMemo(() => buildPath(cpuHistory, 100), [cpuHistory]);
    const cpuFill = useMemo(() => buildPath(cpuHistory, 100, true), [cpuHistory]);

    // Time labels based on history length
    const timeLabels = useMemo(() => {
        const len = tpsHistory.length;
        if (len === 0) return ['—', '—', '—', '—', 'Now'];
        const secs = len * 5; // approximate 5s interval per point
        return [
            `-${Math.round(secs)}s`,
            `-${Math.round(secs * 0.75)}s`,
            `-${Math.round(secs * 0.5)}s`,
            `-${Math.round(secs * 0.25)}s`,
            'Now',
        ];
    }, [tpsHistory.length]);

    if (loading) return <div className="flex-1 flex items-center justify-center text-[var(--color-text-muted)]">Loading...</div>;
    if (!server) return <div className="flex-1 flex items-center justify-center text-[var(--color-text-muted)]">Server not found</div>;

    return (
        <div className="flex-1 overflow-y-auto">
            {/* ─── TOP BAR ─── */}
            <header className="sticky top-0 z-10 glass-panel border-b border-[var(--color-cosmic-border)] px-8 py-4 flex items-center justify-between" style={{ borderRadius: 0 }}>
                <div className="flex items-center gap-4">
                    <Link to="/servers"
                        className="w-10 h-10 rounded-xl bg-[rgba(255,255,255,0.03)] border border-[rgba(13,223,242,0.2)] flex items-center justify-center hover:bg-[rgba(13,223,242,0.1)] transition-colors text-[var(--color-cosmic-cyan)]">
                        ←
                    </Link>
                    <div>
                        <div className="flex items-center gap-2 text-xs text-[var(--color-text-muted)] uppercase tracking-widest font-semibold">
                            <span>Servers</span>
                            <span className="text-[10px]">›</span>
                            <span className="text-[var(--color-cosmic-cyan)]">{server.name}</span>
                        </div>
                        <h2 className="text-xl font-bold flex items-center gap-2">
                            {server.name}
                            <span className="text-[var(--color-text-muted)] font-normal text-base">({server.game || server.executor_type})</span>
                        </h2>
                    </div>
                </div>
                <div className="flex items-center gap-3">
                    <button
                        onClick={handleToggle}
                        disabled={isTransitional}
                        className={`px-5 py-2 rounded-xl font-bold text-sm flex items-center gap-2 transition-all border
                            ${isTransitional
                                ? 'border-[var(--color-text-muted)] text-[var(--color-text-muted)] opacity-50 cursor-not-allowed'
                                : server.status === 'running'
                                    ? 'border-[var(--color-cosmic-red)]/50 text-[var(--color-cosmic-red)] hover:bg-[rgba(239,68,68,0.1)]'
                                    : 'border-[var(--color-cosmic-cyan)] text-[var(--color-cosmic-cyan)] hover:bg-[rgba(13,223,242,0.1)]'
                            }`}
                    >
                        {isTransitional ? '⏳' : server.status === 'running' ? '■' : '▶'}
                        {isTransitional ? (server.status === 'container_running' ? 'Starting Minecraft...' : server.status === 'starting' ? 'Starting...' : 'Stopping...') : server.status === 'running' ? 'Stop' : 'Start'}
                    </button>
                    <Link to="/console"
                        className="px-5 py-2 rounded-xl bg-[rgba(255,255,255,0.03)] border border-[var(--color-cosmic-border)] text-[var(--color-text-main)] font-bold text-sm hover:border-[var(--color-cosmic-cyan)]/50 transition-all flex items-center gap-2">
                        ⌨ Open Console
                    </Link>
                    <span className={`w-2 h-2 rounded-full ml-1 ${connected ? 'bg-[var(--color-cosmic-green)] animate-pulse-glow' : 'bg-[var(--color-cosmic-red)]'}`} title={connected ? 'WebSocket Connected' : 'WebSocket Disconnected'} />
                </div>
            </header>

            {/* ─── TABS ─── */}
            <div className="px-8 mt-4 mb-2 flex items-center gap-4 border-b border-[var(--color-cosmic-border)] pb-0">
                <button
                    onClick={() => setActiveTab('dashboard')}
                    className={`px-4 py-3 font-bold text-sm transition-all border-b-2 ${activeTab === 'dashboard' ? 'border-[var(--color-cosmic-cyan)] text-[var(--color-cosmic-cyan)]' : 'border-transparent text-[var(--color-text-muted)] hover:text-[var(--color-text-main)]'}`}
                >
                    Dashboard
                </button>
                <button
                    onClick={() => setActiveTab('files')}
                    className={`px-4 py-3 font-bold text-sm transition-all border-b-2 ${activeTab === 'files' ? 'border-[var(--color-cosmic-cyan)] text-[var(--color-cosmic-cyan)]' : 'border-transparent text-[var(--color-text-muted)] hover:text-[var(--color-text-main)]'}`}
                >
                    File Manager
                </button>
                <button
                    onClick={() => setActiveTab('plugins')}
                    className={`px-4 py-3 font-bold text-sm transition-all border-b-2 ${activeTab === 'plugins' ? 'border-[var(--color-cosmic-cyan)] text-[var(--color-cosmic-cyan)]' : 'border-transparent text-[var(--color-text-muted)] hover:text-[var(--color-text-main)]'}`}
                >
                    {['PAPER', 'SPIGOT', 'BUKKIT', 'PURPUR', 'FORGE', 'FABRIC', 'NEOFORGE'].includes(server?.mc_loader?.toUpperCase()) ? '🧩 Plugins' : '📦 Datapacks'}
                </button>
                {server.environment !== 'development' && (
                    <button
                        onClick={() => setActiveTab('backups')}
                        className={`px-4 py-3 font-bold text-sm transition-all border-b-2 ${activeTab === 'backups' ? 'border-[var(--color-cosmic-cyan)] text-[var(--color-cosmic-cyan)]' : 'border-transparent text-[var(--color-text-muted)] hover:text-[var(--color-text-main)]'}`}
                    >
                        🗄️ Backups
                    </button>
                )}
                <button
                    onClick={() => setActiveTab('settings')}
                    className={`px-4 py-3 font-bold text-sm transition-all border-b-2 ${activeTab === 'settings' ? 'border-[var(--color-cosmic-cyan)] text-[var(--color-cosmic-cyan)]' : 'border-transparent text-[var(--color-text-muted)] hover:text-[var(--color-text-main)]'}`}
                >
                    ⚙️ Settings
                </button>
                {server.environment === 'development' && (
                    <button
                        onClick={() => setActiveTab('ide')}
                        className={`px-4 py-3 font-bold text-sm transition-all border-b-2 ${activeTab === 'ide' ? 'border-[var(--color-cosmic-magenta)] text-[var(--color-cosmic-magenta)]' : 'border-transparent text-[var(--color-text-muted)] hover:text-[var(--color-text-main)]'}`}
                    >
                        💻 IDE
                    </button>
                )}
                {server.environment === 'development' && (
                    <button
                        onClick={() => setActiveTab('source')}
                        className={`px-4 py-3 font-bold text-sm transition-all border-b-2 ${activeTab === 'source' ? 'border-[var(--color-cosmic-magenta)] text-[var(--color-cosmic-magenta)]' : 'border-transparent text-[var(--color-text-muted)] hover:text-[var(--color-text-main)]'}`}
                    >
                        🔗 Source Control
                    </button>
                )}
                <button
                    onClick={() => setActiveTab('terminal')}
                    className={`px-4 py-3 font-bold text-sm transition-all border-b-2 ${activeTab === 'terminal' ? 'border-[var(--color-cosmic-cyan)] text-[var(--color-cosmic-cyan)]' : 'border-transparent text-[var(--color-text-muted)] hover:text-[var(--color-text-main)]'}`}
                >
                    ⌨ Terminal
                </button>
            </div>

            {/* ─── CONTENT AREA ─── */}
            <div className="p-8 pt-4">
                {activeTab === 'ide' ? (
                    <div className="h-[75vh]">
                        {id ? <WebIDE serverId={id} /> : <div className="text-red-500">No server ID</div>}
                    </div>
                ) : activeTab === 'source' ? (
                    <div className="h-[75vh]">
                        <SourceControl serverId={id} />
                    </div>
                ) : activeTab === 'files' ? (
                    <div className="h-[75vh]">
                        <FileManager serverId={id} />
                    </div>
                ) : activeTab === 'plugins' ? (
                    <div className="h-[75vh]">
                        <PluginManager
                            serverId={id}
                            serverVersion={server?.mc_version}
                            serverLoader={server?.loader_type}
                            mode={['PAPER', 'SPIGOT', 'BUKKIT', 'PURPUR', 'FORGE', 'FABRIC', 'NEOFORGE'].includes(server?.mc_loader?.toUpperCase()) ? 'plugin' : 'datapack'}
                        />
                    </div>
                ) : activeTab === 'backups' ? (
                    <div className="min-h-[75vh]">
                        <ServerBackups serverId={id} />
                    </div>
                ) : activeTab === 'terminal' ? (
                    <div className="h-[75vh]">
                        <Terminal serverId={id} />
                    </div>
                ) : activeTab === 'settings' ? (
                    <div className="max-w-2xl">
                        <section className="glass-panel p-6">
                            <h3 className="text-lg font-bold mb-1">Discord Webhook</h3>
                            <p className="text-xs text-[var(--color-text-muted)] mb-5">
                                Receive alert notifications (Triggered / Recovered) directly in your Discord channel.
                            </p>

                            {webhookToast && (
                                <div className={`mb-4 px-4 py-3 rounded-lg text-sm font-medium border ${webhookToast.type === 'success'
                                    ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-400'
                                    : 'bg-red-500/10 border-red-500/30 text-red-400'
                                    }`}>
                                    {webhookToast.message}
                                </div>
                            )}

                            <div className="flex gap-3">
                                <input
                                    type="url"
                                    value={webhookUrl}
                                    onChange={(e) => setWebhookUrl(e.target.value)}
                                    placeholder="https://discord.com/api/webhooks/..."
                                    className="flex-1 px-4 py-2.5 rounded-lg text-sm
                                               bg-[var(--color-cosmic-card)]/60 border border-[var(--color-cosmic-border)]
                                               text-[var(--color-text-main)] placeholder:text-[var(--color-text-muted)]
                                               focus:outline-none focus:border-[var(--color-cosmic-cyan)] transition-all"
                                />
                                <button
                                    disabled={webhookSaving}
                                    onClick={async () => {
                                        try {
                                            setWebhookSaving(true);
                                            await updateServer(id, { discord_webhook_url: webhookUrl || '' });
                                            setServer(prev => ({ ...prev, discord_webhook_url: webhookUrl || null }));
                                            setWebhookToast({ type: 'success', message: '✅ Webhook URL saved!' });
                                        } catch (e) {
                                            setWebhookToast({ type: 'error', message: `❌ ${e.message}` });
                                        } finally {
                                            setWebhookSaving(false);
                                            setTimeout(() => setWebhookToast(null), 4000);
                                        }
                                    }}
                                    className="px-5 py-2.5 rounded-lg text-sm font-bold
                                               bg-[var(--color-cosmic-cyan)]/10 text-[var(--color-cosmic-cyan)]
                                               hover:bg-[var(--color-cosmic-cyan)]/20 border border-[var(--color-cosmic-cyan)]/30
                                               disabled:opacity-50 transition-all"
                                >
                                    {webhookSaving ? 'Saving...' : 'Save'}
                                </button>
                            </div>
                            <p className="text-[10px] text-[var(--color-text-muted)] mt-3">
                                💡 Use Discord's <strong>Server Settings → Integrations → Webhooks</strong> to create a webhook URL.
                                Leave empty and save to disable notifications.
                            </p>
                        </section>


                    </div>
                ) : (
                    <div className="grid grid-cols-12 gap-8">
                        {/* ─── LEFT COLUMN (65%) ─── */}
                        <div className="col-span-12 lg:col-span-8 flex flex-col gap-8">

                            {/* Performance Chart Card */}
                            <section className="glass-panel p-6">
                                <div className="flex items-center justify-between mb-8">
                                    <div>
                                        <h3 className="text-lg font-bold">Performance (TPS & Players)</h3>
                                        <div className="flex items-center gap-4 mt-1">
                                            <div className="flex items-center gap-2">
                                                <span className="w-2 h-2 rounded-full bg-[var(--color-cosmic-cyan)]" />
                                                <span className="text-xs text-[var(--color-text-muted)] font-medium uppercase">TPS: {metrics?.tps?.toFixed(1) ?? '—'}</span>
                                            </div>
                                            <div className="flex items-center gap-2">
                                                <span className="w-2 h-2 rounded-full bg-[var(--color-cosmic-purple)]" />
                                                <span className="text-xs text-[var(--color-text-muted)] font-medium uppercase">Players: {metrics?.players ?? '—'}</span>
                                            </div>
                                        </div>
                                    </div>
                                    <div className="text-right">
                                        <p className={`text-3xl font-bold tracking-tight ${metrics?.tps && metrics.tps < 18 ? 'text-[var(--color-cosmic-orange)]' : 'text-[var(--color-cosmic-cyan)]'}`}>
                                            {metrics?.tps?.toFixed(1) ?? '—'} TPS
                                        </p>
                                    </div>
                                </div>
                                {/* Real-Time SVG Chart */}
                                <div className="relative h-[250px] w-full">
                                    {tpsHistory.length === 0 ? (
                                        <div className="absolute inset-0 flex items-center justify-center text-[var(--color-text-muted)] text-sm italic">
                                            Waiting for metrics data...
                                        </div>
                                    ) : (
                                        <svg className="w-full h-full" viewBox={`0 0 ${CHART_W} ${CHART_H}`} preserveAspectRatio="none">
                                            <defs>
                                                <linearGradient id="cyan-gradient" x1="0" x2="0" y1="0" y2="1">
                                                    <stop offset="0%" stopColor="var(--color-cosmic-cyan)" stopOpacity="0.3" />
                                                    <stop offset="100%" stopColor="var(--color-cosmic-cyan)" stopOpacity="0" />
                                                </linearGradient>
                                                <linearGradient id="purple-gradient" x1="0" x2="0" y1="0" y2="1">
                                                    <stop offset="0%" stopColor="var(--color-cosmic-purple)" stopOpacity="0.2" />
                                                    <stop offset="100%" stopColor="var(--color-cosmic-purple)" stopOpacity="0" />
                                                </linearGradient>
                                                <filter id="glow-cyan">
                                                    <feGaussianBlur stdDeviation="3" result="blur" />
                                                    <feMerge><feMergeNode in="blur" /><feMergeNode in="SourceGraphic" /></feMerge>
                                                </filter>
                                            </defs>
                                            {/* Horizontal grid lines */}
                                            {[0.25, 0.5, 0.75].map(f => (
                                                <line key={f} x1="0" x2={CHART_W} y1={CHART_H * f} y2={CHART_H * f}
                                                    stroke="rgba(255,255,255,0.04)" strokeWidth="1" />
                                            ))}
                                            {/* CPU / Purple area */}
                                            {cpuHistory.length >= 2 && (
                                                <>
                                                    <path d={cpuFill} fill="url(#purple-gradient)" style={{ transition: 'all 0.4s ease' }} />
                                                    <path d={cpuLine} fill="none" stroke="var(--color-cosmic-purple)" strokeWidth="2"
                                                        style={{ transition: 'all 0.4s ease' }} />
                                                </>
                                            )}
                                            {/* TPS / Cyan area */}
                                            <path d={tpsFill} fill="url(#cyan-gradient)" style={{ transition: 'all 0.4s ease' }} />
                                            <path d={tpsLine} fill="none" stroke="var(--color-cosmic-cyan)" strokeWidth="3"
                                                filter="url(#glow-cyan)" style={{ transition: 'all 0.4s ease' }} />
                                            {/* Latest value dot */}
                                            {tpsHistory.length > 0 && (
                                                <circle
                                                    cx={((tpsHistory.length - 1) / (MAX_POINTS - 1)) * CHART_W}
                                                    cy={CHART_H - (Math.min(tpsHistory[tpsHistory.length - 1], 20) / 20) * (CHART_H - 10) - 5}
                                                    r="4" fill="var(--color-cosmic-cyan)"
                                                    style={{ filter: 'drop-shadow(0 0 6px rgba(13,223,242,0.8))', transition: 'all 0.4s ease' }}
                                                />
                                            )}
                                        </svg>
                                    )}
                                    <div className="flex justify-between mt-4 px-2 border-t border-[var(--color-cosmic-border)] pt-4">
                                        {timeLabels.map((t, i) => (
                                            <span key={i} className="text-[10px] font-bold text-[var(--color-text-muted)] uppercase">{t}</span>
                                        ))}
                                    </div>
                                </div>
                            </section>

                            {/* Server Information Card */}
                            <section className="glass-panel p-6">
                                <div className="flex items-center justify-between mb-6">
                                    <h3 className="text-lg font-bold flex items-center gap-2">
                                        Server Information
                                        <StatusBadge status={server.status} />
                                    </h3>
                                    <button
                                        onClick={handleToggle}
                                        disabled={isTransitional}
                                        className="text-[var(--color-cosmic-cyan)] text-xs font-bold flex items-center gap-1 hover:underline disabled:opacity-50">
                                        ↻ Restart Server
                                    </button>
                                </div>
                                <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                                    {[
                                        { label: 'IP Address', value: server.host || '—', mono: true },
                                        { label: 'Port', value: server.port || '—', mono: true },
                                        { label: 'Memory Usage', value: metrics ? (metrics.memory_usage_mb >= 1024 ? `${(metrics.memory_usage_mb / 1024).toFixed(1)} GB` : `${metrics.memory_usage_mb} MB`) : '—', mono: true },
                                        { label: 'CPU Load', value: metrics ? `${metrics.cpu_usage?.toFixed(1)}%` : '—', mono: true },
                                    ].map(({ label, value, mono }) => (
                                        <div key={label} className="p-4 rounded-xl bg-[var(--color-deep-space)] border border-[var(--color-cosmic-border)]">
                                            <p className="text-[10px] font-bold text-[var(--color-text-muted)] uppercase mb-1">{label}</p>
                                            <p className={`text-sm text-[var(--color-text-main)] ${mono ? 'font-mono' : ''}`}>{value}</p>
                                        </div>
                                    ))}
                                </div>
                                {/* Extra info row */}
                                <div className="mt-6 flex items-center gap-4">
                                    <div className="flex-1 h-2 bg-[var(--color-cosmic-border)] rounded-full overflow-hidden">
                                        <div
                                            className="h-full bg-[var(--color-cosmic-cyan)] rounded-full transition-all duration-500"
                                            style={{ width: metrics ? `${Math.min(metrics.cpu_usage || 0, 100)}%` : '0%', boxShadow: '0 0 10px rgba(13,223,242,0.4)' }}
                                        />
                                    </div>
                                    <span className="text-xs text-[var(--color-text-muted)] font-medium">
                                        Executor: <span className="text-[var(--color-cosmic-cyan)] font-bold">{server.executor_type?.toUpperCase()}</span>
                                    </span>
                                </div>
                            </section>
                        </div>

                        {/* ─── RIGHT COLUMN (35%) ─── */}
                        <div className="col-span-12 lg:col-span-4 flex flex-col gap-8">

                            {/* Live Console Preview */}
                            <section className="glass-panel overflow-hidden flex flex-col" style={{ minHeight: '400px' }}>
                                <div className="px-5 py-4 border-b border-[var(--color-cosmic-border)] flex items-center justify-between">
                                    <div className="flex items-center gap-2">
                                        <span className="text-[var(--color-cosmic-cyan)] text-sm">⌨</span>
                                        <h3 className="text-sm font-bold uppercase tracking-wider">Live Console</h3>
                                    </div>
                                    <div className="flex gap-1.5">
                                        <span className="w-2.5 h-2.5 rounded-full bg-[var(--color-cosmic-border)]" />
                                        <span className="w-2.5 h-2.5 rounded-full bg-[var(--color-cosmic-border)]" />
                                        <span className="w-2.5 h-2.5 rounded-full bg-[var(--color-cosmic-border)]" />
                                    </div>
                                </div>
                                <div className="flex-1 bg-black p-4 font-mono text-[12px] leading-relaxed overflow-y-auto" style={{ maxHeight: '300px' }}>
                                    {logs.length === 0 ? (
                                        <div className="text-[var(--color-text-muted)] italic">Waiting for log output...</div>
                                    ) : logs.map((log, i) => (
                                        <div key={i} className={`${log.isCmd ? 'text-[var(--color-cosmic-cyan)] font-bold' : log.isError ? 'text-[var(--color-cosmic-red)]' : 'text-[var(--color-text-main)]'}`}>
                                            <span className="text-[var(--color-text-muted)]">[{log.time}]</span> {log.text}
                                        </div>
                                    ))}
                                    <div ref={logEndRef} />
                                </div>
                                <form onSubmit={handleCommand} className="p-3 bg-[rgba(0,0,0,0.3)] flex items-center gap-2 border-t border-[var(--color-cosmic-border)]">
                                    <span className="text-[var(--color-cosmic-cyan)] font-mono">&gt;</span>
                                    <input
                                        type="text"
                                        value={cmdInput}
                                        onChange={e => setCmdInput(e.target.value)}
                                        placeholder="Type command..."
                                        className="bg-transparent border-none outline-none text-[var(--color-text-main)] w-full text-xs font-mono placeholder:text-[var(--color-text-muted)]"
                                    />
                                </form>
                                <div className="p-3">
                                    <Link to="/console" className="w-full py-2 rounded-lg bg-[rgba(13,223,242,0.1)] border border-[rgba(13,223,242,0.3)] text-[var(--color-cosmic-cyan)] text-xs font-bold hover:bg-[rgba(13,223,242,0.2)] transition-all flex items-center justify-center gap-2">
                                        Open Full Console ↗
                                    </Link>
                                </div>
                            </section>

                            {/* Active Alerts Card */}
                            {server.environment !== 'development' && (
                                <section className="glass-panel p-6">
                                    <h3 className="text-sm font-bold uppercase tracking-wider mb-4">Active Alerts</h3>
                                    <div className="flex flex-col gap-3">
                                        {history.length === 0 ? (
                                            <div className="text-center text-[var(--color-text-muted)] py-4 text-sm">No alerts for this server</div>
                                        ) : history.slice(0, 5).map(h => (
                                            <div key={h.id} className={`p-3 rounded-lg flex items-start gap-3 border
                                    ${h.event_type === 'triggered'
                                                    ? 'bg-[rgba(245,158,11,0.1)] border-[rgba(245,158,11,0.3)]'
                                                    : 'bg-[rgba(16,185,129,0.05)] border-[rgba(16,185,129,0.2)]'}`}>
                                                <span className={`text-lg ${h.event_type === 'triggered' ? 'text-[var(--color-cosmic-orange)]' : 'text-[var(--color-cosmic-green)]'}`}>
                                                    {h.event_type === 'triggered' ? '⚠' : '✓'}
                                                </span>
                                                <div>
                                                    <p className={`text-xs font-bold ${h.event_type === 'triggered' ? 'text-[var(--color-cosmic-orange)]' : 'text-[var(--color-cosmic-green)]'}`}>
                                                        {h.event_type === 'triggered' ? 'Warning' : 'Resolved'}
                                                    </p>
                                                    <p className="text-[11px] text-[var(--color-text-muted)] mt-0.5">
                                                        {new Date(h.created_at).toLocaleString()}
                                                    </p>
                                                </div>
                                            </div>
                                        ))}
                                    </div>
                                </section>
                            )}
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}
