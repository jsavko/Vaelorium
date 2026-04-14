<script lang="ts">
  interface Props {
    open: boolean
    title: string
    placeholder?: string
    confirmLabel?: string
    initialValue?: string
    onConfirm: (value: string) => void
    onCancel: () => void
  }

  let { open, title, placeholder = '', confirmLabel = 'OK', initialValue = '', onConfirm, onCancel }: Props = $props()

  let value = $state('')
  let inputEl = $state<HTMLInputElement | null>(null)

  function handleConfirm() {
    if (!value.trim()) return
    onConfirm(value.trim())
    value = ''
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!open) return
    if (e.key === 'Escape') { value = ''; onCancel() }
    if (e.key === 'Enter' && value.trim()) handleConfirm()
  }

  $effect(() => {
    if (open) {
      value = initialValue
      setTimeout(() => { inputEl?.focus(); inputEl?.select() }, 100)
    }
  })
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="overlay" onclick={() => { value = ''; onCancel() }} role="dialog" aria-modal="true" tabindex="-1">
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="modal" onclick={(e) => e.stopPropagation()}>
      <h3 class="modal-title">{title}</h3>
      <input
        bind:this={inputEl}
        class="modal-input"
        bind:value
        {placeholder}
      />
      <div class="modal-actions">
        <button class="cancel-btn" onclick={() => { value = ''; onCancel() }}>Cancel</button>
        <button class="confirm-btn" disabled={!value.trim()} onclick={handleConfirm}>{confirmLabel}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed; inset: 0; background: rgba(0, 0, 0, 0.5);
    display: flex; align-items: center; justify-content: center; z-index: 300;
  }
  .modal {
    background: var(--color-surface-card); border: 1px solid var(--color-border-default);
    border-radius: var(--radius-lg); padding: 24px; width: 380px;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.4);
    display: flex; flex-direction: column; gap: 16px;
  }
  .modal-title {
    font-family: var(--font-heading); font-size: 18px; font-weight: 600;
    color: var(--color-fg-primary); margin: 0;
  }
  .modal-input {
    width: 100%; padding: 8px 12px; background: var(--color-surface-primary);
    border: 1px solid var(--color-border-default); border-radius: var(--radius-sm);
    font-family: var(--font-ui); font-size: 14px; color: var(--color-fg-primary);
    outline: none; box-sizing: border-box;
  }
  .modal-input:focus { border-color: var(--color-accent-gold); }
  .modal-actions { display: flex; justify-content: flex-end; gap: 8px; }
  .cancel-btn {
    padding: 8px 16px; background: var(--color-surface-tertiary);
    border: 1px solid var(--color-border-default); border-radius: var(--radius-sm);
    font-family: var(--font-ui); font-size: 13px; color: var(--color-fg-secondary); cursor: pointer;
  }
  .confirm-btn {
    padding: 8px 20px; background: var(--color-accent-gold); border: none;
    border-radius: var(--radius-sm); font-family: var(--font-ui); font-size: 13px;
    font-weight: 600; color: var(--color-fg-inverse); cursor: pointer;
  }
  .confirm-btn:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
