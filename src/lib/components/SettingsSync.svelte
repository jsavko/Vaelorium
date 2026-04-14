<script lang="ts">
  import { isTauri } from '../api/bridge'
  import { showToast } from '../stores/toastStore'
  import {
    syncStatus, backupStatus, syncActivity, restorableTomes,
    refreshSyncStatus, refreshActivity,
  } from '../stores/syncStore'
  import { enableSync, disableSync, syncNow, takeSnapshot } from '../api/sync'
  import { cloudAccount, atTomeQuota } from '../stores/cloudStore'
  import { currentTome, recentTomes } from '../stores/tomeStore'

  interface Props {
    setActiveTab: (id: string) => void
  }
  let { setActiveTab }: Props = $props()

  let syncBusy = $state(false)
  let activityCollapsed = $state(localStorage.getItem('vaelorium-sync-activity-collapsed') === 'true')

  function toggleActivityCollapsed() {
    activityCollapsed = !activityCollapsed
    localStorage.setItem('vaelorium-sync-activity-collapsed', String(activityCollapsed))
  }

  function relativeAge(iso: string): string {
    const d = new Date(iso)
    const diffSec = Math.floor((Date.now() - d.getTime()) / 1000)
    if (diffSec < 5) return 'just now'
    if (diffSec < 60) return `${diffSec}s ago`
    const min = Math.floor(diffSec / 60)
    if (min < 60) return `${min} min ago`
    const hr = Math.floor(min / 60)
    if (hr < 24) return `${hr} hr ago`
    const days = Math.floor(hr / 24)
    if (days < 7) return `${days}d ago`
    return d.toLocaleDateString()
  }

  function activitySummary(r: { opsUploaded: number; opsApplied: number; conflictsCreated: number; durationMs: number; snapshotTaken: string | null }): string {
    const parts: string[] = []
    if (r.opsUploaded || r.opsApplied) parts.push(`⇡${r.opsUploaded} ⇣${r.opsApplied}`)
    if (r.conflictsCreated) parts.push(`${r.conflictsCreated}c`)
    if (r.snapshotTaken) parts.push('📸 snapshot')
    parts.push(`${r.durationMs}ms`)
    return parts.join(' · ')
  }

  // UUID-aware quota check for the "Back up this Tome" CTA: if this
  // Tome's UUID is already on the backend (user stop-synced earlier
  // and is re-enabling), quota shouldn't reject it.
  let currentTomeUuid = $derived(
    $recentTomes.find((t) => t.path === $currentTome?.path)?.tome_uuid ?? null,
  )
  let currentTomeAtQuota = $derived(
    $atTomeQuota && !(currentTomeUuid && $restorableTomes.some((t) => t.tomeUuid === currentTomeUuid)),
  )

  async function handleEnableSync() {
    const tome = $currentTome
    if (!tome) return
    syncBusy = true
    try {
      await enableSync({ tomeId: tome.path })
      await refreshSyncStatus()
      showToast('Sync enabled for this Tome', 'success')
    } catch (e: any) {
      showToast(`Enable failed: ${e.message || e}`, 'error')
    } finally { syncBusy = false }
  }

  async function handleDisableSync() {
    if ($syncStatus.tomeId) {
      syncBusy = true
      try {
        await disableSync($syncStatus.tomeId)
        await refreshSyncStatus()
        showToast('Sync disabled', 'success')
      } finally { syncBusy = false }
    }
  }

  async function handleSyncNow() {
    syncBusy = true
    try {
      const out = await syncNow()
      await refreshSyncStatus()
      const msg = `Synced — ${out.ops_uploaded} up, ${out.ops_applied} down`
      showToast(out.error ? `Sync error: ${out.error}` : msg, out.error ? 'error' : 'success')
    } catch (e: any) {
      showToast(`Sync failed: ${e.message || e}`, 'error')
    } finally { syncBusy = false }
  }

  async function handleTakeSnapshot() {
    syncBusy = true
    try {
      await takeSnapshot()
      await refreshSyncStatus()
      showToast('Snapshot taken', 'success')
    } catch (e: any) {
      showToast(`Snapshot failed: ${e.message || e}`, 'error')
    } finally { syncBusy = false }
  }
</script>

<div class="settings-section">
  <h3 class="settings-section-title">Sync this Tome</h3>
  {#if !isTauri}
    <p class="data-desc">Sync is only available in the desktop app.</p>
  {:else if $syncStatus.backupMissing}
    <p class="data-desc">
      No backup destination is configured. Set one up in the
      <strong>Backup</strong> tab first, then come back here to enable sync for this Tome.
    </p>
    <button class="data-btn" onclick={() => setActiveTab('backup')}>Go to Backup settings</button>
  {:else if $backupStatus.locked}
    <p class="data-desc">
      The backup is locked. Unlock it in the <strong>Backup</strong> tab to resume syncing.
    </p>
    <button class="data-btn" onclick={() => setActiveTab('backup')}>Go to Backup settings</button>
  {:else if !$syncStatus.enabled}
    <p class="data-desc">
      Sync is off for this Tome. Enable it to back up and sync this Tome to the configured destination
      ({$syncStatus.backendKind} — {$syncStatus.backendSummary}).
    </p>
    {#if currentTomeAtQuota && $cloudAccount?.usage}
      <div class="quota-banner">
        <strong>You're at your Vaelorium Cloud plan's Tome limit ({$cloudAccount.usage.tomeCount} of {$cloudAccount.usage.tomeLimit}).</strong>
        <span>Enabling backup for this Tome will fail until you upgrade your plan or remove an existing Tome from backup.</span>
      </div>
    {/if}
    <div class="sync-actions">
      <button class="data-btn primary" onclick={handleEnableSync} disabled={syncBusy || currentTomeAtQuota}>
        {syncBusy ? 'Enabling…' : 'Back up this Tome'}
      </button>
    </div>
  {:else}
    <div class="sync-status-card">
      <div class="sync-status-row">
        <span class="sync-status-label">Status</span>
        <span class="sync-status-value">Enabled</span>
      </div>
      <div class="sync-status-row">
        <span class="sync-status-label">Backend</span>
        <span class="sync-status-value">{$syncStatus.backendKind} — {$syncStatus.backendSummary}</span>
      </div>
      <div class="sync-status-row">
        <span class="sync-status-label">Device</span>
        <span class="sync-status-value">{$syncStatus.deviceName}</span>
      </div>
      <div class="sync-status-row">
        <span class="sync-status-label">Last sync</span>
        <span class="sync-status-value">
          {$syncStatus.lastSyncAt ? new Date($syncStatus.lastSyncAt).toLocaleString() : 'never'}
        </span>
      </div>
      <div class="sync-status-row">
        <span class="sync-status-label">Pending uploads</span>
        <span class="sync-status-value">{$syncStatus.queueSize}</span>
      </div>
      <div class="sync-status-row">
        <span class="sync-status-label">Unresolved conflicts</span>
        <span class="sync-status-value" class:warn={$syncStatus.pendingConflicts > 0}>
          {$syncStatus.pendingConflicts}
        </span>
      </div>
      {#if $syncStatus.lastError}
        <p class="sync-error">Last error: {$syncStatus.lastError}</p>
      {/if}
    </div>
    <div class="sync-actions">
      <button class="data-btn" onclick={handleSyncNow} disabled={syncBusy}>
        {#if syncBusy}<span class="spinner" aria-hidden="true"></span>Syncing…{:else}Sync now{/if}
      </button>
      <button class="data-btn" onclick={handleTakeSnapshot} disabled={syncBusy}>
        {#if syncBusy}<span class="spinner" aria-hidden="true"></span>Working…{:else}Take snapshot{/if}
      </button>
      <button class="data-btn danger" onclick={handleDisableSync} disabled={syncBusy}>Stop syncing this Tome</button>
    </div>

    <div class="activity-section">
      <button class="activity-header" type="button" onclick={() => { toggleActivityCollapsed(); if (!activityCollapsed) refreshActivity() }}>
        <span class="activity-title">Recent activity</span>
        <span class="activity-count">{$syncActivity.length}</span>
        <span class="activity-chevron" class:collapsed={activityCollapsed}>▾</span>
      </button>
      {#if !activityCollapsed}
        {#if $syncActivity.length === 0}
          <p class="activity-empty">No sync activity recorded yet.</p>
        {:else}
          <ul class="activity-list">
            {#each $syncActivity as r (r.id)}
              <li class="activity-row" class:error={r.outcome === 'error'}>
                <span class="activity-icon">{r.outcome === 'error' ? '✗' : '✓'}</span>
                <span class="activity-time" title={r.startedAt}>{relativeAge(r.startedAt)}</span>
                <span class="activity-summary">{activitySummary(r)}</span>
                {#if r.error}
                  <span class="activity-err" title={r.error}>{r.error}</span>
                {/if}
              </li>
            {/each}
          </ul>
        {/if}
      {/if}
    </div>
  {/if}
</div>

<style>
  .quota-banner {
    display: flex; flex-direction: column; gap: 4px;
    padding: 10px 12px; margin: 8px 0;
    background: color-mix(in srgb, var(--color-warning, #c58b3a) 12%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-warning, #c58b3a) 45%, transparent);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui); font-size: 12px;
    color: var(--color-fg-secondary);
  }
  .quota-banner strong { color: var(--color-fg-primary); font-weight: 600; }
  .sync-actions { display: flex; gap: 8px; flex-wrap: wrap; margin-top: 8px; }
  .sync-error {
    color: var(--color-status-error);
    font-family: var(--font-ui); font-size: 13px; margin: 0;
  }
  .sync-status-card {
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-md);
    padding: 14px 16px; margin: 12px 0;
    display: flex; flex-direction: column; gap: 6px;
  }
  .sync-status-row {
    display: flex; justify-content: space-between; align-items: baseline;
    font-family: var(--font-ui); font-size: 13px;
  }
  .sync-status-label { color: var(--color-fg-tertiary); }
  .sync-status-value { color: var(--color-fg-primary); font-weight: 500; }
  .sync-status-value.warn { color: var(--color-status-warning); }

  .activity-section { margin-top: 16px; }
  .activity-header {
    display: flex; align-items: center; gap: 8px; width: 100%;
    background: none; border: none; padding: 8px 0; cursor: pointer;
    font-family: var(--font-ui); font-size: 12px; font-weight: 600;
    color: var(--color-fg-secondary); letter-spacing: 0.5px;
    text-transform: uppercase;
  }
  .activity-title { flex: 1; text-align: left; }
  .activity-count {
    font-weight: 500; font-size: 11px;
    color: var(--color-fg-tertiary);
    background: var(--color-surface-tertiary);
    padding: 1px 8px; border-radius: 10px;
  }
  .activity-chevron { transition: transform 0.15s; }
  .activity-chevron.collapsed { transform: rotate(-90deg); }
  .activity-empty {
    font-family: var(--font-ui); font-size: 12px;
    color: var(--color-fg-tertiary); margin: 4px 0 0;
  }
  .activity-list {
    list-style: none; padding: 0; margin: 4px 0 0;
    max-height: 280px; overflow-y: auto;
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
  }
  .activity-row {
    display: flex; align-items: center; gap: 10px;
    padding: 6px 10px;
    font-family: var(--font-ui); font-size: 12px;
    color: var(--color-fg-secondary);
    border-bottom: 1px solid var(--color-border-default);
  }
  .activity-row:last-child { border-bottom: none; }
  .activity-row.error .activity-icon { color: var(--color-status-error, #d97474); }
  .activity-row .activity-icon { color: var(--color-status-success, #6fb37e); width: 12px; }
  .activity-time { color: var(--color-fg-tertiary); width: 90px; }
  .spinner {
    display: inline-block;
    width: 10px; height: 10px;
    margin-right: 6px;
    border: 2px solid transparent;
    border-top-color: currentColor;
    border-right-color: currentColor;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    vertical-align: -1px;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
