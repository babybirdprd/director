---
description: High-fidelity resolution for Grits issues. Ensures production readiness via deep context analysis, architectural alignment, and rigorous verification.
---

// turbo-all

# Phase 1: Context Hydration & Analysis
1. If no issue ID is provided, run `gr list --status open` and ask the user to pick a target.
2. Run `gr workon <issue-id>` to set the active focus.
3. Run `gr context assemble --issue <issue-id>` to load the primary technical context.
// turbo
4. Run `gr star <primary-file>` for each file identified in the assembly. This ensures you see the *hidden* dependencies and reverse-dependencies that the issue might have missed.
5. Use `grep_search` to check for similar patterns across the codebase to ensure consistency (e.g., how other nodes implement `animate_property`).

# Phase 2: Design & Strategy
1. Open the existing `implementation_plan.md` in the `brain/` directory.
2. Create a new "Implementation Strategy" section for this specific issue.
3. **Critical**: Identify potential breaking changes in `director-schema` or side-effects in `director-core` rendering.
4. Verify if the solution requires new error variants in `crates/director-core/src/errors.rs`.

# Phase 3: Implementation Excellence
1. Implement the fix using idiomatic Rust practices:
   - Use `Result` and `anyhow` for error propagation. No `panic!` or `unwrap!` in production code.
   - Add `tracing` instrumentation (`#[instrument]`) for significant functions.
   - Maintain strict `director-core` vs `director-pipeline` architectural boundaries.
2. If the fix touches `director-schema`, ensure both the Rust structs and the `.json` examples in `tests/` are updated.

# Phase 4: Rigorous Verification
1. **Unit Testing**: Run `cargo test` for the specific crate modified.
2. **Integration Testing**: Run `cargo test --test '*' ` to check for regressions in the engine or pipeline.
3. **Linting**: Run `cargo clippy` and `cargo fmt` to ensure production-grade code quality.
4. **Visual Validation**: If the change affects rendering, create/run a temporary integration test in `crates/director-core/tests/visual/` to verify the output.

# Phase 5: Finalization & Hygiene
1. Update `walkthrough.md` in the `brain/` directory with a "Proof of Work" section (e.g., test results, code snippets).
2. Update the checklist in `task.md`.
3. Run `gr close <issue-id>`.
4. Call `notify_user` with a concise summary of the engineering effort and any remaining technical debt.
