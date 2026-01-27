---
description: High-fidelity orchestration for researching and planning complex technical tasks.
---

1. **Task Selection & Focus**
   - Run `gr list --format json` to triage open work.
   - Select the highest priority issue (Priority 1) that is unblocked.
   - **Sticky Focus**: Run `gr workon <ID>` to lock your workspace.
   - **Context Check**: Run `gr pulse`. If `affected_symbols` (Connected Files) is empty, **STOP**. Switch to [.agent/workflows/discovery-workflow.md](.agent/workflows/discovery-workflow.md) immediately.

2. **Topological Research (Architecture Audit)**
   - // turbo
     Run `gr star` and `gr update --scan-file` to map the source dependencies.
   - **Conflict Check**: Identify if these changes intersect with other in-progress issues (`gr pulse`).
   - **Invariant Audit**: Check `docs/architecture/` (if exists) or code layers. Ensure your plan won't introduce circular dependencies.

3. **Dependency and Environment Check**
   - Verify if new crates, libraries, or system tools are required.
   - Explicitly note any environment variables or config changes needed for the rollout.

4. **Technical Strategy (The Design)**
   - Draft a step-by-step implementation strategy.
   - **Atomic Breakdown**: Divide the work into testable chunks (max 3-5 symbols per chunk).
   - Run `gr update --design "[Technical Plan with Atomic Chunks]"`

5. **Success Criteria (Acceptance Proofs)**
   - Define exact shell commands for verification (e.g., `cargo test path::to::test`).
   - Run `gr update --acceptance-criteria "1. Command X passes\n2. Log Y shows message Z..."`

6. **Handoff Validation**
   - // turbo
     Run `gr pulse` to ensure the "Rich Context" is comprehensive.
   - **Safety Handoff**: Connect relevant files to the issue: `gr update --scan-file <PATH>`.
