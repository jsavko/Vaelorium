//! Snapshot generation & bootstrap (Phase 2).
//!
//! Phase 1 leaves this as a placeholder. Snapshot mechanics depend on
//! op interception (Phase 2) and aren't useful in isolation.
//!
//! Planned API:
//! - `take_snapshot(tome_id, backend, key) -> Result<SnapshotId>`
//! - `restore_snapshot(tome_id, backend, snapshot_id) -> Result<()>`
//! - `should_snapshot(triggers: SnapshotTriggers) -> bool`
//!
//! Triggers (from the brainstorm, locked):
//! - weekly OR
//! - ~5MB / 5000 ops accrued OR
//! - Tome close OR
//! - manual
