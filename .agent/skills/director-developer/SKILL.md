---
name: director-developer
description: Use the director-developer CLI to introspect the Director engine for Spec-Driven Development. Run this tool to get accurate, ground-truth information about available types, APIs, and capabilities.
---

# Director-Developer Skill

This skill enables AI agents to work with **verified system truth** about the Director engine. Instead of relying on potentially outdated documentation, use `director-dev` to introspect the actual codebase.

## When to Use This Skill

- When implementing new features across multiple Director layers
- When you need accurate JSON schemas for data types
- When you need the exact Rhai function signatures
- When validating a feature spec before implementation
- When generating context for AI-assisted development

## Available Commands

### Quick Reference

```bash
# Show system overview
director-dev info

# List available types, functions, nodes, or effects
director-dev list types
director-dev list functions --filter "animate"
director-dev list nodes
director-dev list effects

# Create a new feature spec from template
director-dev new "My Feature" --template effect --output specs

# Validate a feature spec
director-dev validate --spec specs/my_feature.ron

# Generate context from a spec
director-dev generate --spec specs/my_feature.ron --output CONTEXT.md

# Dump all system truth to files
director-dev dump --output .director-dev

# Watch for spec changes (interactive mode)
director-dev watch --dir specs --output .director-dev/contexts

# Generate AI prompts from templates
director-dev prompt --list
director-dev prompt implement_effect --var EFFECT_NAME=Glow

# Show workspace dependency graph
director-dev graph
director-dev graph --impact director-core
director-dev graph --format mermaid
```

## Workflow

### 1. Get System Info First

Before implementing a feature, run:

```bash
cargo run -p director-developer -- info
```

This shows:
- Number of registered Rhai functions
- Available schema types
- Supported node types, effects, and transitions
- Animation support matrix

### 2. List Specific Resources

Query for specific types or functions:

```bash
# Find all animation-related functions
cargo run -p director-developer -- list functions --filter "anim"

# Find all schema types
cargo run -p director-developer -- list types
```

### 3. Create a Feature Spec

Use the `new` command to scaffold a spec from a template:

```bash
# Templates: effect, node, animation, api
cargo run -p director-developer -- new "Glow Effect" --template effect --output specs
```

This generates a pre-filled RON spec that you can customize:

```ron
FeatureSpec(
    title: "Glow Effect",
    user_story: "As a video creator, I want to add a glow effect...",
    priority: 2,
    
    related_types: ["EffectConfig", "Node"],
    related_functions: ["effect", "add_"],
    
    schema_changes: [...],
    scripting_requirements: [...],
    pipeline_requirements: [...],
    
    verification: VerificationSpec(...),
)
```

### 4. Validate the Spec

```bash
cargo run -p director-developer -- validate --spec specs/my_feature.ron
```

This checks:
- ✓ All related types exist in the schema
- ✓ All related function patterns have matches
- Reports proposed changes needed

### 5. Generate Context

```bash
cargo run -p director-developer -- generate --spec specs/my_feature.ron
```

This creates a `CURRENT_CONTEXT.md` with:
- Full JSON schemas for related types
- Matching Rhai function signatures
- Pipeline capabilities table
- Proposed changes summary
- Verification checklist

## Output Files

When running `director-dev dump`, three JSON files are generated:

| File | Contents |
|------|----------|
| `rhai_api.json` | All registered Rhai function metadata |
| `schemas.json` | JSON Schemas for all `director-schema` types |
| `pipeline_capabilities.json` | Node types, effects, transitions, constraints |

## Best Practices

1. **Always query before implementing** - Don't assume API shapes, verify them
2. **Use specs for cross-cutting features** - Features touching schema + scripting + pipeline
3. **Regenerate context after engine changes** - Schemas may have changed
4. **Check animation support** - Use `supports_animation(node, property)` logic

## Example: Implementing a Glow Effect

See [examples/glow_effect.ron](file:///d:/rust-skia-engine/crates/director-developer/examples/glow_effect.ron) for a complete feature spec demonstrating:

- Schema changes (adding EffectConfig::Glow variant)
- Scripting requirements (add_glow function)
- Pipeline requirements (implementation notes)
- Verification criteria

## Troubleshooting

**Q: Command output is empty**
A: The CLI uses `println!` for output. Ensure you're running directly, not capturing stderr only.

**Q: Type not found**
A: Check if the type has `#[derive(JsonSchema)]` in `director-schema` or `director-core`.

**Q: Function not found**
A: Check if the function is registered in `director-core/src/scripting/api/`.
