# Director View Frontend Plan

> **Status:** Draft  
> **Created:** 2025-01-25  
> **Decision:** Start with Vite + React, keep Dioxus as future option

---

## Executive Summary

Improve `director-view` from a basic scrubbing demo into a full-featured preview/editing interface. We'll use **Vite + React** for rapid iteration, with a clear migration path to Tauri or Dioxus later.

---

## Current State

The existing [`director-view`](../crates/director-view) crate provides:

```
┌─────────────────────────────────────────────────────────┐
│  Axum Server (Rust)                                     │
│  ├── /init?script_path=...  → Load Rhai script          │
│  ├── /render?time=...       → Return JPEG frame         │
│  └── Director Thread        → Manages rendering state   │
└─────────────────────────────────────────────────────────┘
                        ↕ HTTP
┌─────────────────────────────────────────────────────────┐
│  Static HTML/JS                                         │
│  ├── Script path input                                  │
│  ├── Preview image                                      │
│  └── Time scrubber                                      │
└─────────────────────────────────────────────────────────┘
```

**Limitations:**
- No script editing (must provide file path)
- No playback controls (play/pause)
- No timeline visualization
- No project management
- Basic styling

---

## Architecture: Vite + React

### Why React over Dioxus (for now)

| Aspect | React | Dioxus |
|--------|-------|--------|
| **Ecosystem** | Massive (Monaco, Zustand, Framer Motion) | Growing but limited |
| **Iteration Speed** | Instant HMR | Requires rebuild |
| **Learning Curve** | Familiar to most devs | Rust-specific patterns |
| **Component Libraries** | Thousands | Few dozen |
| **Migration to Tauri** | Trivial (just wrap) | N/A (already Rust) |

**Verdict:** React for prototyping speed. Dioxus remains viable for V2 if we want pure Rust.

### Project Structure

```
crates/director-view/
├── Cargo.toml              # Rust backend (Axum)
├── src/
│   └── main.rs             # API server
└── frontend/               # NEW: Vite + React app
    ├── package.json
    ├── vite.config.ts
    ├── index.html
    ├── src/
    │   ├── main.tsx
    │   ├── App.tsx
    │   ├── api/
    │   │   └── director.ts       # API client
    │   ├── components/
    │   │   ├── Preview.tsx       # Frame display
    │   │   ├── Timeline.tsx      # Scrubber + markers
    │   │   ├── ScriptEditor.tsx  # Monaco editor
    │   │   ├── Controls.tsx      # Play/pause/export
    │   │   └── ProjectPanel.tsx  # File browser
    │   ├── stores/
    │   │   └── project.ts        # Zustand state
    │   └── styles/
    │       └── globals.css
    └── public/
```

---

## Component Design

### 1. Preview Component

```tsx
// components/Preview.tsx
interface PreviewProps {
  time: number;
  width: number;
  height: number;
}

function Preview({ time, width, height }: PreviewProps) {
  const [frameSrc, setFrameSrc] = useState<string | null>(null);
  
  useEffect(() => {
    // Debounced fetch to /render?time=...
    const controller = new AbortController();
    fetchFrame(time, controller.signal).then(setFrameSrc);
    return () => controller.abort();
  }, [time]);
  
  return (
    <div className="preview-container" style={{ width, height }}>
      {frameSrc ? (
        <img src={frameSrc} alt="Preview" />
      ) : (
        <div className="loading">Rendering...</div>
      )}
    </div>
  );
}
```

**Features:**
- Debounced frame fetching (avoid flooding server)
- Loading state indicator
- Aspect ratio preservation
- Zoom controls (fit/100%/200%)

### 2. Timeline Component

```tsx
// components/Timeline.tsx
interface TimelineProps {
  duration: number;
  currentTime: number;
  onSeek: (time: number) => void;
  markers?: TimelineMarker[];
}

interface TimelineMarker {
  time: number;
  label: string;
  color: string;
}

function Timeline({ duration, currentTime, onSeek, markers }: TimelineProps) {
  return (
    <div className="timeline">
      <div className="timeline-track" onClick={handleClick}>
        <div 
          className="timeline-progress" 
          style={{ width: `${(currentTime / duration) * 100}%` }} 
        />
        <div 
          className="timeline-playhead"
          style={{ left: `${(currentTime / duration) * 100}%` }}
        />
        {markers?.map(m => (
          <div 
            key={m.time}
            className="timeline-marker"
            style={{ left: `${(m.time / duration) * 100}%`, background: m.color }}
            title={m.label}
          />
        ))}
      </div>
      <div className="timeline-labels">
        <span>0:00</span>
        <span>{formatTime(duration)}</span>
      </div>
    </div>
  );
}
```

**Features:**
- Click-to-seek
- Drag scrubbing
- Scene markers (from Director timeline)
- Keyboard shortcuts (J/K/L for playback)

### 3. Script Editor Component

```tsx
// components/ScriptEditor.tsx
import Editor from '@monaco-editor/react';

interface ScriptEditorProps {
  value: string;
  onChange: (value: string) => void;
  onRun: () => void;
}

function ScriptEditor({ value, onChange, onRun }: ScriptEditorProps) {
  return (
    <div className="script-editor">
      <div className="editor-toolbar">
        <button onClick={onRun}>▶ Run Script</button>
        <span className="status">{/* error/success indicator */}</span>
      </div>
      <Editor
        height="100%"
        language="rust"  // Rhai is close enough to Rust syntax
        theme="vs-dark"
        value={value}
        onChange={(v) => onChange(v ?? '')}
        options={{
          minimap: { enabled: false },
          fontSize: 14,
          lineNumbers: 'on',
          scrollBeyondLastLine: false,
        }}
      />
    </div>
  );
}
```

**Features:**
- Monaco editor with syntax highlighting
- Run button to execute script
- Error display with line numbers
- Auto-save to localStorage

### 4. Controls Component

```tsx
// components/Controls.tsx
interface ControlsProps {
  isPlaying: boolean;
  onPlayPause: () => void;
  onExport: () => void;
  currentTime: number;
  duration: number;
}

function Controls({ isPlaying, onPlayPause, onExport, currentTime, duration }: ControlsProps) {
  return (
    <div className="controls">
      <button onClick={onPlayPause}>
        {isPlaying ? '⏸' : '▶'}
      </button>
      <span className="timecode">
        {formatTime(currentTime)} / {formatTime(duration)}
      </span>
      <button onClick={onExport}>Export MP4</button>
    </div>
  );
}
```

---

## State Management (Zustand)

```typescript
// stores/project.ts
import { create } from 'zustand';

interface ProjectState {
  // Script
  scriptContent: string;
  scriptPath: string | null;
  scriptError: string | null;
  
  // Playback
  currentTime: number;
  duration: number;
  isPlaying: boolean;
  fps: number;
  
  // Actions
  setScript: (content: string) => void;
  loadScript: (path: string) => Promise<void>;
  runScript: () => Promise<void>;
  seek: (time: number) => void;
  play: () => void;
  pause: () => void;
}

export const useProjectStore = create<ProjectState>((set, get) => ({
  scriptContent: '',
  scriptPath: null,
  scriptError: null,
  currentTime: 0,
  duration: 10,
  isPlaying: false,
  fps: 30,
  
  setScript: (content) => set({ scriptContent: content }),
  
  loadScript: async (path) => {
    // Load from file system (via API)
    const content = await api.loadFile(path);
    set({ scriptContent: content, scriptPath: path });
  },
  
  runScript: async () => {
    const { scriptContent } = get();
    try {
      // Send to backend, get duration
      const result = await api.initScript(scriptContent);
      set({ duration: result.duration, scriptError: null, currentTime: 0 });
    } catch (e) {
      set({ scriptError: e.message });
    }
  },
  
  seek: (time) => set({ currentTime: time }),
  play: () => set({ isPlaying: true }),
  pause: () => set({ isPlaying: false }),
}));
```

---

## API Client

```typescript
// api/director.ts
const API_BASE = 'http://localhost:3000';

export const api = {
  async initScript(scriptContent: string): Promise<{ duration: number }> {
    // Option 1: Send script content directly (requires backend change)
    // Option 2: Write to temp file, send path (current approach)
    const res = await fetch(`${API_BASE}/init`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ script: scriptContent }),
    });
    return res.json();
  },
  
  async renderFrame(time: number): Promise<string> {
    const res = await fetch(`${API_BASE}/render?time=${time}`);
    const blob = await res.blob();
    return URL.createObjectURL(blob);
  },
  
  async loadFile(path: string): Promise<string> {
    const res = await fetch(`${API_BASE}/file?path=${encodeURIComponent(path)}`);
    return res.text();
  },
  
  async exportVideo(outputPath: string): Promise<void> {
    await fetch(`${API_BASE}/export`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ output: outputPath }),
    });
  },
};
```

---

## Backend API Changes

The current backend needs these additions:

### New Endpoints

```rust
// POST /init - Accept script content directly
#[derive(Deserialize)]
struct InitBody {
    script: String,  // Rhai script content
}

async fn init_handler_post(
    State(state): State<AppState>,
    Json(body): Json<InitBody>,
) -> Json<InitResponse> {
    // Write to temp file, then load
    // Or: eval script directly from string
}

// GET /file?path=... - Read file contents
async fn file_handler(
    Query(params): Query<FileParams>,
) -> String {
    std::fs::read_to_string(&params.path).unwrap_or_default()
}

// POST /export - Trigger video export
async fn export_handler(
    State(state): State<AppState>,
    Json(body): Json<ExportBody>,
) -> Json<ExportResponse> {
    // Trigger render_export in Director thread
}

// GET /scenes - Get timeline structure
async fn scenes_handler(
    State(state): State<AppState>,
) -> Json<Vec<SceneInfo>> {
    // Return timeline items for markers
}
```

---

## Layout Design

```
┌─────────────────────────────────────────────────────────────────────┐
│  Director View                                        [_][□][X]     │
├─────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────┐  ┌───────────────────────────────┐ │
│  │                             │  │  // Script Editor             │ │
│  │                             │  │  let m = new_director(        │ │
│  │      Preview Canvas         │  │    1920, 1080, 30             │ │
│  │      (16:9 aspect)          │  │  );                           │ │
│  │                             │  │                               │ │
│  │                             │  │  let scene = m.add_scene(5.0);│ │
│  │                             │  │  scene.add_box(...)           │ │
│  │                             │  │                               │ │
│  └─────────────────────────────┘  │  [▶ Run Script]               │ │
│                                   └───────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────────┤
│  [▶]  0:02.50 / 0:10.00    ═══════════●═══════════════    [Export] │
│       ↑ Play/Pause          ↑ Timeline with playhead               │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Phases

### Phase 1: Foundation (Week 1)
- [ ] Set up Vite + React project in `frontend/`
- [ ] Create basic Preview component with frame fetching
- [ ] Create Timeline component with seek
- [ ] Wire up to existing Axum backend
- [ ] Basic dark theme styling

### Phase 2: Script Editing (Week 2)
- [ ] Integrate Monaco editor
- [ ] Add POST /init endpoint for script content
- [ ] Error display with line highlighting
- [ ] Auto-save to localStorage

### Phase 3: Playback (Week 3)
- [ ] Play/pause with requestAnimationFrame loop
- [ ] Keyboard shortcuts (Space, J/K/L, arrow keys)
- [ ] FPS control
- [ ] Scene markers on timeline

### Phase 4: Export & Polish (Week 4)
- [ ] Export button with progress indicator
- [ ] Project save/load (JSON manifest)
- [ ] Recent files list
- [ ] Responsive layout

---

## Dioxus: Future Consideration

### Why Keep Dioxus on the Table

1. **Pure Rust Stack** - No JS toolchain, single language
2. **Direct Integration** - Can call `director-core` without HTTP
3. **Cross-Platform** - Desktop, Web, Mobile from one codebase
4. **Smaller Bundle** - No V8/WebView overhead (with native renderer)

### When to Consider Migration

- After React prototype validates the UX
- If HTTP overhead becomes a bottleneck
- If we need native OS features (tray, hotkeys)
- If team prefers Rust over TypeScript

### Migration Path

```
React (Vite)  →  React (Tauri)  →  Dioxus
     ↓                ↓               ↓
  HTTP API      Tauri Commands    Direct Rust
```

The API contract (`init`, `render`, `export`) stays the same at each step.

### Dioxus Component Example

```rust
// For reference: what the Preview component would look like in Dioxus
use dioxus::prelude::*;

#[component]
fn Preview(time: f64, width: u32, height: u32) -> Element {
    let frame_data = use_resource(move || async move {
        render_frame(time).await
    });
    
    rsx! {
        div { class: "preview-container",
            style: "width: {width}px; height: {height}px",
            match &*frame_data.read() {
                Some(Ok(data)) => rsx! { img { src: "{data}" } },
                Some(Err(e)) => rsx! { div { "Error: {e}" } },
                None => rsx! { div { "Loading..." } },
            }
        }
    }
}
```

---

## Open Questions

1. **Script Storage**: Should scripts be stored in-memory or always on disk?
2. **Multi-Project**: Support multiple projects open at once?
3. **Collaboration**: Any plans for real-time collaboration?
4. **Theming**: Light mode support or dark-only?

---

## Related Documents

- [FRONTEND_ARCHITECTURE.md](../docs/specs/FRONTEND_ARCHITECTURE.md) - Original "Raycast for Video" vision
- [SDK_ARCHITECTURE.md](../docs/specs/SDK_ARCHITECTURE.md) - API layer design
- [roadmap.md](../docs/architecture/roadmap.md) - Overall project roadmap
