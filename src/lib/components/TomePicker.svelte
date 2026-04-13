<script lang="ts">
  import { BookOpen, FolderOpen, Plus, Map as MapIcon, DownloadCloud, Lock } from 'lucide-svelte'
  import { recentTomes, openTome, loadRecentTomes } from '../stores/tomeStore'
  import { isTauri } from '../api/bridge'
  import { onMount } from 'svelte'
  import type { RecentTome } from '../api/tomes'
  import {
    getBackupStatus,
    listRestorableTomes,
    restoreTomeFromBackup,
    type RestorableTome,
  } from '../api/backup'

  interface Props {
    onCreateNew: () => void
    onOpenSettings?: (tab?: string) => void
    onOpenWizard?: () => void
  }

  let { onCreateNew, onOpenSettings, onOpenWizard }: Props = $props()

  let backupConfigured = $state(false)
  let backupLocked = $state(false)
  let restorable = $state<RestorableTome[]>([])
  let restorableLoading = $state(false)
  let restorableError = $state<string | null>(null)
  let restoringUuid = $state<string | null>(null)

  onMount(async () => {
    loadRecentTomes()
    await refreshBackupSection()
  })

  async function refreshBackupSection() {
    if (!isTauri) return
    try {
      const status = await getBackupStatus()
      backupConfigured = status.configured
      backupLocked = status.locked
      if (status.configured && !status.locked) {
        await loadRestorable()
      }
    } catch (e) {
      console.warn('backup status fetch failed', e)
    }
  }

  async function loadRestorable() {
    restorableLoading = true
    restorableError = null
    try {
      restorable = await listRestorableTomes()
    } catch (e) {
      restorableError = e instanceof Error ? e.message : String(e)
      restorable = []
    } finally {
      restorableLoading = false
    }
  }

  async function handleRestore(t: RestorableTome) {
    restoringUuid = t.tomeUuid
    try {
      const restored = await restoreTomeFromBackup(t.tomeUuid)
      await openTome(restored.path)
    } catch (e) {
      restorableError = e instanceof Error ? e.message : String(e)
    } finally {
      restoringUuid = null
    }
  }

  function formatBytes(n: number): string {
    if (n < 1024) return `${n} B`
    if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`
    return `${(n / 1024 / 1024).toFixed(1)} MB`
  }

  function relativeAge(iso: string): string {
    const d = new Date(iso)
    const diffMin = Math.floor((Date.now() - d.getTime()) / 60_000)
    if (diffMin < 1) return 'just now'
    if (diffMin < 60) return `${diffMin} min ago`
    const hr = Math.floor(diffMin / 60)
    if (hr < 24) return `${hr} hr ago`
    const days = Math.floor(hr / 24)
    if (days < 30) return `${days}d ago`
    return d.toLocaleDateString()
  }

  async function handleOpenRecent(tome: RecentTome) {
    await openTome(tome.path)
  }

  async function handleOpenFile() {
    if (isTauri) {
      const { open } = await import('@tauri-apps/plugin-dialog')
      const path = await open({
        filters: [{ name: 'Vaelorium Tome', extensions: ['tome', 'vaelorium'] }],
        multiple: false,
      })
      if (path) {
        await openTome(path as string)
      }
    }
  }

  function formatDate(iso: string): string {
    const d = new Date(iso)
    const now = new Date()
    const diff = now.getTime() - d.getTime()
    const days = Math.floor(diff / (1000 * 60 * 60 * 24))
    if (days === 0) return 'Today'
    if (days === 1) return 'Yesterday'
    if (days < 7) return `${days} days ago`
    return d.toLocaleDateString()
  }
</script>

<div class="tome-picker">
  <div class="picker-content">
    <header class="picker-header">
      <h1 class="picker-logo">Vaelorium</h1>
      <p class="picker-subtitle">The Arcane Library</p>
    </header>

    {#if $recentTomes.length > 0}
      <div class="recent-section">
        <span class="section-label">RECENT TOMES</span>
        <div class="tome-grid">
          {#each $recentTomes as tome (tome.path)}
            <button class="tome-card" onclick={() => handleOpenRecent(tome)}>
              <div class="card-cover">
                <BookOpen size={32} />
              </div>
              <div class="card-body">
                <h3 class="card-title">{tome.name}</h3>
                {#if tome.description}
                  <p class="card-desc">{tome.description}</p>
                {/if}
                <span class="card-date">Last opened: {formatDate(tome.last_opened)}</span>
              </div>
            </button>
          {/each}

          <button class="tome-card new-card" onclick={onCreateNew}>
            <div class="new-card-content">
              <Plus size={32} />
              <span class="new-label">Create New Tome</span>
              <span class="new-desc">Start a new world or campaign</span>
            </div>
          </button>
        </div>
      </div>
    {:else}
      <div class="empty-state">
        <p class="empty-text">No recent Tomes</p>
        <button class="empty-create" onclick={onCreateNew}>
          Create Your First Tome
        </button>
      </div>
    {/if}

    {#if isTauri}
      <div class="restore-section">
        <span class="section-label">RESTORE FROM BACKUP</span>
        {#if !backupConfigured}
          <button
            class="restore-prompt"
            onclick={() => onOpenWizard?.()}
            type="button"
          >
            <DownloadCloud size={16} />
            <span>Set up a backup destination to recover Tomes from another device</span>
          </button>
        {:else if backupLocked}
          <button
            class="restore-prompt"
            onclick={() => onOpenSettings?.('backup')}
            type="button"
          >
            <Lock size={16} />
            <span>Backup is locked — unlock it to see Tomes available on this backend</span>
          </button>
        {:else if restorableLoading}
          <p class="restore-status">Looking for Tomes on backup…</p>
        {:else if restorableError}
          <p class="restore-error">Could not list backup: {restorableError}</p>
        {:else if restorable.length === 0}
          <p class="restore-status">No Tomes found on this backend.</p>
        {:else}
          <div class="restore-list">
            {#each restorable as t (t.tomeUuid)}
              <div class="restore-row">
                <DownloadCloud size={20} class="restore-icon" />
                <div class="restore-meta">
                  <h4 class="restore-name">{t.name}</h4>
                  <span class="restore-sub">
                    {formatBytes(t.sizeBytes)} · snapshot {relativeAge(t.lastModified)}
                  </span>
                </div>
                <button
                  class="restore-btn"
                  onclick={() => handleRestore(t)}
                  disabled={restoringUuid !== null}
                  type="button"
                >
                  {restoringUuid === t.tomeUuid ? 'Restoring…' : 'Restore'}
                </button>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}

    <div class="picker-footer">
      <button class="open-file-btn" onclick={handleOpenFile}>
        <FolderOpen size={16} />
        <span>Open Tome File</span>
      </button>
      <span class="drag-hint">or drag a .tome file here</span>
    </div>
  </div>
</div>

<style>
  .tome-picker {
    width: 100%;
    height: 100%;
    background: var(--color-surface-primary);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .picker-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 40px;
    max-width: 900px;
    width: 100%;
    padding: 60px;
  }

  .picker-header {
    text-align: center;
  }

  .picker-logo {
    font-family: var(--font-heading);
    font-size: 48px;
    font-weight: 700;
    color: var(--color-accent-gold);
    margin: 0 0 8px;
  }

  .picker-subtitle {
    font-family: var(--font-body);
    font-size: 18px;
    color: var(--color-fg-tertiary);
    margin: 0;
  }

  .recent-section {
    width: 100%;
  }

  .section-label {
    display: block;
    font-family: var(--font-ui);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 2px;
    color: var(--color-fg-tertiary);
    margin-bottom: 12px;
  }

  .tome-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 20px;
  }

  .tome-card {
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .tome-card:hover {
    border-color: var(--color-accent-gold);
  }

  .card-cover {
    height: 100px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(180deg, var(--color-surface-tertiary), var(--color-surface-primary));
    color: var(--color-accent-gold);
    opacity: 0.3;
  }

  .card-body {
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .card-title {
    font-family: var(--font-heading);
    font-size: 16px;
    font-weight: 600;
    color: var(--color-fg-primary);
    margin: 0;
  }

  .card-desc {
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-fg-tertiary);
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .card-date {
    font-family: var(--font-ui);
    font-size: 11px;
    color: var(--color-fg-tertiary);
    opacity: 0.6;
  }

  .new-card {
    border-style: dashed;
  }

  .new-card-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 32px;
    flex: 1;
    color: var(--color-accent-gold);
  }

  .new-label {
    font-family: var(--font-ui);
    font-size: 14px;
    font-weight: 600;
  }

  .new-desc {
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-fg-tertiary);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    padding: 40px;
  }

  .empty-text {
    font-family: var(--font-ui);
    font-size: 16px;
    color: var(--color-fg-tertiary);
    margin: 0;
  }

  .empty-create {
    padding: 10px 24px;
    background: var(--color-accent-gold);
    border: none;
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 14px;
    font-weight: 600;
    color: var(--color-fg-inverse);
    cursor: pointer;
  }

  .picker-footer {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .open-file-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    background: var(--color-surface-tertiary);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-secondary);
    cursor: pointer;
  }

  .open-file-btn:hover {
    border-color: var(--color-accent-gold);
    color: var(--color-fg-primary);
  }

  .drag-hint {
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-fg-tertiary);
    opacity: 0.5;
  }

  .restore-section {
    width: 100%;
  }

  .restore-status,
  .restore-error {
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-tertiary);
    margin: 0;
    padding: 12px 14px;
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
  }

  .restore-error {
    color: var(--color-status-error, #d97474);
  }

  .restore-prompt {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 12px 14px;
    background: var(--color-surface-card);
    border: 1px dashed var(--color-border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-secondary);
    cursor: pointer;
    text-align: left;
  }

  .restore-prompt:hover {
    border-color: var(--color-accent-gold);
    color: var(--color-fg-primary);
  }

  .restore-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .restore-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
  }

  .restore-meta {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow: hidden;
  }

  .restore-name {
    font-family: var(--font-heading);
    font-size: 14px;
    font-weight: 600;
    color: var(--color-fg-primary);
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .restore-sub {
    font-family: var(--font-ui);
    font-size: 11px;
    color: var(--color-fg-tertiary);
  }

  .restore-btn {
    padding: 6px 14px;
    background: var(--color-accent-gold);
    border: none;
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 12px;
    font-weight: 600;
    color: var(--color-fg-inverse);
    cursor: pointer;
  }

  .restore-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
