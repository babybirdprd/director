# 🤖 Director PlanA: Headless, Git-Native Project Management Kernel for the Age of AI

`director-plan` is a **command-line tool and local server** that turns your filesystem into a high-context Project Management database. It is designed specifically to allow **Human Developers and AI Agents** (Cursor, Radkit, Claude) to collaborate on tasks with zero friction.

## 🚀 Why "Headless" PM?

Traditional tools like Jira or Linear are built for humans clicking buttons in a browser. They are "black boxes" to AI agents.

* **Agents need Context:** They need to read specs, constraints, and relevant code paths instantly.
* **Agents need Verification:** "Done" isn't a checkbox; it's a passing test suite.
* **State matches Code:** If you branch your code to try a feature, your Project Plan should branch with it.

`director-plan` stores tickets as **TOML files** in your repository. It provides a CLI for Agents to read/write state and a Web Dashboard for humans to visualize it.

## 📦 Installation

This tool is part of the `director-engine` workspace.

```bash
# Build and install locally
cargo install --path crates/director-plan
```

## 🛠️ CLI Reference (The Agent API)

Agents (like Cursor/Windsurf) interact with the project via these commands. This ensures they never "hallucinate" file paths or break TOML syntax.

### 1. Discovery

List active tasks in a machine-readable format.

```bash
director-plan list --status todo --format json
```

**Output:**

```json
[
  { "id": "T-001", "title": "Implement Text Shadows", "priority": "high" }
]
```

### 2. Context Loading (The "Prompt")

Generates a massive, context-rich prompt containing the **Ticket Spec**, **Constraints**, and the content of all `relevant_files`.

```bash
director-plan context T-001
```

> Copy-paste this output into your LLM to align it instantly.

### 3. Verification (The "Kill" Feature)

Runs the specific test command defined in the ticket (e.g., visual_regression).

```bash
director-plan verify T-001
```

Returns: **PASS** or **FAIL** (with diff output).

> **Rule:** A ticket cannot move to done unless this command passes.

### 4. Updates

Safely updates ticket state without breaking comments or formatting.

```bash
director-plan update T-001 --status in_progress --owner "agent-claude"
```

### 5. Documentation RAG

Allows agents to search the `docs/` folder for specific technical implementation details.

```bash
director-plan docs search "skparagraph layout"
```

---

## 🖥️ Director Studio (The Human View)

For humans, reading JSON is painful. We provide a local "Director Studio" dashboard.

```bash
director-plan serve
# Listening on http://localhost:3000
```

**Features**

* **Kanban Board:** Drag and drop tickets between Todo / Active / Review / Done.
* **Live Updates:** Changes in the UI write to TOML files instantly.
* **Visual Diffing:** View "Golden Image" vs "Actual Render" for visual regression tasks side-by-side with a slider.
* **Performance Monitoring:** See if a ticket introduced a performance regression.
* **Asset Management:** Drag and drop assets to auto-ingest.
* **Approval Flow:** One-click approval for Agent work that passes verification.

---

## 📂 Data Structure

All data lives in the `plan/` directory at the root of the workspace.

```
plan/
├── tickets/
│   ├── T-001.toml      # Single Source of Truth
│   └── T-002.toml
├── views/
│   └── board.json      # Dashboard config
└── history/
    └── T-001.log       # Append-only agent logs
```

**Ticket Schema Example (T-001.toml)**

```toml
[meta]
id = "T-001"
title = "Implement Text Shadows"
status = "todo"
priority = "high"

[spec]
description = "Text nodes need drop shadows using SkParagraph."
constraints = ["No cosmic-text", "Must use StyleBuilder"]
relevant_files = ["crates/director-core/src/node/text.rs"]

[verification]
command = "cargo test --test visual_regression -- --verify T-001"
golden_image = "tests/snapshots/shadow_golden.png"
```

## 🤖 The "Golden Loop" Workflow

1.  **Human:** Creates `T-001.toml` (via Web UI or file creation) defining the spec and the "Golden Image" requirement.
2.  **Agent:** Runs `director-plan list`, picks up T-001.
3.  **Agent:** Runs `director-plan context T-001`, reads the prompt.
4.  **Agent:** Writes code + runs `director-plan verify T-001`.
5.  **Agent:** Once verified, runs `director-plan update T-001 --status review`.
6.  **Human:** Sees ticket in "Review" column on Dashboard. Checks the Visual Diff. Hits "Approve".

## 🤝 Integration with Cloud Agents

### Cursor / Windsurf

Add this to your `.cursorrules` or System Prompt:

> "You are an Agent using the `director-plan` tool. Always check for active tickets using `director-plan list`. Before writing code, load context via `director-plan context {id}`. You must verify work via `director-plan verify {id}` before submitting."

### Radkit (Autonomous)

The `director-plan` crate exposes a Rust API for embedding radkit agents directly. (See `src/agent.rs` for implementation details).
