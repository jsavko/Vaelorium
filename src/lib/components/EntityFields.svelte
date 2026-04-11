<script lang="ts">
  import {
    currentPageFields,
    currentPageFieldValues,
    setFieldValue,
    loadPageEntityData,
  } from '../stores/entityTypeStore'
  import { currentPage, currentPageId, pageTree } from '../stores/pageStore'
  import { entityTypeMap } from '../stores/entityTypeStore'
  import type { EntityTypeField, FieldValue } from '../api/entityTypes'

  // Load fields + values when page or type changes
  let lastLoadKey = ''
  $effect(() => {
    const page = $currentPage
    if (!page) return
    const key = `${page.id}:${page.entity_type_id || ''}`
    if (key !== lastLoadKey) {
      lastLoadKey = key
      loadPageEntityData(page.id, page.entity_type_id)
    }
  })

  function getFieldValue(fieldId: string): string | null {
    const fv = $currentPageFieldValues.find((v) => v.field_id === fieldId)
    return fv?.value ?? null
  }

  function parseJson(raw: string | null): any {
    if (raw === null || raw === '') return null
    try {
      return JSON.parse(raw)
    } catch {
      return raw
    }
  }

  function parseOptions(optionsJson: string | null): string[] {
    if (!optionsJson) return []
    try {
      return JSON.parse(optionsJson)
    } catch {
      return []
    }
  }

  // Debounced save for text/number fields
  let debounceTimers = new Map<string, ReturnType<typeof setTimeout>>()

  function saveDebounced(fieldId: string, value: any) {
    const pageId = $currentPageId
    if (!pageId) return
    const existing = debounceTimers.get(fieldId)
    if (existing) clearTimeout(existing)
    debounceTimers.set(
      fieldId,
      setTimeout(() => {
        setFieldValue(pageId, fieldId, JSON.stringify(value))
        debounceTimers.delete(fieldId)
      }, 300),
    )
  }

  function saveImmediate(fieldId: string, value: any) {
    const pageId = $currentPageId
    if (!pageId) return
    setFieldValue(pageId, fieldId, JSON.stringify(value))
  }

  // Get pages filtered by reference_type_id for page_reference fields
  function getReferenceCandidates(referenceTypeId: string | null) {
    if (!referenceTypeId) return $pageTree
    return $pageTree.filter((p) => p.entity_type_id === referenceTypeId)
  }

  function getPageTitle(pageUuid: string | null): string {
    if (!pageUuid) return ''
    const node = $pageTree.find((p) => p.id === pageUuid)
    return node?.title || 'Unknown page'
  }
</script>

{#if $currentPageFields.length > 0}
  <div class="section">
    <h3 class="section-label">
      {$currentPage?.entity_type_id ? ($entityTypeMap.get($currentPage.entity_type_id)?.name || 'ENTITY') : 'ENTITY'} FIELDS
    </h3>
    <div class="fields-list">
      {#each $currentPageFields as field (field.id)}
        {@const rawValue = getFieldValue(field.id)}
        {@const value = parseJson(rawValue)}
        <div class="field-row">
          <label class="field-label" for={field.id}>{field.name}</label>

          {#if field.field_type === 'text'}
            <input
              id={field.id}
              type="text"
              class="field-input"
              value={value || ''}
              oninput={(e) => saveDebounced(field.id, (e.target as HTMLInputElement).value)}
              placeholder="Enter {field.name.toLowerCase()}..."
            />

          {:else if field.field_type === 'number'}
            <input
              id={field.id}
              type="number"
              class="field-input"
              value={value ?? ''}
              oninput={(e) => {
                const num = (e.target as HTMLInputElement).valueAsNumber
                if (!isNaN(num)) saveDebounced(field.id, num)
              }}
              placeholder="0"
            />

          {:else if field.field_type === 'select'}
            {@const options = parseOptions(field.options)}
            <select
              id={field.id}
              class="field-select"
              value={value || ''}
              onchange={(e) => saveImmediate(field.id, (e.target as HTMLSelectElement).value)}
            >
              <option value="">—</option>
              {#each options as opt}
                <option value={opt}>{opt}</option>
              {/each}
            </select>

          {:else if field.field_type === 'multi_select'}
            {@const options = parseOptions(field.options)}
            {@const selected = Array.isArray(value) ? value : []}
            <div class="multi-select">
              {#each selected as tag}
                <span class="tag">
                  {tag}
                  <button
                    class="tag-remove"
                    onclick={() => saveImmediate(field.id, selected.filter((t) => t !== tag))}
                  >&times;</button>
                </span>
              {/each}
              <select
                class="field-select tag-add"
                value=""
                onchange={(e) => {
                  const val = (e.target as HTMLSelectElement).value
                  if (val && !selected.includes(val)) {
                    saveImmediate(field.id, [...selected, val])
                  }
                  ;(e.target as HTMLSelectElement).value = ''
                }}
              >
                <option value="">+ Add...</option>
                {#each options.filter((o) => !selected.includes(o)) as opt}
                  <option value={opt}>{opt}</option>
                {/each}
              </select>
            </div>

          {:else if field.field_type === 'long_text'}
            <textarea
              id={field.id}
              class="field-textarea"
              value={value || ''}
              oninput={(e) => saveDebounced(field.id, (e.target as HTMLTextAreaElement).value)}
              placeholder="Enter text..."
              rows="3"
            ></textarea>

          {:else if field.field_type === 'boolean'}
            <label class="toggle-row">
              <input
                type="checkbox"
                class="toggle-input"
                checked={value === true}
                onchange={(e) => saveImmediate(field.id, (e.target as HTMLInputElement).checked)}
              />
              <span class="toggle-label">{value ? 'Yes' : 'No'}</span>
            </label>

          {:else if field.field_type === 'page_reference'}
            {@const candidates = getReferenceCandidates(field.reference_type_id)}
            <select
              id={field.id}
              class="field-select"
              value={value || ''}
              onchange={(e) => {
                const val = (e.target as HTMLSelectElement).value
                saveImmediate(field.id, val || null)
              }}
            >
              <option value="">— Select page —</option>
              {#each candidates as candidate}
                <option value={candidate.id}>{candidate.title}</option>
              {/each}
            </select>
          {/if}
        </div>
      {/each}
    </div>
  </div>
{/if}

<style>
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

  .fields-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .field-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .field-label {
    font-family: var(--font-ui);
    font-size: 12px;
    font-weight: 500;
    color: var(--color-fg-tertiary);
  }

  .field-input,
  .field-textarea {
    width: 100%;
    padding: 6px 10px;
    background: var(--color-surface-primary);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-primary);
    outline: none;
    box-sizing: border-box;
  }

  .field-input:focus,
  .field-textarea:focus {
    border-color: var(--color-accent-gold);
  }

  .field-input::placeholder,
  .field-textarea::placeholder {
    color: var(--color-fg-tertiary);
  }

  .field-textarea {
    resize: vertical;
    min-height: 60px;
  }

  .field-select {
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

  .multi-select {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    align-items: center;
  }

  .tag {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    background: var(--color-surface-tertiary);
    border-radius: 12px;
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-fg-primary);
  }

  .tag-remove {
    background: none;
    border: none;
    color: var(--color-fg-tertiary);
    cursor: pointer;
    font-size: 14px;
    padding: 0;
    line-height: 1;
  }

  .tag-remove:hover {
    color: var(--color-status-error);
  }

  .tag-add {
    width: auto;
    flex: 1;
    min-width: 80px;
  }

  .toggle-row {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
  }

  .toggle-input {
    width: 16px;
    height: 16px;
    accent-color: var(--color-accent-gold);
    cursor: pointer;
  }

  .toggle-label {
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-primary);
  }
</style>
