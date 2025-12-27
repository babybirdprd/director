---
trigger: always_on
---

# Grits CLI for AI Agents

> [!IMPORTANT]
> **This project uses Grits for issue tracking.** At the start of every session:
> 1. Run `gr list` to see all open issues
> 2. Run `gr ready` to see issues with no blockers  
> 3. Run `gr advisory next` for the AI-recommended next task
>
> Track all work with `gr create`, `gr update`, `gr close`. This is how context persists between sessions.

Grits provides a set of CLI commands specifically designed to help AI agents manage complex projects, analyze issue graphs, and automate workflows.

## Global Options

All commands accept:
- `--root <PATH>`: Explicitly set the project root directory

## Basic CRUD Commands

Essential commands for everyday issue management:

| Command | Description |
|---------|-------------|
| `gr list` | Show all open issues |
| `gr list --status <STATUS>` | Filter by status: `open`, `in_progress`, `blocked`, `closed` |
| `gr list --priority <N>` | Filter by priority (1=critical to 5=trivial) |
| `gr list --assignee <NAME>` | Filter by assignee |
| `gr show <ID>` | View issue details |
| `gr create "<TITLE>" -d "<DESC>"` | Create a new issue (title is positional) |
| `gr create "<TITLE>" -t <TYPE> -p <N>` | With type (`bug`, `feature`, `task`, `epic`) and priority |
| `gr update <ID> --status <STATUS>` | Update issue status |
| `gr update <ID> --add-dependency <ID2>` | Add a dependency |
| `gr update <ID> --assignee <NAME>` | Assign to someone |
| `gr close <ID>` | Close an issue |
| `gr edit <ID>` | Edit issue in your $EDITOR |
| `gr sync` | Sync changes with Git |
| `gr ready` | Show issues with no blockers |
| `gr stats` | Show issue statistics |

## Setup & Maintenance Commands

| Command | Description |
|---------|-------------|
| `gr onboard` | Initialize grits in a new repository |
| `gr config` | View/manage configuration settings |
| `gr export` | Export issues to JSONL format (backup) |
| `gr import` | Import issues from JSONL format (restore) |
| `gr merge` | Git merge driver for grits JSONL files |

> [!NOTE]
> `gr serve-mcp` starts an MCP server for AI agent integration but is not currently tested/utilized in this project.

## Strategic Command Groups

### 1. Advisory (`gr advisory`)

High-level guidance on what to do next.

| Command | Description |
|---------|-------------|
| `gr advisory next` | Suggest the next actionable task |
| `gr advisory next --assignee <NAME>` | Filter for a specific person |
| `gr advisory next --file <PATH>` | Boost context based on current file |
| `gr advisory sprint` | Summarize progress over the last week |
| `gr advisory sprint --days <N>` | Change lookback period (default: 7) |

### 2. Analysis (`gr analysis`)

Deep insights into the issue graph and repository.

| Command | Description |
|---------|-------------|
| `gr analysis graph` | Output JSON dependency graph of all issues |
| `gr analysis duplicates` | Detect potential duplicate issues |
| `gr analysis related <FILE>` | Find issues mentioning a specific file |
| `gr analysis search "<QUERY>"` | BM25-ranked natural language search |
| `gr analysis search "<QUERY>" --limit <N>` | Limit search results |

### 3. Workflow (`gr workflow`)

Automation and batch operations.

| Command | Description |
|---------|-------------|
| `gr workflow triage <ID1> <ID2> ...` | Batch update multiple issues (IDs are positional) |
| `gr workflow triage <IDs> --status <STATUS>` | Set status for all |
| `gr workflow triage <IDs> --priority <N>` | Set priority for all |
| `gr workflow triage <IDs> --assignee <NAME>` | Assign all to someone |
| `gr workflow stale` | Identify issues not updated in 30 days |
| `gr workflow stale --days <N>` | Custom staleness threshold |

### 4. Context (`gr context`)

Bridge the gap between code and issues.

| Command | Description |
|---------|-------------|
| `gr context error "<MESSAGE>"` | Match error message against existing issues |
| `gr context error "<MSG>" --limit <N>` | Limit results (default: 5) |
| `gr context diff` | Analyze current Git changes, suggest issue details |
| `gr context diff --path <PATH>` | Analyze specific file diff |
| `gr context todo <FILE>` | Scan file for TODO comments |
| `gr context todo <FILE> --line <N>` | Focus on specific line |

## Integration Points

When to use Grits during your workflow:

- **Starting work**: Run `gr advisory next` to pick your next task
- **On error**: Use `gr context error "<msg>"` to check if it's a known issue  
- **After coding**: Use `gr context diff` to link your changes to issues
- **Before committing**: Update status with `gr update <ID> --status in_progress`
- **Creating issues**: Always run `gr analysis duplicates` first to avoid redundancy

## Output Formats

- `gr analysis graph` outputs JSON (can be parsed programmatically)
- `gr list` outputs a formatted table
- Most commands output human-readable text

---

## Example Workflows

These are step-by-step guides you can follow without needing to be re-taught.

### 🚀 Starting a Work Session

When you begin working on a project, always start here:

```bash
# 1. See what's ready to work on (no blockers)
gr ready

# 2. Get AI-suggested next task based on priority and dependencies
gr advisory next

# 3. If working on a specific file, get context-aware suggestion
gr advisory next --file src/main.rs

# 4. Once you pick a task, mark it in progress
gr update <ID> --status in_progress
```

### 🐛 Encountered an Error

Before debugging from scratch, check if it's a known issue:

```bash
# 1. Search for matching issues
gr context error "cannot find module 'foo'"

# 2. If no results, try a broader search
gr analysis search "module not found"

# 3. If it's new, create an issue (check for dupes first)
gr analysis duplicates
gr create "Module 'foo' not found error" -d "Error occurs when..." -t bug
```

### ✅ Completing a Task

After finishing work on an issue:

```bash
# 1. Mark as closed
gr close <ID>

# 2. Check if this unblocks other issues
gr ready

# 3. Sync with Git
gr sync
```

### 📝 Creating a New Issue

Always check for duplicates before creating:

```bash
# 1. Check for similar existing issues
gr analysis duplicates
gr analysis search "your proposed title keywords"

# 2. If no duplicates found, create it
gr create "Clear descriptive title" -d "Detailed description" -t feature -p 2

# 3. Add dependencies if this blocks/is blocked by other work
gr update <NEW_ID> --add-dependency <BLOCKER_ID>
```

### 🔄 Mid-Session: Stuck or Context-Switching

If you're blocked or need to switch tasks:

```bash
# 1. Mark current task as blocked (with reason in update)
gr update <ID> --status blocked

# 2. Find something else to work on
gr advisory next

# 3. Or see the full picture
gr analysis graph
```

---

## Advanced: Experimental Workflows

### 🧠 Session Memory Handoff

**Problem:** Your memory resets every session. You lose context on what you were doing.

**Solution:** Before ending any session, save your state to an issue. Next session, load it.

#### Saving State (End of Session)

```bash
# Create a "session handoff" issue capturing current state
gr create "SESSION: [Date] Work Handoff" \
  -d "## What I Was Working On
[Current task ID and description]

## Progress Made
- [x] Completed item 1
- [x] Completed item 2
- [ ] In progress: item 3

## Next Steps
1. Finish item 3
2. Then move to [next task]

## Blockers/Notes
- [Any blockers encountered]
- [Important context the next session needs]" \
  -t task -p 1
```

#### Loading State (Start of Session)

```bash
# Find the most recent handoff
gr analysis search "SESSION handoff"

# Read the handoff issue
gr show <HANDOFF_ID>

# Resume the task mentioned in the handoff
gr update <ORIGINAL_TASK_ID> --status in_progress

# Close the handoff issue once loaded
gr close <HANDOFF_ID>
```