---
description: Procedure for discovering technical context for issues without pre-linked files.
---

1. **Hydrate & Pulse**
   - Run `gr pulse` to check the current issue status.
   - If `affected_symbols` is empty, proceed to Discovery.

2. **Topological Discovery**
   - Run `gr star` to see the neighborhood of the issue's focus (if any).
   - Use `grep_search` to find relevant keywords in the codebase if no focus is set.
   - Use `gr star <FILE>` or `view_file` on discovered files to identify key implementation points.

3. **Symbol Enrichment**
   - Once relevant files are identified, run `gr update --scan-file <PATH>` for each file.
   - This populates the "Affected Symbols" and provides the Coder agent with direct context.

4. **Planning (The "Rich Context")**
   - Draft a `design` with atomic chunks using `gr update --design "..."`.
   - Define `acceptance-criteria` with specific test commands using `gr update --acceptance-criteria "..."`.

5. **Validation**
   - Run `gr pulse` again to verify that the "Rich Context" is correctly rendered and includes the necessary files and symbols.
