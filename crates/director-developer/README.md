# director-developer

**Spec-Driven Development harness for the Director video engine.**

`director-dev` is a CLI tool that introspects the Director workspace to extract verified, ground-truth system information. It enables AI agents and developers to work with accurate knowledge of available types, APIs, and capabilities.

## Features

- **Schema Reflection** - Extract JSON Schemas from `director-schema` types
- **API Introspection** - List all registered Rhai scripting functions
- **Pipeline Capabilities** - Query available nodes, effects, and constraints
- **Spec Validation** - Validate feature specs against current system state
- **Context Generation** - Create markdown context files for AI agents
- **Watch Mode** - Auto-regenerate context on spec file changes

## Installation

The crate is part of the Director workspace. Build with:

```bash
cargo build -p director-developer
```

## Quick Start

```bash
# Show system information
director-dev info

# List available types
director-dev list types

# List Rhai functions (with filter)
director-dev list functions --filter "animate"

# Validate a feature spec
director-dev validate --spec specs/my_feature.ron

# Generate context for AI agents
director-dev generate --spec specs/my_feature.ron --output CONTEXT.md

# Dump all system truth
director-dev dump --output .director-dev

# Generate AI prompt from template
director-dev prompt implement_effect --var EFFECT_NAME=Glow

# Show dependency graph
director-dev graph
director-dev graph --impact director-core
director-dev graph --format mermaid
```

## Commands

### `info`

Display a summary of the Director engine's capabilities:

```
Director Developer - System Information
=======================================

Rhai Scripting API:
  • Functions: 58
  • Modules: 1

Schema Types: 14

Pipeline Capabilities:
  • Node types: 8 (Box, Text, Image, Video, Vector, Lottie, Effect, Composition)
  • Effects: 7 (Blur, DropShadow, ColorMatrix, Grayscale, Sepia, DirectionalBlur, FilmGrain)
  • Transitions: 6 (Fade, SlideLeft, SlideRight, WipeLeft, WipeRight, CircleOpen)
```

### `list <what>`

List available resources:

| What | Description |
|------|-------------|
| `types` | Schema types with JsonSchema support |
| `functions` | Registered Rhai functions |
| `nodes` | Available node types with animatable properties |
| `effects` | Visual effects |

Use `--filter` to narrow results:

```bash
director-dev list functions --filter "add_"
```

### `validate --spec <path>`

Validate a RON feature spec against the current system:

```bash
director-dev validate --spec specs/glow_effect.ron
```

Output:
```
Validating spec: Glow Effect
  Priority: 2
  User Story: As a video creator, I want to add a glow effect...

Checking related types:
  ✓ Type 'EffectConfig'
  ✓ Type 'Node'
  ✓ Type 'StyleMap'

Checking related functions:
  ✓ Pattern 'effect' - 3 match(es)
  ✓ Pattern 'blur' - 2 match(es)

✓ Validation complete: All checks passed for 'Glow Effect'
```

### `generate --spec <path> --output <path>`

Generate a `CURRENT_CONTEXT.md` file containing:

- User story and priority
- Full JSON Schemas for related types
- Matching Rhai function signatures
- Pipeline capabilities table
- Proposed changes summary
- Verification checklist

### `dump --output <dir>`

Export all system truth to JSON files:

```bash
director-dev dump --output .director-dev
```

Creates:
- `rhai_api.json` - All Rhai function metadata
- `schemas.json` - JSON Schemas for all types
- `pipeline_capabilities.json` - Node/effect/transition info

### `watch --dir <path> --output <path>`

Watch for `.ron` spec file changes and auto-regenerate contexts:

```bash
director-dev watch --dir specs --output .director-dev/contexts
```

### `new <name> --template <type> --output <dir>`

Create a new feature spec from a template:

```bash
# Create an effect spec
director-dev new "Glow Effect" --template effect --output specs

# Create a node spec
director-dev new "Particle Emitter" --template node --output specs

# Create an animation spec
director-dev new "Bounce Animation" --template animation --output specs

# Create a generic API spec
director-dev new "Media Query" --template api --output specs
```

Available templates:
| Template | Description |
|----------|-------------|
| `effect` | Visual effect (blur, glow, shadow, etc.) |
| `node` | New node type (like Box, Text, Image) |
| `animation` | Animation or timing feature |
| `api` | Generic scripting API function |

### `prompt <template> --var KEY=VALUE`

Generate an AI prompt from a template:

```bash
# List available templates
director-dev prompt --list

# Generate a prompt with variables
director-dev prompt implement_effect --var EFFECT_NAME=Glow

# Save to file
director-dev prompt implement_node --var NODE_NAME=Particle -o prompt.md
```

Available templates:
| Template | Description |
|----------|-------------|
| `implement_effect` | Implement a new visual effect |
| `implement_node` | Implement a new node type |
| `add_rhai_function` | Add a new Rhai function |
| `debug_animation` | Debug animation issues |
| `extend_schema` | Add or modify schema types |

### `graph`

Show workspace dependency graph:

```bash
# List all crates with dependency counts
director-dev graph

# Show crates affected by changes
director-dev graph --impact director-core

# Show dependencies of a crate
director-dev graph --deps director-pipeline

# Output as Mermaid diagram
director-dev graph --format mermaid
```

Example output:
```
Workspace Crates:
=================
Name                            deps |       dependents
--------------------------------------------------
director-cli                  1 deps |     0 dependents
director-core                 3 deps |     5 dependents
director-developer            3 deps |     0 dependents
```

## Feature Spec Format (RON)

Feature specs use [RON](https://github.com/ron-rs/ron) format:

```ron
FeatureSpec(
    title: "My Feature",
    user_story: "As a user, I want...",
    priority: 2,
    
    // Types to include in context (must have JsonSchema)
    related_types: ["Node", "EffectConfig"],
    
    // Function patterns to search for
    related_functions: ["add_", "animate"],
    
    // Proposed changes
    schema_changes: [
        SchemaChange(
            target: "EffectConfig",
            change: add_variant(
                name: "Glow",
                fields: [("color", "Color"), ("radius", "f32")],
            ),
        ),
    ],
    
    scripting_requirements: [
        ScriptingRequirement(
            function_name: "add_glow",
            signature: "add_glow(node_id, color, radius)",
        ),
    ],
    
    pipeline_requirements: [
        PipelineRequirement(
            description: "Implement glow effect shader",
            affected_area: Some("director-core/src/effects/"),
        ),
    ],
    
    verification: VerificationSpec(
        script_compiles: true,
        schema_validates: true,
        custom_scripts: ["examples/test_glow.rhai"],
        test_cases: ["Glow renders correctly"],
    ),
)
```

## Architecture

### Reflectors

Three reflection strategies extract system truth:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Scripting      │    │  Schema         │    │  Pipeline       │
│  Reflector      │    │  Reflector      │    │  Reflector      │
├─────────────────┤    ├─────────────────┤    ├─────────────────┤
│ Rhai Engine     │    │ schemars        │    │ Static Manifest │
│ metadata feature│    │ JsonSchema      │    │ Node/Effect/    │
│ gen_fn_metadata │    │ derive macro    │    │ Transition list │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Output Flow

```
Feature Spec (.ron)
        ↓
   ┌────────────┐
   │ Validator  │ ← Reflectors
   └────────────┘
        ↓
   ┌─────────────┐
   │ Synthesizer │ ← Reflectors
   └─────────────┘
        ↓
CURRENT_CONTEXT.md
```

## Development

### Adding New Types

To make a type available for schema reflection:

1. Add `schemars` dependency to the crate
2. Add `#[derive(JsonSchema)]` to the type
3. Import `use schemars::JsonSchema;`
4. Add the type to `reflectors/schema.rs`

### Adding New Functions

Rhai functions are automatically discovered via the `metadata` feature. Simply register them in `director-core/src/scripting/`.

### Adding New Crates (Future Extensions)

When adding new crates like `director-tts`, `director-ai`, etc., follow these guidelines:

**Why not pre-add support?**
- **YAGNI** - The crate might not be needed, or its API may differ from expectations
- **Maintenance burden** - Empty stubs become lies when the real crate differs
- **Current architecture scales** - New crates integrate by following the same pattern

**Integration steps for a new crate:**

1. **Add `schemars` to the new crate's dependencies**
   ```toml
   schemars = { workspace = true }
   ```

2. **Add `#[derive(JsonSchema)]` to public types**
   ```rust
   use schemars::JsonSchema;
   
   #[derive(Serialize, Deserialize, JsonSchema)]
   pub struct TtsConfig { ... }
   ```

3. **Optionally create a dedicated reflector** (if unique introspection is needed)
   ```
   src/reflectors/tts.rs  # For crate-specific reflection logic
   ```

4. **Register types in `reflectors/schema.rs`**
   ```rust
   "TtsConfig" => Ok(serde_json::to_string_pretty(&schema_for!(TtsConfig))?),
   ```

The plugin-friendly architecture means new crates integrate naturally without modifying the core tool.

## License

MIT - See [LICENSE](../../LICENSE) for details.
