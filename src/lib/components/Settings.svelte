<script lang="ts">
  import { refreshBackupStatus, refreshSyncStatus } from '../stores/syncStore'
  import SettingsKeybinds from './SettingsKeybinds.svelte'
  import SettingsAppearance from './SettingsAppearance.svelte'
  import SettingsData from './SettingsData.svelte'
  import SettingsBackup from './SettingsBackup.svelte'
  import SettingsSync from './SettingsSync.svelte'
  import SettingsAbout from './SettingsAbout.svelte'

  interface Props {
    open: boolean
    initialTab?: string
    onClose: () => void
    onOpenWizard?: () => void
  }

  let { open, initialTab, onClose, onOpenWizard }: Props = $props()

  let activeTab = $state('keybinds')

  // Each time the modal opens, jump to the requested tab if specified.
  // Without this, the tab the user last visited would persist across opens.
  $effect(() => {
    if (open && initialTab) activeTab = initialTab
  })

  const tabs = [
    { id: 'keybinds', label: 'Keybinds' },
    { id: 'appearance', label: 'Appearance' },
    { id: 'data', label: 'Data' },
    { id: 'backup', label: 'Backup' },
    { id: 'sync', label: 'Sync' },
    { id: 'about', label: 'About' },
  ]

  // Kick fresh status reads whenever the modal opens so tabs render
  // against live data. Individual tabs also refresh when they become
  // active (e.g. Backup's cloud-account refresh).
  $effect(() => {
    if (open) {
      refreshBackupStatus()
      refreshSyncStatus()
    }
  })
</script>

{#if open}
  <div class="settings-overlay" data-testid="settings">
    <div class="settings-panel">
      <div class="settings-sidebar">
        <h2 class="settings-title">Settings</h2>
        <nav class="settings-nav">
          {#each tabs as tab}
            <button
              class="settings-nav-item"
              class:active={activeTab === tab.id}
              onclick={() => activeTab = tab.id}
            >
              {tab.label}
            </button>
          {/each}
        </nav>
        <div class="settings-sidebar-spacer"></div>
        <button class="settings-close" onclick={onClose}>Close</button>
      </div>

      <div class="settings-content">
        {#if activeTab === 'keybinds'}
          <SettingsKeybinds />
        {:else if activeTab === 'appearance'}
          <SettingsAppearance />
        {:else if activeTab === 'data'}
          <SettingsData />
        {:else if activeTab === 'backup'}
          <SettingsBackup open={activeTab === 'backup' && open} {onClose} {onOpenWizard} />
        {:else if activeTab === 'sync'}
          <SettingsSync setActiveTab={(id) => activeTab = id} />
        {:else if activeTab === 'about'}
          <SettingsAbout />
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .settings-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }

  .settings-panel {
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-lg);
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.4);
    width: 700px;
    height: 500px;
    display: flex;
    overflow: hidden;
  }

  .settings-sidebar {
    width: 200px;
    background: var(--color-surface-secondary);
    padding: 24px 16px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .settings-title {
    font-family: var(--font-heading);
    font-size: 20px;
    font-weight: 600;
    color: var(--color-fg-primary);
    margin: 0 0 16px 8px;
  }

  .settings-nav-item {
    display: block;
    width: 100%;
    padding: 8px 12px;
    border: none;
    background: transparent;
    font-family: var(--font-ui);
    font-size: 14px;
    color: var(--color-fg-secondary);
    text-align: left;
    cursor: pointer;
    border-radius: var(--radius-sm);
  }

  .settings-nav-item:hover { background: var(--color-surface-tertiary); }

  .settings-nav-item.active {
    background: var(--color-accent-gold-subtle);
    color: var(--color-fg-primary);
  }

  .settings-sidebar-spacer { flex: 1; }

  .settings-close {
    padding: 8px 12px;
    border: none;
    background: transparent;
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-tertiary);
    cursor: pointer;
    border-radius: var(--radius-sm);
    text-align: left;
  }

  .settings-close:hover {
    background: var(--color-surface-tertiary);
    color: var(--color-fg-primary);
  }

  .settings-content {
    flex: 1;
    padding: 24px 32px;
    overflow-y: auto;
  }

  /* Shared controls used across every tab. Exposed as `:global()` so
     scoped-style hashing doesn't require each tab component to
     redeclare them. */
  :global(.settings-section) { display: flex; flex-direction: column; gap: 16px; }
  :global(.section-header-row) { display: flex; align-items: center; justify-content: space-between; }
  :global(.settings-section-title) {
    font-family: var(--font-ui); font-size: 16px; font-weight: 600;
    color: var(--color-fg-primary); margin: 0;
  }
  :global(.data-actions) { display: flex; gap: 8px; margin-bottom: 8px; }
  :global(.data-btn) {
    padding: 8px 16px;
    background: var(--color-surface-tertiary);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui); font-size: 13px;
    color: var(--color-fg-primary); cursor: pointer;
  }
  :global(.data-btn:hover) { border-color: var(--color-accent-gold); }
  :global(.data-btn:disabled) { opacity: 0.6; cursor: not-allowed; }
  :global(.data-btn.primary) {
    background: var(--color-accent-gold);
    color: var(--color-fg-inverse);
    border-color: var(--color-accent-gold);
    font-weight: 600;
  }
  :global(.data-btn.danger) {
    color: var(--color-status-error);
    border-color: var(--color-status-error);
  }
  :global(.data-desc) {
    font-family: var(--font-ui); font-size: 12px;
    color: var(--color-fg-tertiary); margin: 0;
  }
</style>
