import { useEffect, useState } from 'react';
import { Preview, Timeline, ScriptEditor, Controls } from '@/components';
import { useProjectStore } from '@/stores/project';
import { Video, AlertTriangle, RefreshCw } from 'lucide-react';

function App() {
    const {
        checkBackend,
        backendConnected,
        runScript,
        loadAvailableScripts,
        selectScript,
    } = useProjectStore();
    const [hasInitializedScript, setHasInitializedScript] = useState(false);

    // Check backend connection on mount
    useEffect(() => {
        checkBackend();
        const interval = setInterval(checkBackend, 5000);
        return () => clearInterval(interval);
    }, [checkBackend]);

    // Load script catalog on first backend connect, then auto-run first script.
    useEffect(() => {
        if (!backendConnected || hasInitializedScript) {
            return;
        }

        let cancelled = false;
        const timer = setTimeout(async () => {
            try {
                await loadAvailableScripts();
                const { availableScripts } = useProjectStore.getState();
                if (cancelled) {
                    return;
                }

                if (availableScripts.length > 0) {
                    await selectScript(availableScripts[0].path);
                } else {
                    await runScript();
                }
            } finally {
                if (!cancelled) {
                    setHasInitializedScript(true);
                }
            }
        }, 500);

        return () => {
            cancelled = true;
            clearTimeout(timer);
        };
    }, [backendConnected, hasInitializedScript, loadAvailableScripts, runScript, selectScript]);

    return (
        <div className="h-full flex flex-col bg-director-bg">
            {/* Header */}
            <header className="flex items-center justify-between px-4 py-2 border-b border-director-border bg-director-surface">
                <div className="flex items-center gap-3">
                    <Video className="text-director-accent" size={24} />
                    <h1 className="text-lg font-semibold text-director-text">Director View</h1>
                </div>
                <div className="flex items-center gap-4">
                    {/* Connection status */}
                    <div className="flex items-center gap-2 text-sm">
                        <div
                            className={`w-2 h-2 rounded-full ${backendConnected ? 'bg-green-500' : 'bg-red-500'
                                }`}
                        />
                        <span className="text-director-text-muted">
                            {backendConnected ? 'Connected' : 'Disconnected'}
                        </span>
                    </div>
                </div>
            </header>

            {/* Main content */}
            <div className="flex-1 flex overflow-hidden">
                {/* Left panel: Preview */}
                <div className="flex-1 flex flex-col min-w-0 border-r border-director-border">
                    <div className="flex-1 overflow-hidden">
                        <Preview />
                    </div>
                    <Timeline />
                </div>

                {/* Right panel: Script Editor */}
                <div className="w-[500px] flex-shrink-0 flex flex-col">
                    <ScriptEditor />
                </div>
            </div>

            {/* Footer: Controls */}
            <Controls />

            {/* Backend disconnected overlay */}
            {!backendConnected && (
                <div className="fixed inset-0 bg-black/80 flex items-center justify-center z-50">
                    <div className="bg-director-surface border border-director-border rounded-lg p-6 max-w-md text-center">
                        <AlertTriangle className="mx-auto text-yellow-500 mb-4" size={48} />
                        <h2 className="text-xl font-semibold text-director-text mb-2">
                            Backend Not Connected
                        </h2>
                        <p className="text-director-text-muted mb-4">
                            The Director backend server is not running. Start it with:
                        </p>
                        <pre className="bg-director-bg rounded p-3 text-sm font-mono text-left mb-4 overflow-x-auto">
                            cargo run -p director-view
                        </pre>
                        <button
                            onClick={() => checkBackend()}
                            className="btn btn-primary flex items-center gap-2 mx-auto"
                        >
                            <RefreshCw size={16} />
                            Retry Connection
                        </button>
                    </div>
                </div>
            )}
        </div>
    );
}

export default App;
