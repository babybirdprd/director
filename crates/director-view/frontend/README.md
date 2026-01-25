# Director View Frontend

A React-based preview interface for the Director video engine.

## Features

- **Live Preview**: See rendered frames in real-time as you scrub the timeline
- **Script Editor**: Monaco-based editor with Rhai syntax highlighting
- **Timeline**: Visual timeline with scene markers and playback controls
- **Keyboard Shortcuts**: Professional editing shortcuts (J/K/L, Space, etc.)

## Getting Started

### Prerequisites

- Bun (or Node.js 18+ with npm)
- Rust backend running (`cargo run -p director-view`)

### Development

```bash
# Install dependencies
bun install

# Start dev server (with hot reload)
bun run dev
```

The frontend runs on `http://localhost:5173` and proxies API requests to the Rust backend on port 3000.

### Production Build

```bash
bun run build
```

This outputs to `../static/dist/` which can be served by the Rust backend.

## Architecture

```
src/
├── api/
│   └── director.ts      # API client for backend communication
├── components/
│   ├── Preview.tsx      # Frame display with zoom controls
│   ├── Timeline.tsx     # Scrubber with scene markers
│   ├── ScriptEditor.tsx # Monaco editor for Rhai scripts
│   └── Controls.tsx     # Playback controls
├── stores/
│   └── project.ts       # Zustand state management
├── styles/
│   └── globals.css      # Tailwind + custom styles
├── App.tsx              # Main layout
└── main.tsx             # Entry point
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Space` | Play/Pause |
| `J` | Rewind 5 seconds |
| `K` | Play/Pause |
| `L` | Forward 5 seconds |
| `,` | Previous frame |
| `.` | Next frame |
| `←` | Back 0.1s (Shift: 1s) |
| `→` | Forward 0.1s (Shift: 1s) |
| `Home` | Go to start |
| `End` | Go to end |
| `Ctrl+Enter` | Run script |

## API Endpoints

The frontend communicates with these backend endpoints:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/init?script_path=...` | GET | Load script from file path |
| `/init` | POST | Load script from content |
| `/render?time=...` | GET | Render frame at time |
| `/scenes` | GET | Get timeline scene info |
| `/file?path=...` | GET | Read file contents |
| `/health` | GET | Health check |

## Future: Tauri Migration

This frontend is designed to be easily wrapped in Tauri for native desktop features:

1. The API client can be swapped for Tauri commands
2. File dialogs can use native OS dialogs
3. Global hotkeys can be registered
4. System tray integration

See [plans/director-view-frontend.md](../../../plans/director-view-frontend.md) for the migration plan.
