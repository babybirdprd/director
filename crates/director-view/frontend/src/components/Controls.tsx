import { useEffect, useRef, useCallback } from 'react';
import { useProjectStore } from '@/stores/project';
import {
    Play,
    Pause,
    SkipBack,
    SkipForward,
    Download,
    Loader2,
} from 'lucide-react';

function formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toFixed(2).padStart(5, '0')}`;
}

export function Controls() {
    const {
        currentTime,
        duration,
        isPlaying,
        fps,
        isLoading,
        pause,
        togglePlayback,
        seek,
        setFps,
        exportVideo,
    } = useProjectStore();

    const lastTimeRef = useRef<number>(0);
    const animationFrameRef = useRef<number>(0);

    // Playback loop
    const tick = useCallback((timestamp: number) => {
        if (!lastTimeRef.current) {
            lastTimeRef.current = timestamp;
        }

        const deltaTime = (timestamp - lastTimeRef.current) / 1000;
        lastTimeRef.current = timestamp;

        const { currentTime, duration, isPlaying } = useProjectStore.getState();

        if (isPlaying) {
            const newTime = currentTime + deltaTime;
            if (newTime >= duration) {
                seek(duration);
                pause();
            } else {
                seek(newTime);
            }
            animationFrameRef.current = requestAnimationFrame(tick);
        }
    }, [seek, pause]);

    useEffect(() => {
        if (isPlaying) {
            lastTimeRef.current = 0;
            animationFrameRef.current = requestAnimationFrame(tick);
        }

        return () => {
            if (animationFrameRef.current) {
                cancelAnimationFrame(animationFrameRef.current);
            }
        };
    }, [isPlaying, tick]);

    // Keyboard shortcuts
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            // Don't handle if typing in an input or editor
            if (
                e.target instanceof HTMLInputElement ||
                e.target instanceof HTMLTextAreaElement ||
                (e.target as HTMLElement).closest('.monaco-editor')
            ) {
                return;
            }

            switch (e.key) {
                case ' ':
                    e.preventDefault();
                    togglePlayback();
                    break;
                case 'j':
                    // Rewind
                    seek(Math.max(0, currentTime - 5));
                    break;
                case 'k':
                    // Play/pause
                    togglePlayback();
                    break;
                case 'l':
                    // Fast forward
                    seek(Math.min(duration, currentTime + 5));
                    break;
                case ',':
                    // Previous frame
                    if (!isPlaying) {
                        seek(Math.max(0, currentTime - 1 / fps));
                    }
                    break;
                case '.':
                    // Next frame
                    if (!isPlaying) {
                        seek(Math.min(duration, currentTime + 1 / fps));
                    }
                    break;
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [currentTime, duration, fps, isPlaying, seek, togglePlayback]);

    const handleExport = async () => {
        const outputPath = window.prompt('Enter output path:', 'output.mp4');
        if (outputPath) {
            await exportVideo(outputPath);
        }
    };

    return (
        <div className="flex items-center justify-between px-4 py-2 bg-director-surface border-t border-director-border">
            {/* Left: Playback controls */}
            <div className="flex items-center gap-2">
                <button
                    onClick={() => seek(0)}
                    className="btn-icon"
                    title="Go to start (Home)"
                >
                    <SkipBack size={18} />
                </button>

                <button
                    onClick={togglePlayback}
                    disabled={isLoading}
                    className="btn-icon w-10 h-10 bg-director-accent hover:bg-director-accent-hover text-white"
                    title={isPlaying ? 'Pause (Space)' : 'Play (Space)'}
                >
                    {isLoading ? (
                        <Loader2 size={20} className="animate-spin" />
                    ) : isPlaying ? (
                        <Pause size={20} />
                    ) : (
                        <Play size={20} className="ml-0.5" />
                    )}
                </button>

                <button
                    onClick={() => seek(duration)}
                    className="btn-icon"
                    title="Go to end (End)"
                >
                    <SkipForward size={18} />
                </button>
            </div>

            {/* Center: Time display */}
            <div className="flex items-center gap-4">
                <div className="font-mono text-lg">
                    <span className="text-director-text">{formatTime(currentTime)}</span>
                    <span className="text-director-text-muted mx-2">/</span>
                    <span className="text-director-text-muted">{formatTime(duration)}</span>
                </div>
            </div>

            {/* Right: Settings and export */}
            <div className="flex items-center gap-2">
                {/* FPS selector */}
                <div className="flex items-center gap-2">
                    <label className="text-xs text-director-text-muted">FPS:</label>
                    <select
                        value={fps}
                        onChange={(e) => setFps(Number(e.target.value))}
                        className="bg-director-surface border border-director-border rounded px-2 py-1 text-sm"
                    >
                        <option value={24}>24</option>
                        <option value={30}>30</option>
                        <option value={60}>60</option>
                    </select>
                </div>

                <div className="w-px h-6 bg-director-border mx-2" />

                <button
                    onClick={handleExport}
                    className="btn btn-secondary flex items-center gap-2"
                    title="Export video"
                >
                    <Download size={16} />
                    Export
                </button>
            </div>
        </div>
    );
}
