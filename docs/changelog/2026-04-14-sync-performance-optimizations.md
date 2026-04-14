# Sync performance: concurrent journal I/O + HTTP gzip

**Date:** 2026-04-14

## Summary

Ships the app-side Phase 1 + Phase 3 of the sync-performance plan. Phase 2 (sync-meta short-circuit) is blocked on a cloud-side change and tracked on the cross-repo review queue. Phase 4 (optimistic UI toast) is deferred pending real-world measurement of the Phase 1 speedup.

## Changes

### Performance
- **Concurrent journal uploads and downloads.** `sync::engine::sync_tome_once` replaces the sequential `for` loops over local/remote ops with `futures::stream::iter(...).buffer_unordered(8).collect()`. HTTP/2 multiplexes all in-flight requests over a single TLS connection held by the `HostedBackend`. N sequential `put_object` / `get_object` calls collapse to roughly one round-trip of wall time — 3–4× faster for multi-op cycles.
- **HTTP-level gzip decoding.** `reqwest` gains the `gzip` feature flag so responses carrying `Content-Encoding: gzip` auto-decompress. JSON responses (`/list`, `/account`) benefit if the cloud sends gzip (Cloudflare default or middleware).

### Cross-repo
- Filed four items on `~/Projects/vaelorium-cloud/docs/review-queue.md` under a new "Sync performance" section:
  - Add `latest_op_ulid` to `sync-meta.json` so the client can short-circuit idle polls.
  - Honor `If-None-Match` on the sync-meta GET endpoint.
  - Verify `Content-Encoding: gzip` on JSON responses.
  - (Deferred) a batch-journal PUT endpoint if post-Phase-1 metrics still show upload concurrency issues.

### Files Modified
- `src-tauri/src/sync/engine.rs` — new `JOURNAL_CONCURRENCY` const; upload and download loops refactored to `buffer_unordered`; `max_uploaded_id` + `uploaded_count` pre-computed so ownership of `prepared` can move into the concurrent stream.
- `src-tauri/Cargo.toml` — `reqwest` gains `"gzip"`; new `futures = "0.3"` dependency.
- `~/Projects/vaelorium-cloud/docs/review-queue.md` — four new items flagged.

### Verification
- `cargo check --lib` clean (pre-existing warnings only).
- `cargo test --lib` — all 84 tests passing.
- Runtime verification (actual wall-clock improvement against `cloud.vaelorium.com`) is on the user — Tauri + WSL can't drive timed sync measurements through Claude.

### Rationale

Current sync cycle was round-trip-bound, not bytes-bound. N PUTs + M GETs strictly serialized by a `for ... .await` loop in the engine. HTTP/2 multiplexing over a single TLS connection collapses that cost almost entirely when we run them through `buffer_unordered`. Bigger wins (sync-meta short-circuit to skip the cycle entirely when idle, batch endpoints) need cloud protocol evolution and are tracked upstream.

Gzipping journal ops before encryption was explicitly skipped (plan approach #5): tiny per-op size savings don't move the needle when latency dominates, and adding an envelope-version byte for the compatibility path has real cost. Revisit only if snapshots stop being the dominant bytes-on-wire concern.
