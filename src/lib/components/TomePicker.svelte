<script lang="ts">
  import { BookOpen, FolderOpen, Plus, Map as MapIcon } from 'lucide-svelte'
  import { recentTomes, openTome, loadRecentTomes } from '../stores/tomeStore'
  import { onMount } from 'svelte'
  import type { RecentTome } from '../api/tomes'

  interface Props {
    onCreateNew: () => void
  }

  let { onCreateNew }: Props = $props()

  onMount(() => {
    loadRecentTomes()
  })

  async function handleOpenRecent(tome: RecentTome) {
    await openTome(tome.path)
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

    <div class="picker-footer">
      <button class="open-file-btn" onclick={onCreateNew}>
        <FolderOpen size={16} />
        <span>Open Tome File</span>
      </button>
      <span class="drag-hint">or drag a .vaelorium file here</span>
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
</style>
