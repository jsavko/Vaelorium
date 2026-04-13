<script lang="ts">
  import { Shield, Compass, Scroll, Users, Gem, Bug, Sparkles, BookOpen, FileText, Plus } from 'lucide-svelte'
  import CustomTypeBuilder from './CustomTypeBuilder.svelte'
  import { entityTypes, loadEntityTypes } from '../stores/entityTypeStore'
  import { pageTree } from '../stores/pageStore'
  import type { EntityType } from '../api/entityTypes'

  interface Props {
    open: boolean
    onClose: () => void
    onCreate: (title: string, parentId: string | null, entityTypeId: string | null) => void
    initialTypeId?: string | null
  }

  let { open, onClose, onCreate, initialTypeId = null }: Props = $props()

  let title = $state('')
  let selectedTypeId = $state<string | null>(null)
  let parentId = $state<string | null>(null)
  let titleInput = $state<HTMLInputElement | null>(null)
  let builderOpen = $state(false)

  const iconMap: Record<string, any> = {
    shield: Shield,
    compass: Compass,
    scroll: Scroll,
    users: Users,
    gem: Gem,
    bug: Bug,
    sparkles: Sparkles,
    'notebook-pen': BookOpen,
  }

  function getIcon(iconName: string | null) {
    if (!iconName) return FileText
    return iconMap[iconName] || FileText
  }

  function selectType(typeId: string) {
    if (selectedTypeId === typeId) {
      selectedTypeId = null
    } else {
      selectedTypeId = typeId
    }
    // Focus title input after type selection
    setTimeout(() => titleInput?.focus(), 50)
  }

  function handleCreate() {
    if (!title.trim()) return
    onCreate(title.trim(), parentId, selectedTypeId)
    resetAndClose()
  }

  function resetAndClose() {
    title = ''
    selectedTypeId = null
    parentId = null
    onClose()
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') resetAndClose()
    if (e.key === 'Enter' && title.trim()) handleCreate()
  }

  function handleWindowKeydown(e: KeyboardEvent) {
    if (!open) return
    if (e.key === 'Escape') {
      e.preventDefault()
      resetAndClose()
    }
  }

  // Set initial type and focus title input when modal opens
  $effect(() => {
    if (open) {
      selectedTypeId = initialTypeId || null
      setTimeout(() => titleInput?.focus(), 100)
    }
  })
</script>

<svelte:window onkeydown={handleWindowKeydown} />

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="overlay" onclick={resetAndClose} onkeydown={handleKeydown} role="dialog" aria-modal="true" tabindex="-1">
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="modal" onclick={(e) => e.stopPropagation()}>
      <h2 class="modal-title">New Page</h2>

      <!-- Blank Page option -->
      <button
        class="blank-option"
        class:selected={selectedTypeId === null}
        onclick={() => { selectedTypeId = null; titleInput?.focus() }}
      >
        <span class="blank-icon">
          <FileText size={20} />
        </span>
        <span class="blank-label">Blank Page</span>
        <span class="blank-desc">No entity type — free-form wiki page</span>
      </button>

      <!-- Entity Types grid -->
      <div class="section-label">ENTITY TYPES</div>
      <div class="type-grid">
        {#each $entityTypes as type (type.id)}
          {@const TypeIcon = getIcon(type.icon)}
          <button
            class="type-card"
            class:selected={selectedTypeId === type.id}
            style:--type-color={type.color || 'var(--color-fg-tertiary)'}
            onclick={() => selectType(type.id)}
          >
            <span class="type-icon">
              <TypeIcon size={18} />
            </span>
            <span class="type-name">{type.name}</span>
          </button>
        {/each}
        <button
          class="type-card custom-type-card"
          onclick={() => builderOpen = true}
        >
          <span class="type-icon" style:color="var(--color-fg-tertiary)">
            <Plus size={18} />
          </span>
          <span class="type-name">Custom Type</span>
        </button>
      </div>

      <!-- Footer: title + parent + create -->
      <div class="modal-footer">
        <input
          bind:this={titleInput}
          bind:value={title}
          class="title-input"
          placeholder="Page title..."
          onkeydown={handleKeydown}
        />
        <select class="parent-select" bind:value={parentId}>
          <option value={null}>No parent</option>
          {#each $pageTree as node (node.id)}
            <option value={node.id}>{node.title}</option>
          {/each}
        </select>
        <button
          class="create-btn"
          disabled={!title.trim()}
          onclick={handleCreate}
        >
          Create
        </button>
      </div>
    </div>
  </div>
{/if}

<CustomTypeBuilder
  open={builderOpen}
  onClose={() => builderOpen = false}
  onCreated={() => loadEntityTypes()}
/>

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
    padding: 24px;
    width: 520px;
    max-height: 80vh;
    overflow-y: auto;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.4);
  }

  .modal-title {
    font-family: var(--font-heading);
    font-size: 20px;
    font-weight: 600;
    color: var(--color-fg-primary);
    margin: 0 0 16px;
  }

  .blank-option {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 12px 16px;
    background: var(--color-surface-tertiary);
    border: 2px solid transparent;
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    margin-bottom: 16px;
  }

  .blank-option:hover {
    background: var(--color-surface-secondary);
  }

  .blank-option.selected {
    border-color: var(--color-accent-gold);
  }

  .blank-icon {
    color: var(--color-fg-tertiary);
    flex-shrink: 0;
  }

  .blank-label {
    font-family: var(--font-ui);
    font-size: 14px;
    font-weight: 600;
    color: var(--color-fg-primary);
  }

  .blank-desc {
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-fg-tertiary);
    margin-left: auto;
  }

  .section-label {
    font-family: var(--font-ui);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.08em;
    color: var(--color-fg-tertiary);
    margin-bottom: 8px;
  }

  .type-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
    margin-bottom: 20px;
  }

  .type-card {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    background: var(--color-surface-tertiary);
    border: 2px solid transparent;
    border-left: 3px solid var(--type-color);
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
  }

  .type-card:hover {
    background: var(--color-surface-secondary);
  }

  .type-card.selected {
    border-color: var(--color-accent-gold);
    border-left-color: var(--type-color);
  }

  .type-icon {
    color: var(--type-color);
    flex-shrink: 0;
    display: flex;
  }

  .type-name {
    font-family: var(--font-ui);
    font-size: 13px;
    font-weight: 500;
    color: var(--color-fg-primary);
  }

  .custom-type-card {
    border-style: dashed;
    border-left-style: dashed;
  }

  .modal-footer {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .title-input {
    flex: 1;
    padding: 8px 12px;
    background: var(--color-surface-primary);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 14px;
    color: var(--color-fg-primary);
    outline: none;
  }

  .title-input:focus {
    border-color: var(--color-accent-gold);
  }

  .title-input::placeholder {
    color: var(--color-fg-tertiary);
  }

  .parent-select {
    padding: 8px 12px;
    background: var(--color-surface-primary);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-primary);
    max-width: 140px;
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
    white-space: nowrap;
  }

  .create-btn:hover:not(:disabled) {
    background: var(--color-accent-gold-hover);
  }

  .create-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
