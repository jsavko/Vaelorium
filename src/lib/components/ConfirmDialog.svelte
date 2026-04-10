<script lang="ts">
  interface Props {
    open: boolean
    title: string
    message: string
    confirmLabel?: string
    danger?: boolean
    onConfirm: () => void
    onCancel: () => void
  }

  let { open, title, message, confirmLabel = 'Confirm', danger = false, onConfirm, onCancel }: Props = $props()

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onCancel()
    if (e.key === 'Enter') onConfirm()
  }
</script>

{#if open}
  <div class="overlay" onclick={onCancel} onkeydown={handleKeydown} role="dialog" aria-modal="true">
    <div class="dialog" onclick={(e) => e.stopPropagation()}>
      <h2 class="dialog-title">{title}</h2>
      <p class="dialog-message">{message}</p>
      <div class="dialog-actions">
        <button class="btn-cancel" onclick={onCancel}>Cancel</button>
        <button class="btn-confirm" class:danger onclick={onConfirm}>{confirmLabel}</button>
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

  .dialog {
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-lg);
    padding: 24px;
    width: 400px;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.4);
  }

  .dialog-title {
    font-family: var(--font-ui);
    font-size: 16px;
    font-weight: 600;
    color: var(--color-fg-primary);
    margin: 0 0 8px;
  }

  .dialog-message {
    font-family: var(--font-ui);
    font-size: 14px;
    color: var(--color-fg-secondary);
    margin: 0 0 20px;
  }

  .dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .btn-cancel {
    padding: 8px 16px;
    background: var(--color-surface-tertiary);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-secondary);
    cursor: pointer;
  }

  .btn-confirm {
    padding: 8px 16px;
    background: var(--color-accent-gold);
    border: none;
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 13px;
    font-weight: 600;
    color: var(--color-fg-inverse);
    cursor: pointer;
  }

  .btn-confirm.danger {
    background: var(--color-status-error);
    color: white;
  }
</style>
