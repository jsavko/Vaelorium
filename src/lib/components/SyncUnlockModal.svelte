<script lang="ts">
  import { unlockSync } from '../api/sync'
  import { refreshSyncStatus } from '../stores/syncStore'
  import { showToast } from '../stores/toastStore'

  interface Props {
    open: boolean
    onClose: () => void
  }

  let { open, onClose }: Props = $props()
  let passphrase = $state('')
  let busy = $state(false)
  let error = $state<string | null>(null)

  async function handleUnlock() {
    if (!passphrase) return
    busy = true
    error = null
    try {
      await unlockSync(passphrase)
      passphrase = ''
      await refreshSyncStatus()
      showToast('Sync unlocked', 'success')
      onClose()
    } catch (e: any) {
      error = e?.message || String(e)
    } finally {
      busy = false
    }
  }

  function handleKey(e: KeyboardEvent) {
    if (e.key === 'Enter') handleUnlock()
    if (e.key === 'Escape') onClose()
  }
</script>

{#if open}
  <div class="modal-overlay" onclick={onClose} role="presentation">
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="modal-panel" onclick={(e) => e.stopPropagation()}>
      <h2 class="modal-title">Unlock sync</h2>
      <p class="modal-desc">
        Sync is configured for this Tome but locked. Enter your passphrase to resume syncing.
      </p>
      <!-- svelte-ignore a11y_autofocus -->
      <input
        type="password"
        class="modal-input"
        bind:value={passphrase}
        onkeydown={handleKey}
        autocomplete="current-password"
        placeholder="Passphrase"
        autofocus
      />
      {#if error}
        <p class="modal-error">{error}</p>
      {/if}
      <div class="modal-actions">
        <button class="modal-btn" onclick={onClose} disabled={busy}>Cancel</button>
        <button class="modal-btn primary" onclick={handleUnlock} disabled={busy || !passphrase}>
          {busy ? 'Unlocking…' : 'Unlock'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 300;
  }
  .modal-panel {
    background: var(--color-surface-card);
    border: 1px solid var(--color-accent-gold);
    border-radius: var(--radius-lg);
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.5);
    padding: 24px 28px;
    width: 420px;
    max-width: 90vw;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .modal-title {
    margin: 0;
    font-family: var(--font-heading);
    font-size: 20px;
    color: var(--color-fg-primary);
  }
  .modal-desc {
    margin: 0;
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-secondary);
    line-height: 1.5;
  }
  .modal-input {
    padding: 10px 14px;
    background: var(--color-surface-tertiary);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    color: var(--color-fg-primary);
    font-family: var(--font-ui);
    font-size: 14px;
  }
  .modal-input:focus { outline: 1px solid var(--color-accent-gold); }
  .modal-error {
    margin: 0;
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    background: rgba(184, 92, 92, 0.12);
    border: 1px solid var(--color-status-error);
    color: var(--color-fg-primary);
    font-family: var(--font-ui);
    font-size: 13px;
  }
  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 4px;
  }
  .modal-btn {
    padding: 8px 16px;
    border: 1px solid var(--color-border-default);
    background: transparent;
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-primary);
    cursor: pointer;
  }
  .modal-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .modal-btn.primary {
    background: var(--color-accent-gold);
    border-color: var(--color-accent-gold);
    color: var(--color-fg-inverse);
    font-weight: 600;
  }
</style>
