<script lang="ts">
  import BacklinksPanel from './BacklinksPanel.svelte'
  import { currentPage } from '../stores/pageStore'

  interface Props {
    open: boolean
    onClose: () => void
  }

  let { open, onClose }: Props = $props()
</script>

{#if open && $currentPage}
  <div class="panel-divider"></div>
  <aside class="details-panel">
    <header class="panel-header">
      <span class="panel-title">Details</span>
      <button class="close-btn" onclick={onClose} aria-label="Close panel">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </button>
    </header>

    <div class="panel-divider-h"></div>

    <div class="panel-content">
      <!-- Entity fields placeholder for Milestone 2 -->
      <div class="section">
        <h3 class="section-label">PAGE INFO</h3>
        <div class="field">
          <span class="field-label">Visibility</span>
          <span class="field-value">{$currentPage.visibility}</span>
        </div>
        <div class="field">
          <span class="field-label">Created</span>
          <span class="field-value">{new Date($currentPage.created_at).toLocaleDateString()}</span>
        </div>
        <div class="field">
          <span class="field-label">Last edited</span>
          <span class="field-value">{new Date($currentPage.updated_at).toLocaleDateString()}</span>
        </div>
      </div>

      <div class="section-divider"></div>

      <BacklinksPanel />
    </div>
  </aside>
{/if}

<style>
  .details-panel {
    width: 320px;
    min-width: 320px;
    height: 100%;
    background: var(--color-surface-secondary);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .panel-divider {
    width: 1px;
    background: var(--color-border-subtle);
    flex-shrink: 0;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
  }

  .panel-title {
    font-family: var(--font-ui);
    font-size: 14px;
    font-weight: 600;
    color: var(--color-fg-primary);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--color-fg-tertiary);
    cursor: pointer;
    padding: 2px;
  }

  .close-btn:hover {
    color: var(--color-fg-primary);
  }

  .panel-divider-h {
    height: 1px;
    background: var(--color-border-subtle);
  }

  .panel-content {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .section-label {
    font-family: var(--font-ui);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 2px;
    color: var(--color-fg-tertiary);
    margin: 0;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .field-label {
    font-family: var(--font-ui);
    font-size: 12px;
    font-weight: 500;
    color: var(--color-fg-tertiary);
  }

  .field-value {
    font-family: var(--font-ui);
    font-size: 14px;
    color: var(--color-fg-primary);
    text-transform: capitalize;
  }

  .section-divider {
    height: 1px;
    background: var(--color-border-subtle);
  }
</style>
