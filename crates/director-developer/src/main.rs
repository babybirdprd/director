//! # Director Developer CLI
//!
//! System-Truth Generator for Spec-Driven Development.
//!
//! This tool introspects the Director workspace to extract:
//! - Data contracts (JSON Schema from `director-schema`)
//! - Scripting API (Rhai signatures from `director-core`)
//! - Pipeline capabilities (from `director-pipeline`)
//!
//! ## Commands
//! - `generate`: Create CURRENT_CONTEXT.md from a feature spec
//! - `dump`: Export all system truth to JSON files
//! - `validate`: Check a spec against current capabilities
//! - `watch`: Monitor spec files and regenerate on changes
//! - `list`: List available types and functions
//! - `info`: Show system information summary

mod prompts;
mod reflectors;
mod spec;
mod synthesizer;
mod watch;

#[cfg(test)]
mod tests;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "director-dev")]
#[command(about = "Spec-Driven Development harness for Director")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate context from a feature spec
    Generate {
        /// Path to the feature spec (.ron file)
        #[arg(short, long)]
        spec: PathBuf,

        /// Output path for CURRENT_CONTEXT.md
        #[arg(short, long, default_value = "CURRENT_CONTEXT.md")]
        output: PathBuf,
    },

    /// Dump all system truth (for debugging/auditing)
    Dump {
        /// Output directory for reflection artifacts
        #[arg(short, long, default_value = ".director-dev")]
        output: PathBuf,

        /// Include Rhai API signatures
        #[arg(long, default_value = "true")]
        scripting: bool,

        /// Include JSON schemas
        #[arg(long, default_value = "true")]
        schema: bool,

        /// Include pipeline capabilities
        #[arg(long, default_value = "true")]
        pipeline: bool,
    },

    /// Validate a feature spec against current system capabilities
    Validate {
        /// Path to the feature spec (.ron file)
        #[arg(short, long)]
        spec: PathBuf,
    },

    /// Watch spec files and regenerate context on changes
    Watch {
        /// Directory to watch for .ron spec files
        #[arg(short, long, default_value = "specs")]
        dir: PathBuf,

        /// Output directory for generated contexts
        #[arg(short, long, default_value = ".director-dev/contexts")]
        output: PathBuf,
    },

    /// List available types or functions
    List {
        /// What to list: "types", "functions", "effects", "nodes"
        #[arg(default_value = "types")]
        what: String,

        /// Filter pattern (optional)
        #[arg(short, long)]
        filter: Option<String>,
    },

    /// Create a new feature spec from a template
    New {
        /// Feature name (will become filename)
        name: String,

        /// Template: "effect", "node", "animation", "api"
        #[arg(short, long, default_value = "effect")]
        template: String,

        /// Output directory for the spec
        #[arg(short, long, default_value = "specs")]
        output: PathBuf,
    },

    /// Show system information summary
    Info,

    /// Generate an AI prompt from a template
    Prompt {
        /// Template name (e.g., "implement_effect", "implement_node")
        template: Option<String>,

        /// Variable substitutions in KEY=VALUE format
        #[arg(short, long)]
        var: Vec<String>,

        /// List available templates
        #[arg(long)]
        list: bool,

        /// Output file (defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Show workspace dependency graph
    Graph {
        /// Output format: "list", "mermaid", "json"
        #[arg(short, long, default_value = "list")]
        format: String,

        /// Show impact analysis for a specific crate
        #[arg(short, long)]
        impact: Option<String>,

        /// Show dependencies of a specific crate
        #[arg(short, long)]
        deps: Option<String>,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "director_developer=info".parse().unwrap()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { spec, output } => synthesizer::generate_context(&spec, &output),
        Commands::Dump {
            output,
            scripting,
            schema,
            pipeline,
        } => reflectors::dump_all(&output, scripting, schema, pipeline),
        Commands::Validate { spec } => spec::validate(&spec),
        Commands::Watch { dir, output } => watch::watch_specs(&dir, &output),
        Commands::List { what, filter } => cmd_list(&what, filter.as_deref()),
        Commands::New {
            name,
            template,
            output,
        } => cmd_new(&name, &template, &output),
        Commands::Info => cmd_info(),
        Commands::Prompt {
            template,
            var,
            list,
            output,
        } => cmd_prompt(template.as_deref(), &var, list, output.as_deref()),
        Commands::Graph {
            format,
            impact,
            deps,
        } => cmd_graph(&format, impact.as_deref(), deps.as_deref()),
    }
}

/// List available types, functions, effects, or nodes.
fn cmd_list(what: &str, filter: Option<&str>) -> Result<()> {
    match what {
        "types" => {
            println!("Available Schema Types:");
            println!("========================");
            for type_name in reflectors::schema::list_types() {
                if filter.map_or(true, |f| {
                    type_name.to_lowercase().contains(&f.to_lowercase())
                }) {
                    println!("  • {}", type_name);
                }
            }
        }
        "functions" => {
            let pattern = filter.unwrap_or("");
            let functions = reflectors::scripting::find_functions(pattern)?;
            println!("Rhai Functions matching '{}':", pattern);
            println!("================================");
            for func in &functions {
                let params: Vec<String> = func
                    .params
                    .iter()
                    .map(|p| p.name.clone().unwrap_or_else(|| "_".to_string()))
                    .collect();
                println!("  • {}({})", func.name, params.join(", "));
            }
            println!("\nTotal: {} functions", functions.len());
        }
        "nodes" => {
            println!("Available Node Types:");
            println!("=====================");
            let nodes = [
                "Box",
                "Text",
                "Image",
                "Video",
                "Vector",
                "Lottie",
                "Effect",
                "Composition",
            ];
            for node in nodes {
                if filter.map_or(true, |f| node.to_lowercase().contains(&f.to_lowercase())) {
                    let props = reflectors::pipeline::get_animatable_properties(node);
                    println!("  • {} - animatable: {}", node, props.join(", "));
                }
            }
        }
        "effects" => {
            println!("Available Effects:");
            println!("==================");
            let effects = [
                "Blur",
                "DropShadow",
                "ColorMatrix",
                "Grayscale",
                "Sepia",
                "DirectionalBlur",
                "FilmGrain",
            ];
            for effect in effects {
                if filter.map_or(true, |f| effect.to_lowercase().contains(&f.to_lowercase())) {
                    println!("  • {}", effect);
                }
            }
        }
        _ => {
            println!(
                "Unknown list type: {}. Use: types, functions, nodes, effects",
                what
            );
        }
    }
    Ok(())
}

/// Show system information summary.
fn cmd_info() -> Result<()> {
    println!("Director Developer - System Information");
    println!("=======================================\n");

    // API Summary
    let summary = reflectors::scripting::get_api_summary()?;
    println!("Rhai Scripting API:");
    println!("  • Functions: {}", summary.function_count);
    println!("  • Modules: {}", summary.module_count);
    println!();

    // Schema Types
    let types = reflectors::schema::list_types();
    println!("Schema Types: {}", types.len());
    println!();

    // Pipeline
    println!("Pipeline Capabilities:");
    println!("  • Node types: 8 (Box, Text, Image, Video, Vector, Lottie, Effect, Composition)");
    println!("  • Effects: 7 (Blur, DropShadow, ColorMatrix, Grayscale, Sepia, DirectionalBlur, FilmGrain)");
    println!("  • Transitions: 6 (Fade, SlideLeft, SlideRight, WipeLeft, WipeRight, CircleOpen)");
    println!();

    // Animation check example
    println!("Animation Support Check:");
    let test_cases = [
        ("Box", "opacity"),
        ("Box", "border_radius"),
        ("Text", "font_size"),
        ("Lottie", "speed"),
        ("Image", "custom_prop"),
    ];
    for (node, prop) in test_cases {
        let supported = reflectors::pipeline::supports_animation(node, prop);
        let icon = if supported { "✓" } else { "✗" };
        println!("  {} {}.{}", icon, node, prop);
    }

    Ok(())
}

/// Create a new feature spec from a template.
fn cmd_new(name: &str, template: &str, output_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(output_dir)?;

    let filename = format!("{}.ron", name.to_lowercase().replace(' ', "_"));
    let output_path = output_dir.join(&filename);

    if output_path.exists() {
        return Err(anyhow::anyhow!(
            "Spec already exists: {}. Use a different name or delete the existing file.",
            output_path.display()
        ));
    }

    let content = match template {
        "effect" => generate_effect_template(name),
        "node" => generate_node_template(name),
        "animation" => generate_animation_template(name),
        "api" => generate_api_template(name),
        _ => {
            println!(
                "Unknown template: {}. Available: effect, node, animation, api",
                template
            );
            return Ok(());
        }
    };

    std::fs::write(&output_path, content)?;
    println!("Created feature spec: {}", output_path.display());
    println!("\nNext steps:");
    println!("  1. Edit the spec to define your feature");
    println!(
        "  2. Run: director-dev validate --spec {}",
        output_path.display()
    );
    println!(
        "  3. Run: director-dev generate --spec {}",
        output_path.display()
    );

    Ok(())
}

fn generate_effect_template(name: &str) -> String {
    format!(
        r#"// Feature Spec: {}
//
// Template: effect
// Generated by director-dev

FeatureSpec(
    title: "{}",
    
    user_story: "As a video creator, I want to add a {} effect so that I can enhance my visual content.",
    
    priority: 2,
    
    related_types: [
        "EffectConfig",
        "Node",
    ],
    
    related_functions: [
        "effect",
        "add_",
    ],
    
    schema_changes: [
        SchemaChange(
            target: "EffectConfig",
            change: add_variant(
                name: "{}",
                fields: [
                    // Add your effect parameters here
                    // ("color", "Color"),
                    // ("intensity", "f32"),
                ],
            ),
        ),
    ],
    
    scripting_requirements: [
        ScriptingRequirement(
            function_name: "add_{}",
            signature: "add_{}(node_id: NodeId, /* params */)",
            doc_comment: Some("Adds a {} effect to the specified node."),
        ),
    ],
    
    pipeline_requirements: [
        PipelineRequirement(
            description: "Implement {} effect rendering",
            affected_area: Some("director-core/src/effects/"),
        ),
    ],
    
    verification: VerificationSpec(
        script_compiles: true,
        schema_validates: true,
        custom_scripts: [],
        test_cases: [
            "{} effect renders correctly",
            "Effect parameters work as expected",
        ],
    ),
)
"#,
        name,
        name,
        name.to_lowercase(),
        name,
        name.to_lowercase(),
        name.to_lowercase(),
        name.to_lowercase(),
        name,
        name
    )
}

fn generate_node_template(name: &str) -> String {
    format!(
        r#"// Feature Spec: {}
//
// Template: node
// Generated by director-dev

FeatureSpec(
    title: "{}",
    
    user_story: "As a video creator, I want to add a {} node so that I can include new visual elements.",
    
    priority: 2,
    
    related_types: [
        "Node",
        "NodeKind",
        "StyleMap",
    ],
    
    related_functions: [
        "add_",
    ],
    
    schema_changes: [
        SchemaChange(
            target: "NodeKind",
            change: add_variant(
                name: "{}",
                fields: [
                    // Add your node properties here
                    // ("src", "String"),
                ],
            ),
        ),
    ],
    
    scripting_requirements: [
        ScriptingRequirement(
            function_name: "add_{}",
            signature: "add_{}(props: Map) -> NodeId",
            doc_comment: Some("Creates a new {} node."),
        ),
    ],
    
    pipeline_requirements: [
        PipelineRequirement(
            description: "Implement {} node rendering in pipeline",
            affected_area: Some("director-pipeline/src/lib.rs"),
        ),
    ],
    
    verification: VerificationSpec(
        script_compiles: true,
        schema_validates: true,
        custom_scripts: [],
        test_cases: [
            "{} node creates successfully",
            "{} node renders correctly",
        ],
    ),
)
"#,
        name,
        name,
        name.to_lowercase(),
        name,
        name.to_lowercase(),
        name.to_lowercase(),
        name.to_lowercase(),
        name,
        name,
        name
    )
}

fn generate_animation_template(name: &str) -> String {
    format!(
        r#"// Feature Spec: {}
//
// Template: animation
// Generated by director-dev

FeatureSpec(
    title: "{}",
    
    user_story: "As a video creator, I want {} animation so that I can create more dynamic content.",
    
    priority: 2,
    
    related_types: [
        "Animation",
        "EasingType",
        "SpringConfig",
    ],
    
    related_functions: [
        "animate",
        "spring",
    ],
    
    schema_changes: [],
    
    scripting_requirements: [
        ScriptingRequirement(
            function_name: "{}",
            signature: "{}(node_id: NodeId, /* params */)",
            doc_comment: Some("Applies {} animation to a node."),
        ),
    ],
    
    pipeline_requirements: [
        PipelineRequirement(
            description: "Implement {} animation in animation system",
            affected_area: Some("director-core/src/animation.rs"),
        ),
    ],
    
    verification: VerificationSpec(
        script_compiles: true,
        schema_validates: true,
        custom_scripts: [],
        test_cases: [
            "{} animation applies correctly",
            "Animation timing is accurate",
        ],
    ),
)
"#,
        name,
        name,
        name.to_lowercase(),
        name.to_lowercase(),
        name.to_lowercase(),
        name.to_lowercase(),
        name,
        name
    )
}

fn generate_api_template(name: &str) -> String {
    format!(
        r#"// Feature Spec: {}
//
// Template: api
// Generated by director-dev

FeatureSpec(
    title: "{}",
    
    user_story: "As a video creator, I want a {} API so that I can programmatically control my videos.",
    
    priority: 2,
    
    related_types: [],
    
    related_functions: [],
    
    schema_changes: [],
    
    scripting_requirements: [
        ScriptingRequirement(
            function_name: "{}",
            signature: "{}(/* params */)",
            doc_comment: Some("{}"),
        ),
    ],
    
    pipeline_requirements: [],
    
    verification: VerificationSpec(
        script_compiles: true,
        schema_validates: true,
        custom_scripts: [],
        test_cases: [
            "{} function works as expected",
        ],
    ),
)
"#,
        name,
        name,
        name.to_lowercase(),
        name.to_lowercase(),
        name.to_lowercase(),
        name,
        name
    )
}

/// Generate an AI prompt from a template.
fn cmd_prompt(
    template_name: Option<&str>,
    var_args: &[String],
    list: bool,
    output: Option<&Path>,
) -> Result<()> {
    // List templates if requested
    if list {
        println!("Available Prompt Templates:");
        println!("============================");
        for template in prompts::list_templates() {
            println!("  • {} - {}", template.name, template.description);
        }
        println!("\nUsage:");
        println!("  director-dev prompt implement_effect --var EFFECT_NAME=Glow");
        return Ok(());
    }

    // Template is required
    let template = template_name.ok_or_else(|| {
        anyhow::anyhow!("Template name required. Use --list to see available templates.")
    })?;

    // Parse variables
    let vars = prompts::parse_vars(var_args);

    // Generate the prompt
    let result = prompts::generate_prompt(template, &vars)?;

    // Output to file or stdout
    if let Some(path) = output {
        std::fs::write(path, &result)?;
        println!("Prompt written to: {}", path.display());
    } else {
        println!("{}", result);
    }

    Ok(())
}

/// Show workspace dependency graph.
fn cmd_graph(format: &str, impact: Option<&str>, deps: Option<&str>) -> Result<()> {
    // Impact analysis mode
    if let Some(crate_name) = impact {
        let affected = reflectors::graph::get_impact(crate_name)?;
        println!("Crates affected by changes to '{}':", crate_name);
        println!("=====================================");
        if affected.is_empty() {
            println!("  (no dependents)");
        } else {
            for name in affected {
                println!("  • {}", name);
            }
        }
        return Ok(());
    }

    // Dependency analysis mode
    if let Some(crate_name) = deps {
        let dependencies = reflectors::graph::get_dependencies(crate_name)?;
        println!("Dependencies of '{}':", crate_name);
        println!("==========================");
        if dependencies.is_empty() {
            println!("  (no dependencies)");
        } else {
            for name in dependencies {
                println!("  • {}", name);
            }
        }
        return Ok(());
    }

    // Format output
    match format {
        "mermaid" => {
            let diagram = reflectors::graph::generate_mermaid_diagram()?;
            println!("{}", diagram);
        }
        "json" => {
            let json = reflectors::graph::generate_json()?;
            println!("{}", json);
        }
        "list" | _ => {
            let crates = reflectors::graph::list_crates()?;
            println!("Workspace Crates:");
            println!("=================");
            println!("{:<25} {:>5} deps | {:>5} dependents", "Name", "", "");
            println!("{:-<50}", "");
            for (name, dep_count, dependent_count) in crates {
                println!(
                    "{:<25} {:>5} deps | {:>5} dependents",
                    name, dep_count, dependent_count
                );
            }
        }
    }

    Ok(())
}
