import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { api, SceneInfo } from '@/api/director';

export interface ProjectState {
    // Script state
    scriptContent: string;
    scriptPath: string | null;
    scriptError: string | null;
    isScriptDirty: boolean;

    // Playback state
    currentTime: number;
    duration: number;
    isPlaying: boolean;
    fps: number;

    // Timeline state
    scenes: SceneInfo[];

    // UI state
    isLoading: boolean;
    backendConnected: boolean;

    // Actions
    setScript: (content: string) => void;
    setScriptPath: (path: string | null) => void;
    loadScriptFromPath: (path: string) => Promise<void>;
    runScript: () => Promise<void>;
    seek: (time: number) => void;
    play: () => void;
    pause: () => void;
    togglePlayback: () => void;
    setFps: (fps: number) => void;
    checkBackend: () => Promise<void>;
}

const DEFAULT_SCRIPT = `// Hello World - Minimal Director Script
//
// This is the simplest possible script that produces output.
// It demonstrates:
// - Creating a director (movie)
// - Adding a scene
// - Adding text

let movie = new_director(1920, 1080, 30);
let scene = movie.add_scene(3.0);

// Add centered text
let root = scene.add_box(#{
    width: "100%",
    height: "100%",
    justify_content: "center",
    align_items: "center",
    bg_color: "#1a1a2e"
});

root.add_text(#{
    content: "Hello, Director!",
    size: 72.0,
    color: "#ffffff",
    weight: "bold"
});

movie
`;

export const useProjectStore = create<ProjectState>()(
    persist(
        (set, get) => ({
            // Initial state
            scriptContent: DEFAULT_SCRIPT,
            scriptPath: null,
            scriptError: null,
            isScriptDirty: false,
            currentTime: 0,
            duration: 3,
            isPlaying: false,
            fps: 30,
            scenes: [],
            isLoading: false,
            backendConnected: false,

            // Actions
            setScript: (content) => set({
                scriptContent: content,
                isScriptDirty: true,
                scriptError: null,
            }),

            setScriptPath: (path) => set({ scriptPath: path }),

            loadScriptFromPath: async (path) => {
                set({ isLoading: true, scriptError: null });
                try {
                    const content = await api.readFile(path);
                    set({
                        scriptContent: content,
                        scriptPath: path,
                        isScriptDirty: false,
                        isLoading: false,
                    });
                } catch (e) {
                    set({
                        scriptError: e instanceof Error ? e.message : 'Failed to load file',
                        isLoading: false,
                    });
                }
            },

            runScript: async () => {
                const { scriptPath, scriptContent } = get();
                set({ isLoading: true, scriptError: null });

                try {
                    // Use path-based init if we have a path, otherwise content-based
                    const result = scriptPath
                        ? await api.initFromPath(scriptPath)
                        : await api.initFromContent(scriptContent);

                    // Fetch scene info for timeline markers
                    const scenes = await api.getScenes();

                    set({
                        duration: result.duration,
                        scenes,
                        currentTime: 0,
                        isScriptDirty: false,
                        isLoading: false,
                    });
                } catch (e) {
                    set({
                        scriptError: e instanceof Error ? e.message : 'Script execution failed',
                        isLoading: false,
                    });
                }
            },

            seek: (time) => {
                const { duration } = get();
                const clampedTime = Math.max(0, Math.min(time, duration));
                set({ currentTime: clampedTime });
            },

            play: () => set({ isPlaying: true }),

            pause: () => set({ isPlaying: false }),

            togglePlayback: () => set((state) => ({ isPlaying: !state.isPlaying })),

            setFps: (fps) => set({ fps }),

            checkBackend: async () => {
                const connected = await api.healthCheck();
                set({ backendConnected: connected });
            },
        }),
        {
            name: 'director-project',
            partialize: (state) => ({
                scriptContent: state.scriptContent,
                scriptPath: state.scriptPath,
                fps: state.fps,
            }),
        }
    )
);

// Playback loop hook
export function usePlaybackLoop() {
    const { isPlaying, currentTime, duration, fps, seek, pause } = useProjectStore();

    const frameTime = 1 / fps;

    // This would be called in a useEffect with requestAnimationFrame
    const tick = (deltaTime: number) => {
        if (!isPlaying) return;

        const newTime = currentTime + deltaTime;
        if (newTime >= duration) {
            seek(duration);
            pause();
        } else {
            seek(newTime);
        }
    };

    return { tick, frameTime };
}
