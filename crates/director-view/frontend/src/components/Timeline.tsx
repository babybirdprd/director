import { useRef, useCallback, useEffect } from 'react';
import { useProjectStore } from '@/stores/project';

function formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toFixed(2).padStart(5, '0')}`;
}

export function Timeline() {
    const { currentTime, duration, scenes, seek, isPlaying } = useProjectStore();
    const trackRef = useRef<HTMLDivElement>(null);
    const isDraggingRef = useRef(false);

    const calculateTimeFromPosition = useCallback((clientX: number): number => {
        if (!trackRef.current) return 0;

        const rect = trackRef.current.getBoundingClientRect();
        const x = clientX - rect.left;
        const percent = Math.max(0, Math.min(1, x / rect.width));
        return percent * duration;
    }, [duration]);

    const handleMouseDown = useCallback((e: React.MouseEvent) => {
        isDraggingRef.current = true;
        const time = calculateTimeFromPosition(e.clientX);
        seek(time);

        // Prevent text selection while dragging
        e.preventDefault();
    }, [calculateTimeFromPosition, seek]);

    const handleMouseMove = useCallback((e: MouseEvent) => {
        if (!isDraggingRef.current) return;
        const time = calculateTimeFromPosition(e.clientX);
        seek(time);
    }, [calculateTimeFromPosition, seek]);

    const handleMouseUp = useCallback(() => {
        isDraggingRef.current = false;
    }, []);

    // Global mouse events for dragging
    useEffect(() => {
        window.addEventListener('mousemove', handleMouseMove);
        window.addEventListener('mouseup', handleMouseUp);

        return () => {
            window.removeEventListener('mousemove', handleMouseMove);
            window.removeEventListener('mouseup', handleMouseUp);
        };
    }, [handleMouseMove, handleMouseUp]);

    // Keyboard shortcuts
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            // Don't handle if typing in an input
            if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
                return;
            }

            switch (e.key) {
                case 'ArrowLeft':
                    e.preventDefault();
                    seek(Math.max(0, currentTime - (e.shiftKey ? 1 : 0.1)));
                    break;
                case 'ArrowRight':
                    e.preventDefault();
                    seek(Math.min(duration, currentTime + (e.shiftKey ? 1 : 0.1)));
                    break;
                case 'Home':
                    e.preventDefault();
                    seek(0);
                    break;
                case 'End':
                    e.preventDefault();
                    seek(duration);
                    break;
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [currentTime, duration, seek]);

    const progress = duration > 0 ? (currentTime / duration) * 100 : 0;

    return (
        <div className="px-4 py-3 bg-director-surface border-t border-director-border">
            {/* Time display */}
            <div className="flex justify-between text-xs text-director-text-muted mb-2">
                <span className="font-mono">{formatTime(currentTime)}</span>
                <span className="font-mono">{formatTime(duration)}</span>
            </div>

            {/* Timeline track */}
            <div
                ref={trackRef}
                className="relative h-6 bg-director-bg rounded cursor-pointer group"
                onMouseDown={handleMouseDown}
            >
                {/* Scene markers */}
                {scenes.map((scene, i) => {
                    const startPercent = (scene.startTime / duration) * 100;
                    const widthPercent = (scene.duration / duration) * 100;
                    const colors = ['#3b82f6', '#8b5cf6', '#ec4899', '#f59e0b', '#10b981'];
                    const color = colors[i % colors.length];

                    return (
                        <div
                            key={i}
                            className="absolute top-0 h-full opacity-30 rounded"
                            style={{
                                left: `${startPercent}%`,
                                width: `${widthPercent}%`,
                                backgroundColor: color,
                            }}
                            title={scene.name || `Scene ${i + 1}`}
                        />
                    );
                })}

                {/* Progress bar */}
                <div
                    className="absolute top-0 h-full bg-director-accent/50 rounded-l"
                    style={{ width: `${progress}%` }}
                />

                {/* Playhead */}
                <div
                    className="absolute top-0 h-full w-0.5 bg-white shadow-lg"
                    style={{ left: `${progress}%`, transform: 'translateX(-50%)' }}
                >
                    {/* Playhead handle */}
                    <div className="absolute -top-1 left-1/2 -translate-x-1/2 w-3 h-3 bg-white rounded-full shadow-md" />
                </div>

                {/* Hover time indicator */}
                <div className="absolute inset-0 opacity-0 group-hover:opacity-100 pointer-events-none">
                    {/* This would show time on hover - simplified for now */}
                </div>
            </div>

            {/* Tick marks */}
            <div className="relative h-3 mt-1">
                {Array.from({ length: 11 }).map((_, i) => (
                    <div
                        key={i}
                        className="absolute w-px h-2 bg-director-border"
                        style={{ left: `${i * 10}%` }}
                    />
                ))}
            </div>

            {/* Playback indicator */}
            {isPlaying && (
                <div className="flex items-center justify-center mt-2">
                    <div className="flex items-center gap-1 text-xs text-director-accent">
                        <div className="w-2 h-2 bg-director-accent rounded-full animate-pulse" />
                        Playing
                    </div>
                </div>
            )}
        </div>
    );
}
