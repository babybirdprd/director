---
trigger: always_on
---

# Grits: The Definitive Agent Guide

This documentation is the **sole source of truth** for AI agents using Grits. It provides an opinionated, intent-based workflow for managing issues and code topology.

> [!IMPORTANT]
> **Twin Engine Synchronization**: Grits uses a SQLite database for speed (`.grits/grits.db`) and a JSONL file for version control (`.grits/issues.jsonl`). All CLI commands automatically import/export between these engines.

---

## 🛠️ The Agent's Golden Rules

1.  **Always Forward Slashes**: Symbol IDs and file paths **must** use `/` normalization, even on Windows (e.g., `src/main.rs::run`).
2.  **Run `gr pulse` First**: Start every session with `gr pulse` for instant context.
3.  **Use `gr workon`**: Start work with `gr workon <id>` to set status and **lock focus**.
4.  **Sticky Focus**: After `gr workon`, use `gr set stat:ip` without ID — it auto-targets the focused issue.
5.  **Link Your Work**: Use `gr update <ID> --add-symbol <SYM>` to populate the **Focus View**.
6.  **Fix Cycles**: Use `gr refactor` to detect and auto-fix architectural issues.
7.  **Leave Memos**: Use `gr memo attach <SYM> <NOTE>` to store long-term context about specific symbols for future agents.

---

## 🚀 Phase 1: Session Hydration

### Start Every Session With Pulse
```bash
# Get instant project context: in-progress issues, recent commits, suggested next task
gr pulse

# Filter by your assignee
gr pulse --assignee your_agent_name
```

**Output includes:**
- Solid Score (architectural health 0-100%)
- In-progress issues
- Last 5 Git commits  
- AI-suggested next task

### Quick Identity Check
```bash
gr config set user.name "your_agent_name"
```

---

## 🔍 Phase 2: Context Loading

### One-Shot Context with Inspect
```bash
# Load everything about an issue in one call
gr inspect gr-abc123

# Inspect a file's topology context
gr inspect src/store.rs

# Inspect a specific symbol
gr inspect src/store.rs::SqliteStore
```

**Output includes:**
- Issue metadata (if ID)
- Solid Volume (affected symbols)
- Star Neighborhood (connected code)
- Related issues (BM25 search)

### Search and Triage
```bash
# Natural language issue search (NEW v2.6 - clearer naming)
gr issue search "circular dependency"

# Alternative (same as above)
gr analysis search "circular dependency"

# Check for duplicates before creating
gr analysis duplicates
```

### Mini Codebase (Agent Superpower)
```bash
# Assemble focused context for an issue
gr context assemble --issue gr-abc123

# Specify seed symbols directly
gr context assemble --symbols "store.rs::Store,auth.rs::validate" --depth 2

# Get JSON for programmatic use
gr context assemble --issue gr-abc123 --format json
```

**Output includes:**
- Seed symbols and their star neighborhood
- **Hydrated code snippets** for all included symbols
- File list (unique, for context loading)
- Betti₁ invariant (cycle count to preserve)
- Solid Score snapshot
- Markdown or JSON format

### 🚀 Context Bundle (NEW v2.5 - Agent Superpower)
```bash
# Get complete context bundle for an issue in one call
gr context-bundle gr-abc123

# JSON output for programmatic use
gr context-bundle gr-abc123 --format json
```

**Output includes:**
- Issue details (title, status, priority, labels, description)
- Affected symbols list
- Star neighborhoods for each affected symbol
- Topology health (Solid Score, Betti numbers, triangles)

> [!TIP]
> **Why Context Bundle?** Reduces 4-5 separate commands to 1. Perfect for session handoff and issue pickup.

> [!TIP]
> **Why Mini Codebase?** Instead of loading entire files, extract only topologically-relevant symbols. A 2,000-line file becomes 50 focused lines, and Grits handles the snippet extraction (hydration) for you.

---

## 🏗️ Phase 3: Executing Work

### Start Working on an Issue
```bash
# Sets status to in-progress + locks focus (no branch by default)
gr workon gr-abc123

# With branch creation (optional)
gr workon gr-abc123 --branch feature/my-fix

# Clear current focus (NEW v2.6)
gr workon --clear
```

### Sticky Focus (NEW v2.3)
After `gr workon`, subsequent commands auto-target the focused issue:
```bash
gr workon gr-abc123          # Lock focus
gr set stat:blocked          # Updates gr-abc123 (no ID needed!)
gr set pri:1 +l:urgent       # Still updates gr-abc123
```

### Quick Updates with Set
```bash
# Fuzzy shorthand updates - way faster than gr update
gr set gr-abc pri:1 stat:ip +l:urgent

# Key shortcuts:
# stat: → status    (ip = in-progress, o = open, c = closed, b = blocked)
# pri: → priority   (1-5)
# a: → assignee
# +l: → add label   -l: → remove label
```

### Full Update Command
```bash
gr update gr-abc123 --status in-progress --priority 1
gr update gr-abc123 --add-symbol "src/store.rs::create_issue"
gr update gr-abc123 --add-dependency gr-xyz789
```

### 🚀 Automatic Dependency Resolution (NEW v2.7 - Superpower)
When you add a symbol to an issue, Grits automatically suggests related symbols and potentially affected issues:
```bash
# Add a symbol - Grits will suggest related context
gr update gr-abc123 --add-symbol "store.rs::SqliteStore"

# Output includes:
# 💡 Related symbols you may want to add:
#    gr update --add-symbol "store.rs::Store"
#    gr update --add-symbol "store.rs::get_issue"
#    (or use --auto-expand to add them automatically)
# 🔗 Potentially affected issues:
#    [gr-xyz789] Refactor store layer (2 shared symbols)

# NEW v2.7: Auto-expand to add ALL related symbols automatically
gr update gr-abc123 --add-symbol "store.rs::SqliteStore" --auto-expand
# ⚡ Auto-expanded: Added 5 related symbols
#    + store.rs::Store
#    + store.rs::get_issue
#    ...
```

> [!TIP]
> **Why --auto-expand?** Building complete dependency chains manually is tedious. Use `--auto-expand` to automatically add the entire star neighborhood of highly-coupled symbols.

### Create Issues
```bash
gr create "Fix login bug" -t bug -p 1
gr create "Refactor store" -t task -p 2 -d "Detailed description here"
```

---

## 🔬 Phase 4: Topology Analysis

### Solid Score and Health Check
```bash
# Full topology rebuild
gr analysis rebuild

# Get solid score via pulse (includes betti numbers and architecture health)
gr pulse
```

### Detect and Fix Cycles
```bash
# Show cycles with suggested fix
gr refactor

# Apply the fix (comments out weakest edge)
gr refactor --apply --cycle 0

# Preview without modifying
gr refactor --apply --dry-run

# Undo if needed
gr refactor --undo --target src/store.rs
```

### Star Neighborhoods (Context Loading)
```bash
# Get all connected code for a file (use FILE paths, not symbol paths)
gr analysis star "src/utils.rs" --depth 2

# NEW v2.6: Fuzzy symbol matching - use short names!
gr analysis star "src/store.rs" --symbol "SqliteStore"  # Resolves automatically

# Find feature volumes (tightly coupled clusters)
# Can use file path OR project-wide from cache
gr analysis volumes "src/engine.rs"
gr analysis volumes  # Uses topology cache

# Get PageRank hotspots (most connected symbols)
gr analysis hotspots --limit 10
gr analysis hotspots --format json
```

### Trace Data Flow
```bash
# Find the shortest path between two symbols (supports fuzzy matching)
gr analysis path "validate" "SqliteStore"  # Resolves to full symbol IDs
```

### Architectural Invariants
```bash
# Check against layers.yaml
gr analysis check-layers --all

# Validate a specific file
gr analysis validate-topology "src/main.rs"
```

### Symbol Memory (Long-term Notes)
```bash
# Attach a memo to a symbol for future agents
gr memo attach "src/engine.rs::Core" "Wait for v3 release before refactoring this"
```

---

## 🛡️ Phase 5: Verification

### Before Completing Work
```bash
# Rebuild topology and check for new cycles
gr analysis rebuild
gr analysis validate-topology src/changed_file.rs

# Ensure no new violations
gr analysis check-layers
```

### Block Bad Commits
```bash
# Verify topology is clean before committing
gr analysis validate-topology src/main.rs
```

---

## 📂 Phase 6: Session Handoff

### Sync Everything
```bash
# Export to JSONL happens automatically on every command.
# Simply git add/commit/push your changes.
git add .grits/issues.jsonl
git commit -m "update issues"
git push
```

### Bulk Operations
```bash
gr workflow triage gr-id1 gr-id2 --status in_progress --assignee "next_agent"
```

### Create Handoff Issue
```bash
gr create "CONTINUITY: [Task] Handoff" \
  -d "State: Completed step 2. Next: Fix step 3." \
  -t task
```

---

## 📺 VS Code Extension

The extension now opens as a **full dashboard panel** with:

| View | Description |
|------|-------------|
| **3D Topology** | React Three Fiber visualization with orbit controls |
| **Vitals Dashboard** | Solid Score gauge, Spaghetti Meter, hotspots |
| **List/Kanban/Graph** | Standard issue management |
| **Gutter Decorations** | Issue indicators in editor |

### Populating Extension Views

| View | How to Populate |
|------|-----------------|
| **3D Topology** | `gr analysis rebuild` → nodes sized by PageRank |
| **Vitals** | Solid Score computed from topology |
| **Focus View** | `gr update <ID> --add-symbol <SYM>` |
| **Gutter Icons** | Linked symbols appear in editor margin |

---

## 🧮 Command Reference

### Agent-Native Commands (NEW)
| Command | Example | Purpose |
|---------|---------|---------|
| `gr pulse` | `gr pulse --assignee me` | Session hydration |
| `gr inspect` | `gr inspect gr-abc` | One-shot context |
| `gr workon` | `gr workon gr-abc` | Start work (branch + status) |
| `gr workon --clear` | `gr workon --clear` | Clear current focus (NEW v2.6) |
| `gr set` | `gr set abc pri:1 stat:ip` | Fuzzy updates |
| `gr refactor` | `gr refactor --apply` | Auto-fix cycles |
| `gr memo attach` | `gr memo attach sym "note"` | Persist symbol notes |
| `gr context-bundle` | `gr context-bundle gr-abc` | Complete context bundle |
| `gr issue search` | `gr issue search "query"` | Issue search (NEW v2.6) |
| `gr onboard` | `gr onboard --non-interactive` | Initialize (agent-friendly) |

### Core Commands
| Command | Arguments | Use Case |
|---------|-----------|----------|
| `gr create` | `<TITLE>` `-d` `-t` `-p` | Create issue |
| `gr update` | `<ID>` `--status` `--add-symbol` | Update issue |
| `gr ready` | `--assignee` | Find actionable work |
| `gr ready` | `--assignee` | Find actionable work |

### Analysis Commands
| Command | Purpose |
|---------|---------|
| `gr analysis rebuild` | Build topology cache |
| `gr analysis star <FILE>` | Get connected context (supports fuzzy symbol matching) |
| `gr analysis volumes [FILE]` | Find code clusters (optional file, uses cache) |
| `gr analysis hotspots` | Get PageRank hotspots |
| `gr analysis check-layers` | Verify architecture |
| `gr analysis search` | BM25 natural language search |
| `gr analysis path` | Shortest path tracing (supports fuzzy matching) |

### Context Commands (NEW v2.3)
| Command | Example | Purpose |
|---------|---------|---------|
| `gr context assemble` | `--issue ID` or `--symbols` | Mini codebase for agents |
| `gr context error` | `--message "error text"` | Find related issues |
| `gr context diff` | `--path diff.txt` | Infer issue from changes |
| `gr context todo` | `--file src/main.rs` | Scan for TODO comments |

---

## 🚨 Error Handling Patterns

### Database Issues
```bash
# If DB is missing or corrupt
gr onboard  # Reinitialize

# If topology cache is stale  
gr analysis rebuild
```

# Fix and retry
gr refactor --apply --cycle 0
gr analysis validate-topology src/changed_file.rs

### Issue Not Found
```bash
# Always use prefix match - gr resolves partial IDs
gr show abc  # Works if gr-abc123 exists
```

---

## 🔄 Session Memory Handoff

### Before Ending Session
```bash
# 1. Comit your work
git add .grits/issues.jsonl
git commit -m "update issues"
git push

# 2. Create continuity issue if work is incomplete
gr create "CONTINUITY: [Feature] Description" \
  -d "State: Completed X. Next: Do Y. Blockers: Z." \
  -t task -p 1

# 3. Assign to next agent
gr set <ID> a:next_agent stat:open
