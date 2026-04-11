<script lang="ts">
  import BacklinksPanel from './BacklinksPanel.svelte'
  import RelationsPanel from './RelationsPanel.svelte'
  import EntityFields from './EntityFields.svelte'
  import TagInput from './TagInput.svelte'
  import { currentPage, updateCurrentPage } from '../stores/pageStore'
  import { entityTypes, entityTypeMap } from '../stores/entityTypeStore'
  import { pickAndUploadImage, getImageUrl } from '../api/images'

  interface Props {
    open: boolean
    onClose: () => void
  }

  let { open, onClose }: Props = $props()

  async function setImage() {
    const info = await pickAndUploadImage()
    if (info) {
      const url = await getImageUrl(info.id)
      await updateCurrentPage({ featured_image_path: url })
    }
  }

  async function removeImage() {
    await updateCurrentPage({ featured_image_path: '' })
  }
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
      <div class="section">
        <h3 class="section-label">ENTITY TYPE</h3>
        <select
          class="type-select"
          value={$currentPage.entity_type_id || ''}
          onchange={(e) => {
            const val = (e.target as HTMLSelectElement).value
            updateCurrentPage({ entity_type_id: val || '' })
          }}
        >
          <option value="">None (blank page)</option>
          {#each $entityTypes as type (type.id)}
            <option value={type.id}>{type.name}</option>
          {/each}
        </select>
      </div>

      <div class="section-divider"></div>

      <EntityFields />

      <div class="section-divider"></div>

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

      <TagInput />

      <div class="section-divider"></div>

      <RelationsPanel />

      <div class="section-divider"></div>

      <BacklinksPanel />

      <div class="section-divider"></div>

      <div class="section">
        <h3 class="section-label">FEATURED IMAGE</h3>
        {#if $currentPage?.featured_image_path}
          <img class="featured-img" src={$currentPage.featured_image_path} alt="Featured" />
          <button class="remove-img-btn" onclick={removeImage}>Remove image</button>
        {:else}
          <button class="set-img-btn" onclick={setImage}>Set featured image</button>
        {/if}
      </div>
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

  .type-select {
    width: 100%;
    padding: 6px 10px;
    background: var(--color-surface-primary);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-primary);
    cursor: pointer;
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

  .featured-img {
    width: 100%;
    height: 120px;
    object-fit: cover;
    border-radius: var(--radius-md);
  }

  .set-img-btn,
  .remove-img-btn {
    background: none;
    border: 1px dashed var(--color-border-default);
    color: var(--color-fg-tertiary);
    font-family: var(--font-ui);
    font-size: 12px;
    padding: 8px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    width: 100%;
    text-align: center;
  }

  .set-img-btn:hover,
  .remove-img-btn:hover {
    border-color: var(--color-accent-gold);
    color: var(--color-accent-gold);
  }
</style>
