<script lang="ts">
  import { createEntityType } from '../stores/entityTypeStore'
  import { createEntityTypeField } from '../api/entityTypes'

  interface Props {
    open: boolean
    onClose: () => void
    onCreated: () => void
  }

  let { open, onClose, onCreated }: Props = $props()

  let typeName = $state('')
  let typeColor = $state('#B85C5C')
  let typeIcon = $state('file-text')
  let saving = $state(false)

  interface FieldDef {
    name: string
    field_type: string
    options: string
  }

  let fields = $state<FieldDef[]>([])

  const colorPresets = [
    '#B85C5C', '#4A8C6A', '#5C7AB8', '#8B5CB8',
    '#B8955C', '#5CB8A8', '#B85C8B', '#7A8C5C',
    '#C87040', '#5C6CB8',
  ]

  const iconPresets = [
    'file-text', 'star', 'heart', 'flag', 'zap', 'feather',
    'book', 'map', 'sword', 'crown', 'flame', 'moon',
  ]

  const fieldTypes = [
    { value: 'text', label: 'Text' },
    { value: 'number', label: 'Number' },
    { value: 'select', label: 'Select' },
    { value: 'multi_select', label: 'Multi-select' },
    { value: 'long_text', label: 'Long text' },
    { value: 'boolean', label: 'Boolean' },
    { value: 'page_reference', label: 'Page reference' },
  ]

  function addField() {
    fields = [...fields, { name: '', field_type: 'text', options: '' }]
  }

  function removeField(index: number) {
    fields = fields.filter((_, i) => i !== index)
  }

  async function handleSave() {
    if (!typeName.trim()) return
    saving = true
    try {
      const type = await createEntityType(typeName.trim(), typeIcon, typeColor)

      // Create fields
      for (const field of fields) {
        if (!field.name.trim()) continue
        const options = field.options.trim()
          ? JSON.stringify(field.options.split(',').map((o) => o.trim()).filter(Boolean))
          : null
        await createEntityTypeField(
          type.id,
          field.name.trim(),
          field.field_type,
          options,
        )
      }

      resetAndClose()
      onCreated()
    } finally {
      saving = false
    }
  }

  function resetAndClose() {
    typeName = ''
    typeColor = '#B85C5C'
    typeIcon = 'file-text'
    fields = []
    onClose()
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') resetAndClose()
  }
</script>

<svelte:window onkeydown={(e) => { if (open) handleKeydown(e) }} />

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="overlay" onclick={resetAndClose} role="dialog" aria-modal="true" tabindex="-1">
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="builder" onclick={(e) => e.stopPropagation()}>
      <h2 class="builder-title">Create Custom Type</h2>

      <div class="form-row">
        <label class="form-label" for="type-name-input">Name</label>
        <input
          id="type-name-input"
          class="form-input"
          bind:value={typeName}
          placeholder="e.g. Spell, Vehicle, Language..."
        />
      </div>

      <div class="form-row" role="group" aria-labelledby="color-grid-label">
        <!-- svelte-ignore a11y_label_has_associated_control -- labels a button-grid, not a single input -->
        <label class="form-label" id="color-grid-label">Color</label>
        <div class="color-grid">
          {#each colorPresets as color}
            <button
              class="color-swatch"
              class:selected={typeColor === color}
              style:background-color={color}
              onclick={() => typeColor = color}
              aria-label={color}
            ></button>
          {/each}
        </div>
      </div>

      <div class="form-row" role="group" aria-labelledby="icon-grid-label">
        <!-- svelte-ignore a11y_label_has_associated_control -- labels a button-grid, not a single input -->
        <label class="form-label" id="icon-grid-label">Icon</label>
        <div class="icon-grid">
          {#each iconPresets as icon}
            <button
              class="icon-option"
              class:selected={typeIcon === icon}
              onclick={() => typeIcon = icon}
            >
              {icon.replace(/-/g, ' ')}
            </button>
          {/each}
        </div>
      </div>

      <div class="form-row" role="group" aria-labelledby="fields-list-label">
        <div class="fields-header">
          <!-- svelte-ignore a11y_label_has_associated_control -- labels a list, not a single input -->
          <label class="form-label" id="fields-list-label">Fields</label>
          <button class="add-field-btn" onclick={addField}>+ Add field</button>
        </div>
        {#if fields.length === 0}
          <p class="no-fields">No fields yet. Add fields to define the structure.</p>
        {:else}
          <div class="field-list">
            {#each fields as field, i}
              <div class="field-def">
                <input
                  class="field-name-input"
                  bind:value={field.name}
                  placeholder="Field name"
                />
                <select class="field-type-select" bind:value={field.field_type}>
                  {#each fieldTypes as ft}
                    <option value={ft.value}>{ft.label}</option>
                  {/each}
                </select>
                {#if field.field_type === 'select' || field.field_type === 'multi_select'}
                  <input
                    class="field-options-input"
                    bind:value={field.options}
                    placeholder="Options (comma-separated)"
                  />
                {/if}
                <button class="field-remove" onclick={() => removeField(i)}>&times;</button>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <div class="builder-footer">
        <button class="cancel-btn" onclick={resetAndClose}>Cancel</button>
        <button
          class="save-btn"
          disabled={!typeName.trim() || saving}
          onclick={handleSave}
        >
          {saving ? 'Creating...' : 'Create Type'}
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
    z-index: 310;
  }

  .builder {
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-lg);
    padding: 24px;
    width: 520px;
    max-height: 80vh;
    overflow-y: auto;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.4);
  }

  .builder-title {
    font-family: var(--font-heading);
    font-size: 20px;
    font-weight: 600;
    color: var(--color-fg-primary);
    margin: 0 0 20px;
  }

  .form-row {
    margin-bottom: 16px;
  }

  .form-label {
    display: block;
    font-family: var(--font-ui);
    font-size: 12px;
    font-weight: 600;
    color: var(--color-fg-tertiary);
    margin-bottom: 6px;
  }

  .form-input {
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

  .form-input:focus {
    border-color: var(--color-accent-gold);
  }

  .color-grid {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .color-swatch {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
  }

  .color-swatch.selected {
    border-color: var(--color-accent-gold);
    box-shadow: 0 0 0 2px var(--color-surface-card);
  }

  .icon-grid {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }

  .icon-option {
    padding: 4px 8px;
    background: var(--color-surface-tertiary);
    border: 2px solid transparent;
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 11px;
    color: var(--color-fg-secondary);
    cursor: pointer;
    text-transform: capitalize;
  }

  .icon-option.selected {
    border-color: var(--color-accent-gold);
    color: var(--color-accent-gold);
  }

  .fields-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 6px;
  }

  .fields-header .form-label {
    margin: 0;
  }

  .add-field-btn {
    background: none;
    border: none;
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-accent-gold);
    cursor: pointer;
  }

  .no-fields {
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-tertiary);
    margin: 0;
  }

  .field-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .field-def {
    display: flex;
    gap: 6px;
    align-items: start;
    flex-wrap: wrap;
  }

  .field-name-input {
    flex: 1;
    min-width: 120px;
    padding: 6px 10px;
    background: var(--color-surface-primary);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-primary);
    outline: none;
  }

  .field-type-select {
    padding: 6px 10px;
    background: var(--color-surface-primary);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-primary);
  }

  .field-options-input {
    width: 100%;
    padding: 6px 10px;
    background: var(--color-surface-primary);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-fg-primary);
    outline: none;
  }

  .field-remove {
    background: none;
    border: none;
    color: var(--color-fg-tertiary);
    font-size: 18px;
    cursor: pointer;
    padding: 4px;
    line-height: 1;
  }

  .field-remove:hover {
    color: var(--color-status-error);
  }

  .builder-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 20px;
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

  .save-btn {
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

  .save-btn:hover:not(:disabled) {
    background: var(--color-accent-gold-hover);
  }

  .save-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
