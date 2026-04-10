<script lang="ts">
  import { toasts } from '../stores/toastStore'
</script>

{#if $toasts.length > 0}
  <div class="toast-container">
    {#each $toasts as toast (toast.id)}
      <div class="toast" class:success={toast.type === 'success'} class:error={toast.type === 'error'}>
        {toast.message}
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    bottom: 24px;
    right: 24px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    z-index: 400;
  }

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
  }

  .toast.success {
    border-left: 3px solid var(--color-status-success);
  }

  .toast.error {
    border-left: 3px solid var(--color-status-error);
  }

  @keyframes slideIn {
    from { transform: translateX(20px); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }
</style>
