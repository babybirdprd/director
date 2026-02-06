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

export interface ApiErrorPayload {
    error: string;
    line?: number;
    column?: number;
    snippet?: string;
}

export interface ExportResponse {
    status: string;
    output: string;
}

async function parseErrorResponse(res: Response): Promise<Error> {
    const text = await res.text();
    try {
        const payload = JSON.parse(text) as ApiErrorPayload;
        if (payload.error) {
            const location = payload.line
                ? ` (line ${payload.line}${payload.column ? `, col ${payload.column}` : ''})`
                : '';
            const snippet = payload.snippet ? `\n> ${payload.snippet}` : '';
            return new Error(`${payload.error}${location}${snippet}`);
        }
    } catch {
        // Fall through to plain-text error.
    }
    return new Error(text || `HTTP ${res.status}`);
}

class DirectorApi {
    /**
     * Initialize a script from a file path
     */
    async initFromPath(scriptPath: string): Promise<InitResponse> {
        const res = await fetch(`${API_BASE}/init?script_path=${encodeURIComponent(scriptPath)}`);
        if (!res.ok) {
            throw await parseErrorResponse(res);
        }
        return res.json();
    }

    /**
     * Initialize a script from inline content
     */
    async initFromContent(scriptContent: string): Promise<InitResponse> {
        const res = await fetch(`${API_BASE}/init`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ script: scriptContent }),
        });

        if (!res.ok) {
            throw await parseErrorResponse(res);
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
            throw await parseErrorResponse(res);
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
            throw await parseErrorResponse(res);
        }

        return res.text();
    }

    /**
     * Save script content to a file path
     */
    async saveFile(path: string, content: string): Promise<void> {
        const res = await fetch(`${API_BASE}/file`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ path, content }),
        });

        if (!res.ok) {
            throw await parseErrorResponse(res);
        }
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
    async exportVideo(outputPath: string): Promise<ExportResponse> {
        const res = await fetch(`${API_BASE}/export`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ output: outputPath }),
        });

        if (!res.ok) {
            throw await parseErrorResponse(res);
        }

        return res.json();
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
