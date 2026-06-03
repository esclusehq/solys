import { useState, useEffect, useRef } from 'react';
import { useServers } from '../hooks/useServers';
import Terminal from '../components/Terminal';

export default function Console() {
    const { servers } = useServers();
    const [selectedId, setSelectedId] = useState('');

    return (
        <div className="flex-1 flex flex-col overflow-hidden h-full min-h-0">
            {/* Header */}
            <header className="shrink-0 px-8 py-6 border-b border-[var(--color-cosmic-border)] flex items-center justify-between">
                <h1 className="text-2xl font-bold">Console</h1>
                <div className="flex items-center gap-4">
                    <select
                        value={selectedId}
                        onChange={(e) => setSelectedId(e.target.value)}
                        className="bg-[rgba(255,255,255,0.05)] border border-[var(--color-cosmic-border)] text-[var(--color-text-main)] rounded-lg px-4 py-2.5 text-sm outline-none w-full max-w-md"
                    >
                        <option value="">— Select Server —</option>
                        {servers.map(s => (
                            <option key={s.id} value={s.id}>{s.name} ({s.host}:{s.port})</option>
                        ))}
                    </select>
                    <div className="flex items-center gap-2">
                        <span
                            className={`w-2 h-2 rounded-full ${
                                selectedId
                                    ? 'bg-[var(--color-cosmic-green)] animate-pulse-glow'
                                    : 'bg-[var(--color-cosmic-red)]'
                            }`}
                        />
                        <span className="text-xs text-[var(--color-text-muted)]">
                            {selectedId ? 'Connected' : 'Disconnected'}
                        </span>
                    </div>
                </div>
            </header>

            {/* Terminal Area */}
            <div className="flex-1 flex px-8 pb-6 pt-5 overflow-hidden min-h-0">
                {!selectedId ? (
                    <div className="flex-1 flex items-center justify-center text-[var(--color-text-muted)]">
                        Select a server to open its console
                    </div>
                ) : (
                    <div className="flex-1 flex min-h-[400px]">
                        <Terminal serverId={selectedId} />
                    </div>
                )}
            </div>
        </div>
    );
}
