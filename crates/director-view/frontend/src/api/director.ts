/**
 * Director API Client
 * 
 * Communicates with the Rust backend (Axum server) for:
 * - Script initialization
 * - Frame rendering
 * - File operations
 * - Video export
 */

const API_BASE = '/api';

export interface InitResponse {
    status: string;
    duration: number;
}

export interface SceneInfo {
    index: number;
    startTime: number;
    duration: number;
    name?: string;
}

export interface ExportProgress {
    frame: number;
    totalFrames: number;
    percent: number;
}

class DirectorApi {
    /**
     * Initialize a script from a file path
     */
    async initFromPath(scriptPath: string): Promise<InitResponse> {
        const res = await fetch(`${API_BASE}/init?script_path=${encodeURIComponent(scriptPath)}`);
        const text = await res.text();

        try {
            return JSON.parse(text);
        } catch {
            throw new Error(text);
        }
    }

    /**
     * Initialize a script from content (requires backend update)
     */
    async initFromContent(scriptContent: string): Promise<InitResponse> {
        const res = await fetch(`${API_BASE}/init`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ script: scriptContent }),
        });

        if (!res.ok) {
            const error = await res.text();
            throw new Error(error);
        }

        return res.json();
    }

    /**
     * Render a frame at the given time
     * Returns a blob URL that should be revoked after use
     */
    async renderFrame(time: number, signal?: AbortSignal): Promise<string> {
        const res = await fetch(`${API_BASE}/render?time=${time}`, { signal });

        if (!res.ok) {
            const error = await res.text();
            throw new Error(error);
        }

        const blob = await res.blob();
        return URL.createObjectURL(blob);
    }

    /**
     * Read a file from the filesystem
     */
    async readFile(path: string): Promise<string> {
        const res = await fetch(`${API_BASE}/file?path=${encodeURIComponent(path)}`);

        if (!res.ok) {
            throw new Error(`Failed to read file: ${path}`);
        }

        return res.text();
    }

    /**
     * Get timeline scene information
     */
    async getScenes(): Promise<SceneInfo[]> {
        const res = await fetch(`${API_BASE}/scenes`);

        if (!res.ok) {
            return [];
        }

        return res.json();
    }

    /**
     * Trigger video export
     */
    async exportVideo(outputPath: string): Promise<void> {
        const res = await fetch(`${API_BASE}/export`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ output: outputPath }),
        });

        if (!res.ok) {
            const error = await res.text();
            throw new Error(error);
        }
    }

    /**
     * Check if the backend is available
     */
    async healthCheck(): Promise<boolean> {
        try {
            const res = await fetch(`${API_BASE}/health`, {
                method: 'GET',
                signal: AbortSignal.timeout(2000),
            });
            return res.ok;
        } catch {
            return false;
        }
    }
}

export const api = new DirectorApi();
