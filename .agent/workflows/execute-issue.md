---
description: Orchestrates the implementation of a planned issue, moving it to completion.
---

// turbo-all

### Phase 0: Baseline Verification (Safety First)
1. **Focus**: Run `gr workon <ID>` to lock your session and set the technical focus.
2. **Hydrate**: Run `gr pulse` (no ID) to read the `design` and `acceptance_criteria`.
3. **Plan Check**: If `design` is empty, **STOP**. Switch to [.agent/workflows/plan-issue.md](.agent/workflows/plan-issue.md) immediately.
4. **Context**: Run `gr context assemble` (no ID) to load the technical environment into your memory.
5. **Audit**: Run the project's default build/test command (e.g., `cargo check` or `npm run build`).
6. **STOP**: If the baseline is broken, do **NOT** attempt the implementation. Fix the base or flag a blocker.

### Phase 1: The Implement-Verify-Log (IVL) Cycle
For each "Atomic Chunk" defined in the `design`:

1. **Implement**: Perform the file modifications for this chunk.
2. **Verify**: Run the *specific* verification commands defined in the `acceptance_criteria` for this chunk.
3. **Self-Correction**: 
   - If tests fail: Debug the failure, fix the code, and **re-verify**.
   - If you cannot fix the failure within 2 iterations: Revert the chunk and flag the issue as `blocked`.
4. **Log**: Record the technical proof to the state store: 
   `gr update --notes "Chunk X: [Implemented/Fixed/Verified]. Test Output: [Success/Details]" --append`

### Phase 2: Final Acceptance Proof
1. **Regression Check**: Run the full project test suite.
2. **Acceptance Proof**: Run the final verification commands from `acceptance_criteria`.
3. **Formal Handoff**: Update the issue to `closed` with a summary of the technical proofs.
   `gr update --status closed --notes "Final: [Summary of verified changes and proof of success]" --append`
