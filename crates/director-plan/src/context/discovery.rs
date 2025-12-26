use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::types::Ticket;

/// Discovers relevant files based on the ticket description.
pub fn discover_context(ticket: &Ticket, root: &Path) -> Vec<String> {
    // If auto_context is enabled, we use AST Engine but seeded by heuristics.
    // The dead code block previously here is removed.

    let mut seeds = ticket.spec.relevant_files.clone();

    // 1. Heuristic Discovery (run if seeds are empty)
    if seeds.is_empty() {
        seeds = heuristic_discovery(ticket, root);
    }

    // 2. AST Expansion (if auto_context is true)
    if ticket.spec.auto_context && !seeds.is_empty() {
        let mut graph = crate::context::ast::DependencyGraph::new(root);
        if let Ok(_) = graph.build() {
            // Get context expands the graph from seeds
            let context_data = graph.get_context(&seeds);
            // Return only paths. Pruning of content happens in execution_loop if it uses get_context again.
            // Or ideally execution_loop should rely on this function returning paths, but it re-reads them.
            // To get pruning benefit, execution_loop logic was updated to use AST directly.
            // This function supports the CLI 'context' command mainly now.
            return context_data.into_iter().map(|(p, _)| p).collect();
        } else {
             eprintln!("AST Context failed to build, using seeds only.");
        }
    }

    seeds
}

fn heuristic_discovery(ticket: &Ticket, root: &Path) -> Vec<String> {
    let tokens = tokenize(&ticket.spec.description);
    if tokens.is_empty() {
        return vec![];
    }

    let mut scored_files: Vec<(String, u32)> = Vec::new();
    let ignore_patterns = vec![
        "target/",
        "node_modules/",
        ".git/",
        "dist/",
        "build/",
        ".lock",
        "package-lock.json",
        "yarn.lock",
        "assets/",
    ];

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        // Skip ignored paths
        if path.components().any(|c| {
            let s = c.as_os_str().to_string_lossy();
            ignore_patterns.iter().any(|pat| s.contains(&pat.replace("/", "")))
        }) {
            continue;
        }

        let rel_path = match path.strip_prefix(root) {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => continue,
        };

        let rel_path_normalized = rel_path.replace("\\", "/");

        let mut score = 0;
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();

        for token in &tokens {
            if file_name.contains(token) {
                score += 10;
            }
        }

        if score < 10 {
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy();
                if ["rs", "ts", "tsx", "js", "toml", "json", "md", "css", "html"].contains(&ext_str.as_ref()) {
                     if let Ok(content) = fs::read_to_string(path) {
                        for token in &tokens {
                            if content.contains(token) {
                                score += 1;
                            }
                        }
                     }
                }
            }
        }

        if score > 0 {
            scored_files.push((rel_path_normalized, score));
        }
    }

    scored_files.sort_by(|a, b| b.1.cmp(&a.1));
    scored_files.into_iter().map(|(path, _)| path).collect()
}

fn tokenize(text: &str) -> HashSet<String> {
    let stop_words: HashSet<&str> = [
        "the", "and", "a", "an", "to", "in", "of", "for", "with", "on", "at",
        "by", "from", "up", "about", "into", "over", "after", "implement", "update",
        "create", "add", "fix", "remove", "delete", "refactor", "change", "modify",
        "use", "using", "ensure", "make", "is", "are", "was", "were", "be", "been",
        "can", "could", "should", "would", "will", "may", "might", "must", "have", "has", "had",
        "do", "does", "did", "todo", "done", "spec", "ticket", "description", "title", "status", "priority"
    ].iter().cloned().collect();

    text.split_whitespace()
        .map(|s| {
            s.chars()
             .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
             .collect::<String>()
             .to_lowercase()
        })
        .filter(|s| !s.is_empty() && !stop_words.contains(s.as_str()))
        .collect()
}
