use std::path::{Path, PathBuf};
use std::collections::{HashMap, VecDeque};
use std::fs;
use anyhow::{Result};
use petgraph::graph::DiGraph;
use petgraph::prelude::*;
use oxc_allocator::Allocator;
use oxc_parser::{Parser};
use oxc_span::{SourceType, GetSpan}; // Added GetSpan
use oxc_ast::ast::{Statement};
use walkdir::WalkDir;

/// Represents a node in our dependency graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileNode {
    pub path: String, // Relative path from root
    pub file_type: FileType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FileType {
    TypeScript, // .ts, .tsx
    Rust,       // .rs
    Other,
}

/// The dependency graph of the workspace.
pub struct DependencyGraph {
    pub graph: DiGraph<FileNode, ()>,
    pub node_map: HashMap<String, NodeIndex>,
    pub root: PathBuf,
}

impl DependencyGraph {
    pub fn new(root: &Path) -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
            root: root.to_path_buf(),
        }
    }

    /// Builds the full dependency graph by scanning the workspace.
    pub fn build(&mut self) -> Result<()> {
        let ignore_patterns = vec![
            "target", "node_modules", ".git", "dist", "build",
        ];

        // 1. Discover all files first
        let mut files = Vec::new();
        for entry in WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if path.components().any(|c| {
                let s = c.as_os_str().to_string_lossy();
                ignore_patterns.iter().any(|pat| s == *pat)
            }) {
                continue;
            }

            let rel_path = path.strip_prefix(&self.root)?.to_string_lossy().replace("\\", "/");
            let file_type = if rel_path.ends_with(".ts") || rel_path.ends_with(".tsx") {
                FileType::TypeScript
            } else if rel_path.ends_with(".rs") {
                FileType::Rust
            } else {
                FileType::Other
            };

            files.push((rel_path, file_type));
        }

        // 2. Add nodes
        for (rel_path, file_type) in &files {
            self.add_node(rel_path, file_type.clone());
        }

        // 3. Add edges (Analyze imports)
        let files_clone = files.clone();
        for (rel_path, file_type) in files_clone {
             if let Err(e) = self.analyze_imports(&rel_path, &file_type) {
                 eprintln!("Failed to analyze imports for {}: {}", rel_path, e);
             }
        }

        Ok(())
    }

    fn add_node(&mut self, path: &str, file_type: FileType) -> NodeIndex {
        if let Some(&idx) = self.node_map.get(path) {
            return idx;
        }
        let node = FileNode {
            path: path.to_string(),
            file_type,
        };
        let idx = self.graph.add_node(node);
        self.node_map.insert(path.to_string(), idx);
        idx
    }

    fn add_edge(&mut self, from: &str, to: &str) {
        if let (Some(&from_idx), Some(&to_idx)) = (self.node_map.get(from), self.node_map.get(to)) {
            if !self.graph.contains_edge(from_idx, to_idx) {
                self.graph.add_edge(from_idx, to_idx, ());
            }
        }
    }

    fn analyze_imports(&mut self, rel_path: &str, file_type: &FileType) -> Result<()> {
        let abs_path = self.root.join(rel_path);
        let content = fs::read_to_string(&abs_path)?;

        match file_type {
            FileType::TypeScript => {
                let imports = parse_ts_imports(&rel_path, &content, &self.root)?;
                for import in imports {
                    if let Some(resolved) = self.resolve_ts_import(rel_path, &import) {
                         self.add_edge(rel_path, &resolved);
                    }
                }
            },
            FileType::Rust => {
                let imports = parse_rs_imports(&content);
                for import in imports {
                    if let Some(resolved) = self.resolve_rs_import(rel_path, &import) {
                        self.add_edge(rel_path, &resolved);
                    }
                }
            },
            _ => {}
        }
        Ok(())
    }

    fn resolve_ts_import(&self, current_file: &str, import_path: &str) -> Option<String> {
        let current_dir = Path::new(current_file).parent().unwrap_or(Path::new(""));

        let mut candidates = Vec::new();

        if import_path.starts_with(".") {
            candidates.push(current_dir.join(import_path));
        } else if import_path.starts_with("@/") {
             let alias_content = import_path.strip_prefix("@/").unwrap();
             candidates.push(Path::new("apps/director-plan/src").join(alias_content));
             candidates.push(Path::new("src").join(alias_content));
        }

        let extensions = ["ts", "tsx", "d.ts", "js", "jsx"];

        for candidate in candidates {
            let s = candidate.to_string_lossy().replace("\\", "/");
            if self.node_map.contains_key(&s) { return Some(s); }

            for ext in &extensions {
                let with_ext = format!("{}.{}", s, ext);
                if self.node_map.contains_key(&with_ext) { return Some(with_ext); }
            }

            for ext in &extensions {
                let index = format!("{}/index.{}", s, ext);
                if self.node_map.contains_key(&index) { return Some(index); }
            }
        }

        None
    }

    fn resolve_rs_import(&self, current_file: &str, module_path: &str) -> Option<String> {
        let parts: Vec<&str> = module_path.split("::").collect();
        if parts.is_empty() { return None; }

        let current_path = Path::new(current_file);
        let parent = current_path.parent().unwrap_or(Path::new(""));

        let neighbor = parent.join(format!("{}.rs", parts[0]));
        let s = neighbor.to_string_lossy().replace("\\", "/");
        if self.node_map.contains_key(&s) { return Some(s); }

        let mod_rs = parent.join(parts[0]).join("mod.rs");
        let s_mod = mod_rs.to_string_lossy().replace("\\", "/");
        if self.node_map.contains_key(&s_mod) { return Some(s_mod); }

        if module_path.starts_with("crate::") {
             let mut p = parent;
             loop {
                 if p.file_name().and_then(|n| n.to_str()) == Some("src") {
                     break;
                 }
                 if let Some(parent_p) = p.parent() {
                     p = parent_p;
                 } else {
                     break; // Not found
                 }
             }

             let sub_path = module_path.strip_prefix("crate::").unwrap();
             let resolved = p.join(sub_path.replace("::", "/")).with_extension("rs");
             let s_crate = resolved.to_string_lossy().replace("\\", "/");
             if self.node_map.contains_key(&s_crate) { return Some(s_crate); }
        }

        None
    }

    pub fn get_context(&self, entry_files: &[String]) -> Vec<(String, String)> {
        let mut visited = HashMap::new();
        let mut queue = VecDeque::new();

        for f in entry_files {
            if let Some(&idx) = self.node_map.get(f) {
                visited.insert(f.clone(), 0);
                queue.push_back((idx, 0));
            }
        }

        while let Some((idx, depth)) = queue.pop_front() {
            if depth >= 2 {
                continue;
            }

            for neighbor in self.graph.neighbors(idx) {
                let neighbor_path = &self.graph[neighbor].path;
                let new_depth = depth + 1;

                if !visited.contains_key(neighbor_path) || visited[neighbor_path] > new_depth {
                    visited.insert(neighbor_path.clone(), new_depth);
                    if new_depth < 3 {
                         queue.push_back((neighbor, new_depth));
                    }
                }
            }
        }

        let mut results = Vec::new();
        for (path, depth) in visited {
             let abs_path = self.root.join(&path);
             if let Ok(content) = fs::read_to_string(&abs_path) {
                 if depth <= 1 {
                     results.push((path, content));
                 } else if depth == 2 {
                     let pruned = prune_content(&path, &content);
                     results.push((path, pruned));
                 }
             }
        }

        results.sort_by(|a, b| a.0.cmp(&b.0));
        results
    }
}

// --- AST Parsing (TypeScript/OXC) ---

fn parse_ts_imports(_path: &str, content: &str, _root: &Path) -> Result<Vec<String>> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(Path::new(_path)).unwrap_or_default().with_typescript(true).with_module(true);

    let parser = Parser::new(&allocator, content, source_type);
    let ret = parser.parse();

    if !ret.errors.is_empty() {
        return Ok(vec![]);
    }

    let program = ret.program;
    let mut imports = Vec::new();

    for stmt in program.body {
        match stmt {
             Statement::ImportDeclaration(decl) => {
                 imports.push(decl.source.value.to_string());
             },
             Statement::ExportAllDeclaration(decl) => {
                 imports.push(decl.source.value.to_string());
             },
             Statement::ExportNamedDeclaration(decl) => {
                 if let Some(source) = &decl.source {
                     imports.push(source.value.to_string());
                 }
             },
             _ => {}
        }
    }

    Ok(imports)
}

// --- AST Parsing (Rust/Syn) ---

fn parse_rs_imports(content: &str) -> Vec<String> {
    let file = match syn::parse_file(content) {
        Ok(f) => f,
        Err(_) => return vec![],
    };

    let mut imports = Vec::new();
    for item in file.items {
        match item {
            syn::Item::Use(u) => {
                extract_use_paths(&u.tree, String::new(), &mut imports);
            },
            syn::Item::Mod(m) => {
                if m.content.is_none() {
                    imports.push(m.ident.to_string());
                }
            }
            _ => {}
        }
    }
    imports
}

fn extract_use_paths(tree: &syn::UseTree, prefix: String, results: &mut Vec<String>) {
    match tree {
        syn::UseTree::Path(p) => {
            let new_prefix = if prefix.is_empty() {
                p.ident.to_string()
            } else {
                format!("{}::{}", prefix, p.ident)
            };
            extract_use_paths(&p.tree, new_prefix, results);
        },
        syn::UseTree::Name(n) => {
            let full = if prefix.is_empty() {
                n.ident.to_string()
            } else {
                format!("{}::{}", prefix, n.ident)
            };
            results.push(full);
        },
        syn::UseTree::Rename(n) => {
            let full = if prefix.is_empty() {
                n.ident.to_string()
            } else {
                format!("{}::{}", prefix, n.ident)
            };
            results.push(full);
        },
        syn::UseTree::Glob(_) => {
            results.push(prefix);
        },
        syn::UseTree::Group(g) => {
            for item in &g.items {
                extract_use_paths(item, prefix.clone(), results);
            }
        },
    }
}


// --- Content Pruning ---

fn prune_content(path: &str, content: &str) -> String {
    if path.ends_with(".ts") || path.ends_with(".tsx") {
        prune_ts(content)
    } else {
        content.lines().take(50).collect::<Vec<_>>().join("\n") + "\n... (pruned)"
    }
}

fn prune_ts(content: &str) -> String {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(Path::new("dummy.tsx")).unwrap_or_default().with_typescript(true).with_module(true);
    let parser = Parser::new(&allocator, content, source_type);
    let ret = parser.parse();

    if !ret.errors.is_empty() {
         return content.to_string(); // Fallback if parse error
    }

    let program = ret.program;
    let mut parts = Vec::new();
    let mut last_pos = 0;

    for stmt in program.body {
        match stmt {
             Statement::ImportDeclaration(_) |
             Statement::ExportAllDeclaration(_) |
             Statement::ExportNamedDeclaration(_) |
             Statement::TSTypeAliasDeclaration(_) |
             Statement::TSInterfaceDeclaration(_) => {
                 let span = stmt.span();
                 parts.push(&content[last_pos..span.end as usize]);
                 last_pos = span.end as usize;
             },
             Statement::FunctionDeclaration(f) => {
                 if let Some(body) = &f.body {
                     let body_span = body.span;
                     let start = f.span.start as usize;
                     parts.push(&content[last_pos..start]);
                     parts.push(&content[start..body_span.start as usize]);
                     parts.push("{ /* body pruned */ }");
                     last_pos = body_span.end as usize;
                 } else {
                     let span = f.span;
                     parts.push(&content[last_pos..span.end as usize]);
                     last_pos = span.end as usize;
                 }
             },
             Statement::ClassDeclaration(c) => {
                 let body_span = c.body.span;
                 let start = c.span.start as usize;
                 parts.push(&content[last_pos..start]);
                 parts.push(&content[start..body_span.start as usize]);
                 parts.push("{ /* class members pruned */ }");
                 last_pos = body_span.end as usize;
             },
             Statement::VariableDeclaration(v) => {
                 let span = v.span;
                 parts.push(&content[last_pos..span.end as usize]);
                 last_pos = span.end as usize;
             }
             _ => {
                 let span = stmt.span();
                 parts.push(&content[last_pos..span.end as usize]);
                 last_pos = span.end as usize;
             }
        }
    }

    parts.push(&content[last_pos..]);

    parts.join("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_ts_import_parsing() {
        let content = r#"
            import { Button } from '@/components/ui/button';
            import React from 'react';
            export * from './utils';

            const x: number = 1;
            function foo() {}
        "#;

        let imports = super::parse_ts_imports("test.tsx", content, Path::new(".")).unwrap();
        assert!(imports.contains(&"@/components/ui/button".to_string()));
        assert!(imports.contains(&"react".to_string()));
        assert!(imports.contains(&"./utils".to_string()));
    }

    #[test]
    fn test_ts_pruning() {
         let content = r#"
            import { Button } from 'ui';

            interface User {
                id: string;
            }

            function process(u: User): void {
                console.log(u);
                if (true) { return; }
            }

            class Manager {
                data: any;
                constructor() { this.data = {}; }
            }
         "#;

         let pruned = super::prune_ts(content);
         assert!(pruned.contains("interface User"));
         assert!(pruned.contains("function process(u: User): void { /* body pruned */ }"));
         assert!(pruned.contains("class Manager { /* class members pruned */ }"));
         assert!(!pruned.contains("console.log"));
    }
}
