<script lang="ts">
  import { createTome } from '../stores/tomeStore'

  interface Props {
    open: boolean
    onClose: () => void
  }

  let { open, onClose }: Props = $props()

  let name = $state('')
  let description = $state('')
  let saving = $state(false)

  async function handleCreate() {
    if (!name.trim()) return
    saving = true
    try {
      // For browser mock, use a synthetic path. In Tauri, this would be a file dialog result.
      const path = `/tomes/${name.trim().toLowerCase().replace(/\s+/g, '-')}.vaelorium`
      await createTome(path, name.trim(), description.trim() || undefined)
      resetAndClose()
    } finally {
      saving = false
    }
  }

  function resetAndClose() {
    name = ''
    description = ''
    onClose()
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!open) return
    if (e.key === 'Escape') resetAndClose()
    if (e.key === 'Enter' && name.trim() && !saving) handleCreate()
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="overlay" onclick={resetAndClose} role="dialog" aria-modal="true">
    <div class="modal" onclick={(e) => e.stopPropagation()}>
      <h2 class="modal-title">Create New Tome</h2>
      <p class="modal-desc">A Tome is a self-contained world — all your pages, maps, and settings in one file.</p>

      <div class="form-field">
        <label class="field-label" for="tome-name">Name</label>
        <input
          id="tome-name"
          class="field-input"
          bind:value={name}
          placeholder="e.g. The Bolagian Chronicle"
        />
      </div>

      <div class="form-field">
        <label class="field-label" for="tome-desc">Description (optional)</label>
        <textarea
          id="tome-desc"
          class="field-textarea"
          bind:value={description}
          placeholder="A sprawling campaign across..."
          rows="3"
        ></textarea>
      </div>

      <div class="divider"></div>

      <div class="modal-footer">
        <button class="cancel-btn" onclick={resetAndClose}>Cancel</button>
        <button
          class="create-btn"
          disabled={!name.trim() || saving}
          onclick={handleCreate}
        >
          {saving ? 'Creating...' : 'Create Tome'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 300;
  }

  .modal {
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-lg);
    padding: 28px;
    width: 520px;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.4);
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .modal-title {
    font-family: var(--font-heading);
    font-size: 24px;
    font-weight: 600;
    color: var(--color-fg-primary);
    margin: 0;
  }

  .modal-desc {
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-tertiary);
    margin: 0;
  }

  .form-field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .field-label {
    font-family: var(--font-ui);
    font-size: 12px;
    font-weight: 600;
    color: var(--color-fg-tertiary);
  }

  .field-input,
  .field-textarea {
    width: 100%;
    padding: 8px 12px;
    background: var(--color-surface-primary);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 14px;
    color: var(--color-fg-primary);
    outline: none;
    box-sizing: border-box;
  }

  .field-input:focus,
  .field-textarea:focus {
    border-color: var(--color-accent-gold);
  }

  .field-textarea {
    resize: vertical;
    min-height: 60px;
  }

  .divider {
    height: 1px;
    background: var(--color-border-subtle);
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .cancel-btn {
    padding: 8px 16px;
    background: var(--color-surface-tertiary);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-secondary);
    cursor: pointer;
  }

  .create-btn {
    padding: 8px 20px;
    background: var(--color-accent-gold);
    border: none;
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 13px;
    font-weight: 600;
    color: var(--color-fg-inverse);
    cursor: pointer;
  }

  .create-btn:hover:not(:disabled) {
    background: var(--color-accent-gold-hover);
  }

  .create-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
