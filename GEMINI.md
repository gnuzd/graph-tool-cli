# **Graph: Real-Time JS/TS Dependency Visualizer**

## **Project Overview**
**Graph** is a high-performance, standalone CLI tool designed to visualize JavaScript and TypeScript dependencies in real-time. It aims for a zero-configuration experience, allowing users to run `graph .` in any project directory to instantly see a visual representation of their module graph.

### **Core Principles**
- **Zero-Configuration:** Works out-of-the-box without requiring complex setup files.
- **Read-Only:** Analyzes source code without modifying files or creating disk-heavy caches.
- **Instantaneous:** Updates the UI in under 15ms upon file changes.
- **Single Binary:** Bundles the UI directly into the Rust executable for easy distribution.

## **Architecture & Tech Stack**

### **Engine (Rust)**
- **AST Parsing:** Powered by `oxc_parser` for high-speed analysis of JS/TS files.
- **File Watching:** Uses `notify` to detect file system changes and trigger updates.
- **Web Server:** `axum` handles the web dashboard and WebSocket broadcasting.
- **Asset Embedding:** `rust-embed` bundles the Svelte UI into the final binary.

### **Frontend (Svelte 5)**
- **Reactivity:** Leverages Svelte 5 for minimal bundle size and direct DOM reactivity.
- **Styling:** Tailwind CSS for modern, responsive design.
- **Icons:** Lucide-Svelte for consistent iconography.

## **Directory Structure (Planned)**
- `src/`: Rust engine source code (`main.rs`, `parser.rs`, `server.rs`).
- `ui/`: Svelte 5 frontend application.
- `.blueprint/`: Project specifications, roadmap, and architectural guidelines.

## **Building and Running**

### **Prerequisites**
- Rust (latest stable)
- Node.js & npm (for UI development)

### **Development Commands**
- **Initialize Rust Engine:**
  ```bash
  # In the root directory
  cargo init
  # Add dependencies as specified in .blueprint/SETUP.md
  ```
- **Initialize UI:**
  ```bash
  mkdir ui
  cd ui
  npm create vite@latest . -- --template svelte-ts
  npm install
  ```
- **Run the Project:**
  ```bash
  # TODO: Define combined run command (e.g., using a Makefile or script)
  cargo run -- .
  ```

## **Development Conventions**
- **Performance First:** All AST analysis and path resolution must be optimized for speed.
- **Stateless Analysis:** Avoid creating temporary files or long-lived caches; prefer in-memory processing.
- **Type Safety:** Strict TypeScript usage in the UI and idiomatic Rust in the engine.
