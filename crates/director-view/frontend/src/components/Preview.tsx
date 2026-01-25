import { useEffect, useRef, useState, useCallback } from 'react';
import { api } from '@/api/director';
import { useProjectStore } from '@/stores/project';
import { Loader2, ZoomIn, ZoomOut, Maximize2 } from 'lucide-react';

type ZoomLevel = 'fit' | 50 | 100 | 200;

export function Preview() {
    const { currentTime, isLoading } = useProjectStore();
    const [frameSrc, setFrameSrc] = useState<string | null>(null);
    const [isRendering, setIsRendering] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [zoom, setZoom] = useState<ZoomLevel>('fit');

    const containerRef = useRef<HTMLDivElement>(null);
    const abortControllerRef = useRef<AbortController | null>(null);
    const pendingTimeRef = useRef<number | null>(null);
    const renderingRef = useRef(false);

    // Debounced frame fetching
    const fetchFrame = useCallback(async (time: number) => {
        // If already rendering, queue this time
        if (renderingRef.current) {
            pendingTimeRef.current = time;
            return;
        }

        renderingRef.current = true;
        setIsRendering(true);

        // Cancel any pending request
        if (abortControllerRef.current) {
            abortControllerRef.current.abort();
        }
        abortControllerRef.current = new AbortController();

        try {
            const url = await api.renderFrame(time, abortControllerRef.current.signal);

            // Revoke old URL
            if (frameSrc) {
                URL.revokeObjectURL(frameSrc);
            }

            setFrameSrc(url);
            setError(null);
        } catch (e) {
            if (e instanceof Error && e.name !== 'AbortError') {
                setError(e.message);
            }
        } finally {
            renderingRef.current = false;
            setIsRendering(false);

            // Process pending request
            if (pendingTimeRef.current !== null) {
                const pendingTime = pendingTimeRef.current;
                pendingTimeRef.current = null;
                fetchFrame(pendingTime);
            }
        }
    }, [frameSrc]);

    // Fetch frame when time changes
    useEffect(() => {
        fetchFrame(currentTime);
    }, [currentTime, fetchFrame]);

    // Cleanup on unmount
    useEffect(() => {
        return () => {
            if (frameSrc) {
                URL.revokeObjectURL(frameSrc);
            }
            if (abortControllerRef.current) {
                abortControllerRef.current.abort();
            }
        };
    }, []);

    const cycleZoom = () => {
        const levels: ZoomLevel[] = ['fit', 50, 100, 200];
        const currentIndex = levels.indexOf(zoom);
        const nextIndex = (currentIndex + 1) % levels.length;
        setZoom(levels[nextIndex]);
    };

    const getZoomStyle = () => {
        if (zoom === 'fit') {
            return { maxWidth: '100%', maxHeight: '100%', width: 'auto', height: 'auto' };
        }
        return { width: `${zoom}%`, height: 'auto' };
    };

    return (
        <div className="flex flex-col h-full">
            {/* Toolbar */}
            <div className="flex items-center justify-between px-3 py-2 border-b border-director-border bg-director-surface/50">
                <span className="text-sm text-director-text-muted">Preview</span>
                <div className="flex items-center gap-1">
                    <button
                        onClick={() => setZoom(50)}
                        className={`btn-icon text-xs ${zoom === 50 ? 'bg-director-accent' : ''}`}
                        title="50%"
                    >
                        <ZoomOut size={16} />
                    </button>
                    <button
                        onClick={cycleZoom}
                        className="btn-icon text-xs min-w-[48px]"
                        title="Cycle zoom"
                    >
                        {zoom === 'fit' ? 'Fit' : `${zoom}%`}
                    </button>
                    <button
                        onClick={() => setZoom(200)}
                        className={`btn-icon text-xs ${zoom === 200 ? 'bg-director-accent' : ''}`}
                        title="200%"
                    >
                        <ZoomIn size={16} />
                    </button>
                    <button
                        onClick={() => setZoom('fit')}
                        className={`btn-icon text-xs ${zoom === 'fit' ? 'bg-director-accent' : ''}`}
                        title="Fit to view"
                    >
                        <Maximize2 size={16} />
                    </button>
                </div>
            </div>

            {/* Preview area */}
            <div
                ref={containerRef}
                className="flex-1 flex items-center justify-center overflow-auto bg-black/50 relative"
            >
                {isLoading && !frameSrc && (
                    <div className="flex flex-col items-center gap-2 text-director-text-muted">
                        <Loader2 className="animate-spin" size={32} />
                        <span>Loading script...</span>
                    </div>
                )}

                {error && (
                    <div className="flex flex-col items-center gap-2 text-red-400 p-4 text-center">
                        <span className="font-medium">Render Error</span>
                        <span className="text-sm opacity-75">{error}</span>
                    </div>
                )}

                {frameSrc && !error && (
                    <img
                        src={frameSrc}
                        alt="Preview frame"
                        style={getZoomStyle()}
                        className="object-contain"
                        draggable={false}
                    />
                )}

                {!frameSrc && !isLoading && !error && (
                    <div className="flex flex-col items-center gap-2 text-director-text-muted">
                        <span>No preview available</span>
                        <span className="text-sm">Run a script to see the preview</span>
                    </div>
                )}

                {/* Rendering indicator */}
                {isRendering && (
                    <div className="absolute top-2 right-2 flex items-center gap-2 px-2 py-1 bg-director-surface/80 rounded text-xs text-director-text-muted">
                        <Loader2 className="animate-spin" size={12} />
                        Rendering...
                    </div>
                )}
            </div>
        </div>
    );
}
