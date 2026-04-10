# Vaelorium

Self-hosted offline-first worldbuilding and RPG campaign manager.

## Prerequisites

- Node.js 20+
- Rust 1.77+
- System dependencies (Ubuntu/WSL):
  ```bash
  sudo apt-get install -y pkg-config libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev build-essential libssl-dev
  ```

## Setup

```bash
npm install
```

## Development

```bash
# Frontend only (hot reload)
npm run dev

# Full Tauri app (desktop window)
npm run tauri:dev

# Type checking
npm run check
```

## Build

```bash
npm run tauri:build
```

## Project Structure

```
src/                    # Svelte 5 frontend
  lib/
    api/                # Tauri command wrappers (TypeScript)
    components/         # Svelte components
    editor/             # TipTap editor configuration
    stores/             # Svelte stores (reactive state)
  App.svelte            # Root layout
  app.css               # Tailwind + design tokens
  main.ts               # Entry point

src-tauri/              # Rust backend (Tauri 2)
  src/
    db/                 # SQLite database module
    lib.rs              # Tauri app setup
    main.rs             # Entry point
  migrations/           # SQLite migrations
  Cargo.toml

design/                 # Design files
  specs/                # Design specifications per milestone
  exports/              # Exported mockup PNGs
  *.pen                 # Pencil design files
```

## Tech Stack

- **Desktop:** Tauri 2 (Rust backend, webview frontend)
- **Frontend:** Svelte 5, Tailwind CSS 4
- **Editor:** TipTap (ProseMirror-based) with Yjs CRDTs
- **Database:** SQLite (via sqlx) with FTS5 full-text search
- **Sync:** Optional self-hosted or managed server (Milestone 8)
