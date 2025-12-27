# Frontend Architecture: Raycast for Video

> **Status:** Draft  
> **Author:** babybirdprd  
> **Created:** 2024-12-27  
> **Updated:** 2024-12-27  
> **Epic:** gr-b3ib8w

## Summary

The Director Frontend is a **lightweight launcher** following the "Raycast for Video" philosophy: a minimal entry point that dynamically loads heavy capabilities on demand. It's designed as an **extensible platform** for future "apps" and potential plugin systems.

---

## Philosophy: Raycast for Video

**Raycast model:**
- Tiny always-running process
- Instant keyboard trigger
- Extensions add capabilities without bloating core
- Heavy operations lazy-loaded

**Applied to Director:**
- Launcher uses <50MB RAM at idle
- `Alt+Space` (or similar) triggers menu
- "Apps" (Recorder, Editor, Text-to-Video) are separate capability modules
- Heavy engine (`director-core` + Skia + FFmpeg) loads only when needed

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    director-app (Dioxus)                    │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  Launcher Shell                                     │    │
│  │  - App discovery                                    │    │
│  │  - Settings                                         │    │
│  │  - Keyboard shortcuts                               │    │
│  └─────────────────────────────────────────────────────┘    │
│                           │                                 │
│         ┌─────────────────┼─────────────────┐               │
│         ▼                 ▼                 ▼               │
│  ┌─────────────┐  ┌─────────────┐  ┌────────────────┐       │
│  │ Recorder    │  │ Editor      │  │ Text-to-Video  │       │
│  │ (Light)     │  │ (Heavy)     │  │ (AI-Heavy)     │       │
│  │             │  │             │  │                │       │
│  │ windows-    │  │ director-   │  │ director-sdk   │       │
│  │ capture     │  │ core        │  │ + LLM          │       │
│  └─────────────┘  └─────────────┘  └────────────────┘       │
└─────────────────────────────────────────────────────────────┘
```

---

## Dynamic Loading Strategy

### Option A: Feature Flags (Compile-Time)
```toml
[features]
default = ["launcher"]
launcher = []  # Minimal binary
editor = ["director-core", "director-sdk"]
full = ["editor", "recorder", "ai"]
```

**Pros:** Simple, no runtime complexity  
**Cons:** Multiple binaries, larger download

### Option B: Dynamic Linking (Runtime)
```rust
// Launcher loads engine DLL on demand
static ENGINE: OnceLock<libloading::Library> = OnceLock::new();

fn enter_editor_mode() {
    ENGINE.get_or_init(|| {
        libloading::Library::new("director_engine.dll").unwrap()
    });
}
```

**Pros:** Single binary, true lazy loading  
**Cons:** Platform complexity (DLL paths, versioning)

### Recommended: Hybrid
- Feature flags for distribution variants
- OnceLock for within-binary lazy init
- Future: Consider plugin DLLs for community extensions

---

## App Discovery System

### Built-in Apps (V1)
| App | Weight | Dependencies |
|-----|--------|--------------|
| Recorder | Light | `windows-capture` only |
| Editor | Heavy | Full director-core stack |
| Quick Export | Medium | director-sdk + encoder |

### Future: Plugin Apps
```toml
# ~/.director/apps/my-plugin/manifest.toml
[app]
name = "AI Background Remover"
version = "1.0.0"
entry = "plugin.wasm"  # or .dll
capabilities = ["gpu", "sam3"]
```

---

## UI Framework: Dioxus

**Why Dioxus:**
- Pure Rust (no JS bridge)
- Cross-platform (Desktop, Web, Mobile potential)
- Hot reload for rapid iteration
- Component model familiar to React devs

**Rendering Backend:**
- Desktop: WebKitGTK (Linux), WebView2 (Windows), WKWebView (macOS)
- Consider: Skia-based renderer for true native feel (later)

---

## Modes

### Recorder Mode (Light)
```rust
struct RecorderState {
    capture: WindowsCapture,
    output_path: PathBuf,
    recording: bool,
}
// No director-core loaded
```

### Editor Mode (Heavy)
```rust
struct EditorState {
    director: Arc<Mutex<Director>>,  // Lazy initialized
    sdk: DirectorSdk,
    preview: SkiaSurface,
}
```

### Transition
```rust
fn launch_editor() {
    // Show loading indicator
    spawn(async {
        let director = Director::new(...);  // Heavy load happens here
        // Transition UI to editor
    });
}
```

---

## Implementation Phases

### Phase 1: Launcher Shell
- Dioxus desktop window
- App menu (hardcoded list)
- Settings persistence

### Phase 2: Recorder App
- Screen capture integration
- Recording controls
- File output

### Phase 3: Editor App
- Lazy director-core loading
- Real-time preview
- Rhai script execution

### Phase 4: Plugin System (Post-V1)
- App manifest format
- Plugin discovery
- Sandboxed execution

---

## Open Questions

- [ ] Single binary with feature flags vs separate app downloads?
- [ ] How to handle plugin security (sandboxing)?
- [ ] Cross-platform priority order (Windows first, then?)
- [ ] Tray icon / background process model?

---

## Related Documents

- [SDK_ARCHITECTURE.md](SDK_ARCHITECTURE.md) — The API layer the frontend uses
- [TEMPLATE_STRATEGY.md](TEMPLATE_STRATEGY.md) — Template system the editor exposes
