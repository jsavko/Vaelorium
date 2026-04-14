# Tauri command bridge

## Responsibility
Type-safe RPC between Svelte frontend and Rust backend. Every Rust fn marked `#[tauri::command]` registered in `invoke_handler![...]` is callable from TS via `callCommand`. Also owns the browser-mock fallback for non-Tauri dev (Vite-only).

## Entry points

### Rust
- `src-tauri/src/lib.rs` — `run()` is the Tauri app entry. `invoke_handler![...]` macro (around line 113) lists every command. **New commands must be added here** or they silently fail at invoke time.
- Command modules live in `src-tauri/src/commands/*.rs`. Each fn is `#[tauri::command] pub async fn xxx(...) -> Result<T, String>`.

### Frontend
- `src/lib/api/bridge.ts` — `callCommand<T>(name, args)`. Branches on `isTauri`: uses `@tauri-apps/api/core` invoke in Tauri, otherwise calls `mockCommand` (in-memory stubs for browser dev).
- `src/lib/api/*.ts` — per-subsystem wrappers. Each exposes typed fns calling `callCommand` with the right name.

## camelCase rule (critical)
- **Tauri `invoke()` args MUST be camelCase** — snake_case silently fails (`feedback_tauri_camelcase`).
- Rust side: fields auto-convert via serde `rename_all = "camelCase"` or manual renaming. If a command takes `tome_id`, call from TS with `{ tomeId }`.
- Response mapping: Rust returns snake_case JSON (e.g. `tome_uuid`); TS API wrappers convert to camelCase TS types (e.g. `cloud.ts` has `fromRawUsage`).

## Error handling
- Rust commands return `Result<T, String>`. `String` flows back as a rejected promise with that message.
- Frontend typically wraps calls with try/catch and `showToast(msg, 'error')`.

## Mock backend
- `bridge.ts` has an in-memory `mockDb`. Used for vitest unit tests + browser-only Playwright runs.
- Not all commands are mocked — Tauri-only paths (sync, backup) check `isTauri` and no-op gracefully.

## Data flow (typical call)
1. TS: `callCommand('save_page', { pageId, content })`
2. Bridge: `invoke('save_page', {pageId, content})`
3. Tauri deserializes args → Rust fn gets them after serde camelCase conversion.
4. Rust returns `Result<T, String>` → serialized back → TS resolves / rejects.

## Gotchas
- Browser dev: if a feature depends on Tauri-only state (sync, filesystem), gate on `isTauri` and no-op.
- Adding a new table / column: update migrations, schema registry, AND every test harness list (`feedback_register_migrations_in_both_runners`).
- Migration SQL comments `--` must be on their own lines (`feedback_migration_sql_comments`).
- Restart `npm run tauri dev` after Rust changes; use `pkill -f "target/debug/vaelorium"` to kill (`feedback_restart_tauri_dev`, `feedback_tauri_dev_restart_pkill`).

## Where NOT to look
- `@tauri-apps/plugin-dialog` etc. are plugins — direct `import`, not commands.
- `@tauri-apps/api/event` for pub/sub events is a separate mechanism (see `subscribeSyncStatus` in `api/sync.ts`).
