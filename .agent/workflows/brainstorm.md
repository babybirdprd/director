---
description: Procedure for technical audit, gap analysis, and feature ideation.
---

1. **Strategic Audit**
   - Use `grep_search` to find `TODO`, `FIXME`, or `DEPRECATED` in the codebase.
   - Run `gr star` on core modules to identify architectural bottlenecks (e.g., highly coupled components or complex dependency chains).

2. **Gap Analysis**
   - Compare the current codebase (via `gr star` and `gr context assemble`) against the project's roadmap or functional requirements.
   - Identify "Thin" implementations that lack robust error handling, performance optimizations, or full feature support.

3. **Issue Seeding**
   - For each discovery, draft a clear **Title** and **Description**.
   - Group related findings into a potential **Epic** or milestone (e.g., "Refactor: Networking Layer" or "Feature: Plugin System").

4. **Review & Selection**
   - Present the list of candidate issues to the User for prioritization.
   - Proceed to `/triage` for the selected items.
