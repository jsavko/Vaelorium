# Vaelorium

Self-hosted, offline-first LegendKeeper alternative for worldbuilders. Cross-platform desktop app (Tauri + Svelte 5 + Rust) with optional sync and a web build.

- **Domain:** vaelorium.com
- **Theme:** arcane library; dark walnut palette is the default
- **Tome file extension:** `.tome`
- **Status:** v0.2.0 released; milestones M0–M7, M9, M10 complete; M8 (collaboration) remaining
- **Marketing site:** [vaelorium-website](https://github.com/jsavko/vaelorium-website) (private repo, sibling directory; see Marketing Website section below)

## Stack

- **Frontend:** Svelte 5 (runes: `$state`, `$props`, `$derived`), TypeScript, Vite
- **Backend:** Rust, Tauri v2, SQLite
- **Testing:** Vitest (unit), Playwright (e2e)
- **CI/CD:** GitHub Actions builds Windows/macOS/Linux; Tauri auto-updater with signed releases

## Project Conventions

- **Tauri `invoke()` args must be camelCase** — snake_case silently fails
- **No native dialogs** — never use `confirm()`/`alert()`; use `ConfirmDialog.svelte`. Skip toasts for routine actions
- **Theming** — never hardcode colors; use CSS custom properties from the design tokens. Dark cozy is default, multiple themes supported
- **Testing workflow** — always runtime-test via browser (Tauri WebKitGTK on WSL can't connect to Chrome DevTools MCP). Compile success ≠ feature works
- **Task completion bar** — a task is not done until it's fully functional AND has passing unit + Playwright tests
- **Scope discipline** — don't add features, refactor, or "improve" beyond what was asked; don't add comments/docstrings to code you didn't change

## Key Paths

- `src/lib/components/` — Svelte components
- `src/lib/stores/` — Svelte stores
- `src/lib/api/` — Tauri bridge + API wrappers
- `src-tauri/src/` — Rust backend
- `src-tauri/tauri.conf.json` — version, updater config
- `tests/` — Playwright e2e

## Updater

- Signing keys: `~/.tauri/vaelorium.key`
- GitHub release workflow publishes `latest.json` endpoint
- Runtime check in `src/lib/components/UpdateNotification.svelte` (6-hour passive auto-check)

## Releases

- All three manifests must agree on version: `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`
- Bump them together: `npm run bump <semver>` (writes all three atomically)
- CI's `version-guard` job fails the build if manifests disagree or the git tag doesn't match
- Tag push → CI builds → creates a **draft** release. Promote to public with `gh release edit vX.Y.Z --draft=false` when ready to ship
- Promoting a release fires `release.published` → triggers website rebuild (see below)

## Sync (M7 — shipped in v0.2.0)

- **Status:** All phases complete. 12 of ~14 user-data tables sync (images + versions + wiki_links intentionally deferred). S3 + filesystem backends. First-run wizard, activity log, retry/backoff, cross-device conflict resolution, restore-from-backup all shipped.
- **User guide:** `docs/sync-user-guide.md`.
- **Schema registry** at `src-tauri/src/sync/registry.rs` lists every sync-tracked table. Adding one: register in `TABLES`, call `journal::emit_for_row(&mut *tx, &TABLES.<name>, ...)` from each mutation function, call `session.nudge()` after commit.
- **Special apply paths:** `page_content` (binary BLOB) and `page_tags` (M:N pivot with composite `page_id|tag_id` row_id). Everything else goes through the generic `engine::apply_op_via_schema`.
- **Backends:** Filesystem (local folder or Syncthing) and S3-compatible (AWS, Cloudflare R2, Minio, Backblaze B2, Wasabi, Garage) — built on `aws-sdk-s3`.
- **Tome identity:** bucket prefix is `tomes/{tome_uuid}/`, where `tome_uuid` is a stable per-Tome UUID lazy-created in `tome_metadata` (see `sync::tome_identity::get_or_create_uuid`). Path-independent — the same Tome produces the same prefix on every device, which is what makes cross-device restore work.
- **Intentionally NOT synced:** wiki_links (derived from page content), versions (large; local-only), images (binary blob deferred), relation_types (built-ins dominate). Don't relitigate without strong reason.
- **S3 testing:** `docs/sync-s3-testing.md` has a Minio-in-Docker recipe; the S3 backend has unit tests but no automated integration tests (thin translation layer on top of aws-sdk-s3 that would need a real bucket to exercise meaningfully).

## Marketing Website

- **Repo:** `jsavko/vaelorium-website` (private), separate sibling at `~/Projects/vaelorium-website/`
- **Stack:** Astro 5, hosted on Cloudflare Workers + Static Assets
- **Domain:** vaelorium.com (DNS on Cloudflare)
- **Auto-rebuild:** `.github/workflows/website-rebuild.yml` in this repo fires on `release.published` and pings the `CF_DEPLOY_HOOK` secret. The site re-fetches the GitHub Releases API at build time and updates download links automatically
- **Per-release upkeep:** add a Markdown file at `src/content/changelog/<version>.md` in the website repo. **Do not** bump version constants — the API fetch handles that
- The `website/` directory in this repo is gitignored as a safety net; never commit website code here

## System Dependencies

- `fonts-noto-color-emoji` required for emoji rendering
