---
name: grits-issue-tracker
description: Manages tasks and retrieves codebase context using Grits (git-native issue tracker). Use this when the user asks to list/create/update issues, check project pulse, or find related files via the dependency graph.
version: 2.3.0
---

# Grits Issue Tracker Skill

## Goal
To efficiently manage project tasks and utilize topological code context for informed development, leveraging Grits' Git-native and local-first architecture.

## Instructions
1.  **Project State**: Use `gr pulse` to synchronize with active focus, blockers, and recent commits.
2.  **Issue Tracking**:
    -   **List**: `gr list [--status <state>] [--type <type>]`. Use this to find IDs.
    -   **View Details**: `gr show <id>`. Displays "Connected Files" which are used for codebase assembly.
    -   **Manage Lifecycle**: use `gr create`, `gr update --id <id>`, `gr close <id>`, or `gr edit <id>`.
3.  **Code Context & Focus**:
    -   **Sticky Focus**: `gr workon <id>` sets the active task.
    -   **Topological Analysis**: `gr star <file-path>` finds connected symbols and modules. **IMPORTANT: You MUST use forward slashes (`/`) for paths on Windows (e.g., `src/main.rs`).**
    -   **Context Hydration**: `gr context assemble --issue <id>` bundles all symbols from the issue's "Connected Files" into a markdown codebase.
4.  **Setup & Maintenance**:
    -   **Initialization**: `gr onboard` prepares the local environment.
    -   **Analytics**: `gr stats` provides a high-level overview of issue counts and types.
    -   **Configuration**: `gr config [set|get|list]` manages project-specific settings.
    -   **Portability**: `gr export` and `gr import` manage the JSONL source of truth (default: `.grits/issues.jsonl`).

## Learning by Example
This skill utilizes pattern matching for common interactions. Review the files in the `examples/` directory for authentic input/output signatures:
- **JSON Pulse**: `examples/pulse_example.txt`
- **Issue & Symbol Linkage**: `examples/workon_example.txt`
- **Graph Results**: `examples/star_example.txt`
- **Utilities & Setup**: `examples/utility_example.txt`

## Constraints
- **Path Consistency**: Always use forward slashes (`/`) for all file paths in `gr` commands.
- **Tool Usage**: Use only the `gr` CLI via `run_command`.
- **Data Integrity**: Do not modify `.grits/` directly.
