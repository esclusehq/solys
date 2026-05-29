import { NavLink } from 'react-router-dom';
import { useWorkspace } from '../context/WorkspaceContext';
import { useAuthStore } from '../store/authStore';

const opsNav = [
    { to: '/', icon: 'grid', label: 'Dashboard' },
    { to: '/servers', icon: 'server', label: 'Server Manager' },
    { to: '/nodes', icon: 'cluster', label: 'Nodes' },
    { to: '/console', icon: 'terminal', label: 'RCON Console' },
    { to: '/alerts', icon: 'bell', label: 'Alerts' },
    { to: '/settings', icon: 'gear', label: 'System Settings' },
];

const devNav = [
    { to: '/', icon: 'grid', label: 'Dashboard' },
    { to: '/servers', icon: 'server', label: 'Dev Environments' },
    // Later we will add: Web IDE, Git Repos, Build Pipelines, Profiler here
    { to: '/settings', icon: 'gear', label: 'System Settings' },
];

const icons = {
    grid: <><rect x="3" y="3" width="7" height="9" /><rect x="14" y="3" width="7" height="5" /><rect x="14" y="12" width="7" height="9" /><rect x="3" y="16" width="7" height="5" /></>,
    server: <><rect x="2" y="2" width="20" height="8" rx="2" /><rect x="2" y="14" width="20" height="8" rx="2" /><line x1="6" y1="6" x2="6.01" y2="6" /><line x1="6" y1="18" x2="6.01" y2="18" /></>,
    cluster: <><circle cx="12" cy="12" r="3"/><circle cx="19" cy="5" r="2"/><circle cx="5" cy="5" r="2"/><circle cx="5" cy="19" r="2"/><circle cx="19" cy="19" r="2"/></>,
    terminal: <><polyline points="4 17 10 11 4 5" /><line x1="12" y1="19" x2="20" y2="19" /></>,
    bell: <><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9" /><path d="M13.73 21a2 2 0 0 1-3.46 0" /></>,
    gear: <><circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" /></>,
};

export default function Sidebar() {
    const { activeMode, setActiveMode } = useWorkspace();
    const currentNav = activeMode === 'ops' ? opsNav : devNav;
    const { user } = useAuthStore();

    return (
        <aside className="w-60 h-screen sticky top-0 flex flex-col glass-panel border-r border-[var(--color-cosmic-border)] z-10">
            <div className="p-5 border-b border-[var(--color-cosmic-border)]">
                <div className="flex items-center gap-2 text-xl font-bold mb-4">
                    <img src="/logo.svg" alt="Escluse" className="w-16 h-16" />
                    <span className="font-bold text-lg">Escluse</span>
                    <span style={{
                        fontSize: '0.55rem',
                        fontWeight: '700',
                        letterSpacing: '0.08em',
                        padding: '2px 7px',
                        borderRadius: '999px',
                        background: 'linear-gradient(135deg, #f59e0b, #d97706)',
                        color: '#000',
                        boxShadow: '0 0 8px rgba(245,158,11,0.5)',
                        alignSelf: 'center',
                        textTransform: 'uppercase',
                        lineHeight: '1.4',
                        marginBottom: '1px',
                    }}>Alpha</span>
                </div>

                {/* Perspective Toggle */}
                <div className="bg-[rgba(0,0,0,0.3)] rounded-lg p-1 flex border border-[var(--color-cosmic-border)]">
                    <button
                        onClick={() => setActiveMode('ops')}
                        className={`flex-1 py-1.5 px-2 text-xs font-medium rounded transition-all duration-200 flex items-center justify-center gap-1.5 ${activeMode === 'ops'
                                ? 'bg-[var(--color-cosmic-cyan)] text-black shadow-[0_0_10px_rgba(13,223,242,0.5)]'
                                : 'text-[var(--color-text-muted)] hover:text-white'
                            }`}
                    >
                        <span>🛡️</span> Ops
                    </button>
                    <button
                        onClick={() => setActiveMode('dev')}
                        className={`flex-1 py-1.5 px-2 text-xs font-medium rounded transition-all duration-200 flex items-center justify-center gap-1.5 ${activeMode === 'dev'
                                ? 'bg-[var(--color-cosmic-magenta)] text-white shadow-[0_0_10px_rgba(255,0,255,0.5)]'
                                : 'text-[var(--color-text-muted)] hover:text-white'
                            }`}
                    >
                        <span>🔨</span> Dev
                    </button>
                </div>
            </div>
            <nav className="flex-1 p-3 flex flex-col gap-1 overflow-y-auto">
                {currentNav.map(({ to, icon, label }) => (
                    <NavLink
                        key={to}
                        to={to}
                        end={to === '/'}
                        className={({ isActive }) =>
                            `flex items-center gap-3 px-4 py-3 rounded-lg text-sm font-medium transition-all duration-200
              ${isActive
                                ? `bg-[rgba(255,255,255,0.1)] text-white ${activeMode === 'ops' ? 'glow-cyan' : 'glow-magenta'}`
                                : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-main)] hover:bg-[rgba(255,255,255,0.05)]'}`
                        }
                    >
                        <svg className="w-5 h-5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            {icons[icon]}
                        </svg>
                        {label}
                    </NavLink>
                ))}
            </nav>

            {/* User Info Area — D-11: Sidebar only */}
            {user && (
                <div className="p-3 border-t border-[var(--color-cosmic-border)]">
                    <div className="flex items-center gap-3 px-3 py-2 rounded-lg bg-[rgba(255,255,255,0.03)]">
                        <div className="w-8 h-8 rounded-full overflow-hidden flex-shrink-0 bg-gray-700">
                            {user.avatar_url ? (
                                <img src={user.avatar_url} alt="" className="w-full h-full object-cover" />
                            ) : (
                                <div className="w-full h-full flex items-center justify-center text-gray-400 text-xs font-medium">
                                    {(user.display_name || user.email || '?')[0].toUpperCase()}
                                </div>
                            )}
                        </div>
                        <div className="min-w-0 flex-1">
                            <p className="text-sm text-white truncate">
                                {user.display_name || user.email?.split('@')[0] || 'User'}
                            </p>
                            <p className="text-xs text-gray-500 truncate">{user.email}</p>
                        </div>
                    </div>
                </div>
            )}
        </aside>
    );
}
