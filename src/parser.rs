use std::path::{Path, PathBuf};
use ignore::Walk;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_ast::ast::Statement;
use oxc_resolver::Resolver;

#[derive(Debug)]
pub struct Module {
    pub path: PathBuf,
    pub dependencies: Vec<String>,
    pub resolved_dependencies: Vec<PathBuf>,
}

pub fn scan_directory(dir: &Path, include_root: bool) -> Vec<PathBuf> {
    Walk::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            if !include_root && e.depth() == 1 && e.file_type().map_or(false, |ft| ft.is_file()) {
                return false;
            }
            let path = e.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    return ext == "js" || ext == "ts" || ext == "jsx" || ext == "tsx" || ext == "svelte";
                }
            }
            false
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

pub fn parse_module(path: &Path, resolver: &Resolver) -> Option<Module> {
    let source_text = std::fs::read_to_string(path).ok()?;
    
    // If it's a svelte file, try to extract script content
    let (content, source_type) = if path.extension().map_or(false, |ext| ext == "svelte") {
        let script_content = extract_svelte_script(&source_text).unwrap_or_default();
        (script_content, SourceType::default().with_typescript(true))
    } else {
        (source_text, SourceType::from_path(path).unwrap_or_default())
    };

    let allocator = Allocator::default();
    let parser = Parser::new(&allocator, &content, source_type);
    let ret = parser.parse();
    
    let program = ret.program;
    let mut dependencies = Vec::new();

    for stmt in &program.body {
        match stmt {
            Statement::ImportDeclaration(import) => {
                dependencies.push(import.source.value.to_string());
            }
            Statement::ExportNamedDeclaration(export) => {
                if let Some(source) = &export.source {
                    dependencies.push(source.value.to_string());
                }
            }
            Statement::ExportAllDeclaration(export) => {
                dependencies.push(export.source.value.to_string());
            }
            _ => {}
        }
    }

    let parent_dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut resolved_dependencies = Vec::new();

    for dep in &dependencies {
        if let Ok(resolved) = resolver.resolve(parent_dir, dep) {
            resolved_dependencies.push(resolved.full_path().to_path_buf());
        }
    }

    Some(Module {
        path: path.to_path_buf(),
        dependencies,
        resolved_dependencies,
    })
}

fn extract_svelte_script(source: &str) -> Option<String> {
    let start_tag = "<script";
    let end_tag = "</script>";
    
    if let Some(start_pos) = source.find(start_tag) {
        if let Some(content_start) = source[start_pos..].find('>') {
            let content_start = start_pos + content_start + 1;
            if let Some(end_pos) = source[content_start..].find(end_tag) {
                return Some(source[content_start..content_start + end_pos].to_string());
            }
        }
    }
    None
}
