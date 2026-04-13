<script lang="ts">
  import { toasts, dismissToast } from '../stores/toastStore'
</script>

{#if $toasts.length > 0}
  <div class="toast-container">
    {#each $toasts as toast (toast.id)}
      <div class="toast" class:success={toast.type === 'success'} class:error={toast.type === 'error'}>
        <span class="toast-message">{toast.message}</span>
        {#if toast.action}
          <button
            class="toast-action"
            onclick={() => { toast.action!.onClick(); dismissToast(toast.id) }}
          >{toast.action.label}</button>
        {/if}
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    top: 24px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    flex-direction: column;
    gap: 8px;
    z-index: 9999;
    pointer-events: none;
    max-width: 90vw;
  }
  .toast { pointer-events: auto; }

  .toast {
    padding: 12px 20px;
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-md);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-primary);
    animation: slideIn 0.2s ease-out;
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .toast-message { flex: 1; }
  .toast-action {
    background: transparent;
    border: 1px solid var(--color-accent-gold);
    color: var(--color-accent-gold);
    padding: 4px 12px;
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
  }
  .toast-action:hover { background: var(--color-accent-gold); color: var(--color-fg-inverse); }

  .toast.success {
    border-left: 3px solid var(--color-status-success);
  }

  .toast.error {
    border-left: 3px solid var(--color-status-error);
  }

  @keyframes slideIn {
    from { transform: translateY(-20px); opacity: 0; }
    to { transform: translateY(0); opacity: 1; }
  }
</style>
