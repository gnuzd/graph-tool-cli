mod parser;
mod server;

use clap::Parser;
use notify::{RecursiveMode, Watcher};
use oxc_resolver::{ResolveOptions, Resolver};
use server::{Edge, GraphData, GraphUpdate, Node};
use std::env;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(default_value = ".")]
    path: PathBuf,
    #[arg(long)]
    root: bool,
}

fn rebuild_graph(root: &Path, resolver: &Resolver, include_root: bool) -> GraphData {
    let files = parser::scan_directory(root, include_root);
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for file in &files {
        if let Some(module) = parser::parse_module(file, resolver) {
            let file_id = file.to_string_lossy().to_string();
            nodes.push(Node {
                id: file_id.clone(),
                label: file
                    .strip_prefix(root)
                    .unwrap_or(file)
                    .to_string_lossy()
                    .to_string(),
            });

            for resolved in module.resolved_dependencies {
                edges.push(Edge {
                    source: file_id.clone(),
                    target: resolved.to_string_lossy().to_string(),
                });
            }
        }
    }

    println!(
        "Graph rebuilt: {} nodes, {} edges",
        nodes.len(),
        edges.len()
    );
    GraphData { nodes, edges }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let analysis_path = Arc::new(
        args.path
            .canonicalize()
            .expect("Failed to canonicalize path"),
    );

    let app_path = env::current_dir().expect("Failed to get current directory");
    let ui_path = app_path.join("ui/dist");

    println!("Analyzing and watching: {:?}", analysis_path);

    // Setup resolver
    let options = ResolveOptions {
        extensions: vec![".ts".into(), ".js".into(), ".svelte".into()],
        ..ResolveOptions::default()
    };
    let resolver = Arc::new(Resolver::new(options));

    // Initial Scan
    let initial_graph = rebuild_graph(&analysis_path, &resolver, args.root);
    let shared_graph = Arc::new(Mutex::new(initial_graph));

    // WebSocket State
    let (tx, _) = broadcast::channel(100);
    let state = Arc::new(server::AppState {
        tx: tx.clone(),
        current_graph: Arc::clone(&shared_graph),
    });

    // Setup File Watcher
    let tx_clone = tx.clone();
    let root_path_clone = Arc::clone(&analysis_path);
    let resolver_clone = Arc::clone(&resolver);
    let shared_graph_clone = Arc::clone(&shared_graph);
    let include_root_clone = args.root;

    let mut watcher =
        notify::recommended_watcher(move |res: notify::Result<notify::Event>| match res {
            Ok(event) => {
                if event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove() {
                    let new_graph =
                        rebuild_graph(&root_path_clone, &resolver_clone, include_root_clone);

                    {
                        let mut graph = shared_graph_clone.lock().unwrap();
                        *graph = new_graph.clone();
                    }

                    let _ = tx_clone.send(GraphUpdate {
                        kind: "graph".to_string(),
                        payload: new_graph,
                    });
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        })
        .unwrap();

    watcher
        .watch(&*analysis_path, RecursiveMode::Recursive)
        .unwrap();

    // Open browser in background after a short delay
    tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let _ = open::that("http://localhost:8000");
    });

    // Start Server
    server::start_server(state, ui_path).await;
}
