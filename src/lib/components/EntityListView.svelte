<script lang="ts">
  import { Shield, Compass, Scroll, Users, Gem, Bug, Sparkles, BookOpen, FileText } from 'lucide-svelte'
  import { loadPage, pageTree } from '../stores/pageStore'
  import { entityTypeMap } from '../stores/entityTypeStore'
  import { listPagesByType } from '../api/pages'
  import { getPageFieldValues } from '../api/entityTypes'
  import { listEntityTypeFields } from '../api/entityTypes'
  import type { Page } from '../api/pages'
  import type { EntityType, EntityTypeField, FieldValue } from '../api/entityTypes'

  interface Props {
    entityTypeId: string
    onOpenNewPage: () => void
    onClose: () => void
  }

  let { entityTypeId, onOpenNewPage, onClose }: Props = $props()

  let pages = $state<Page[]>([])
  let fields = $state<EntityTypeField[]>([])
  let fieldValuesMap = $state<Map<string, FieldValue[]>>(new Map())
  let searchQuery = $state('')
  let loading = $state(true)

  const iconMap: Record<string, any> = {
    shield: Shield, compass: Compass, scroll: Scroll, users: Users,
    gem: Gem, bug: Bug, sparkles: Sparkles, 'notebook-pen': BookOpen,
  }

  let entityType = $derived($entityTypeMap.get(entityTypeId))

  function getIcon(iconName: string | null) {
    if (!iconName) return FileText
    return iconMap[iconName] || FileText
  }

  let filteredPages = $derived(
    searchQuery
      ? pages.filter((p) => p.title.toLowerCase().includes(searchQuery.toLowerCase()))
      : pages,
  )

  // Load pages and fields when type changes, and refresh whenever the
  // global page tree mutates (create / delete / rename). Touching
  // $pageTree here makes it a reactive dependency — creating a page
  // of this type now shows up on the list without navigating away.
  $effect(() => {
    void $pageTree
    loadData(entityTypeId)
  })

  async function loadData(typeId: string) {
    loading = true
    const [pageList, fieldList] = await Promise.all([
      listPagesByType(typeId),
      listEntityTypeFields(typeId),
    ])
    pages = pageList
    fields = fieldList.slice(0, 3) // Show first 3 fields on cards

    // Load field values for each page (batch)
    const valMap = new Map<string, FieldValue[]>()
    await Promise.all(
      pageList.map(async (p) => {
        const vals = await getPageFieldValues(p.id)
        valMap.set(p.id, vals)
      }),
    )
    fieldValuesMap = valMap
    loading = false
  }

  function getFieldDisplay(pageId: string, fieldId: string): string {
    const vals = fieldValuesMap.get(pageId)
    if (!vals) return ''
    const fv = vals.find((v) => v.field_id === fieldId)
    if (!fv?.value) return ''
    try {
      const parsed = JSON.parse(fv.value)
      if (Array.isArray(parsed)) return parsed.join(', ')
      return String(parsed)
    } catch {
      return fv.value
    }
  }

  function openPage(pageId: string) {
    loadPage(pageId)
    onClose()
  }
</script>

<div class="entity-list-view">
  <header class="list-header">
    <div class="header-left">
      <button class="back-btn" onclick={onClose} aria-label="Back to editor">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="15 18 9 12 15 6"></polyline>
        </svg>
      </button>
      {#if entityType}
        {@const HeaderIcon = getIcon(entityType.icon)}
        <span class="header-icon" style:color={entityType.color || 'var(--color-fg-tertiary)'}>
          <HeaderIcon size={22} />
        </span>
        <h2 class="header-title">{entityType.name}s</h2>
        <span class="header-count">{pages.length}</span>
      {/if}
    </div>
    <button class="new-btn" onclick={onOpenNewPage}>
      + New {entityType?.name || 'Page'}
    </button>
  </header>

  <div class="search-bar">
    <input
      class="search-input"
      placeholder="Search {entityType?.name?.toLowerCase() || 'page'}s..."
      bind:value={searchQuery}
    />
  </div>

  {#if loading}
    <div class="loading">Loading...</div>
  {:else if filteredPages.length === 0}
    <div class="empty-state">
      <p class="empty-text">No {entityType?.name?.toLowerCase() || 'page'}s yet</p>
      <button class="empty-create" onclick={onOpenNewPage}>
        Create your first {entityType?.name?.toLowerCase() || 'page'}
      </button>
    </div>
  {:else}
    <div class="card-grid">
      {#each filteredPages as page (page.id)}
        <button class="card" onclick={() => openPage(page.id)}>
          <div class="card-image" style:border-color={entityType?.color || 'var(--color-border-default)'}>
            {#if page.featured_image_path}
              <img src={page.featured_image_path} alt="" class="card-img" />
            {:else}
              {@const CardIcon = getIcon(entityType?.icon || null)}
              <span class="card-placeholder" style:color={entityType?.color || 'var(--color-fg-tertiary)'}>
                <CardIcon size={32} />
              </span>
            {/if}
          </div>
          <div class="card-body">
            <h3 class="card-title">{page.title}</h3>
            {#if fields.length > 0}
              <div class="card-meta">
                {#each fields as field}
                  {@const val = getFieldDisplay(page.id, field.id)}
                  {#if val}
                    <span class="meta-item">{field.name}: {val}</span>
                  {/if}
                {/each}
              </div>
            {/if}
          </div>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .entity-list-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .list-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 24px;
    background: var(--color-surface-secondary);
    flex-shrink: 0;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .back-btn {
    background: none;
    border: none;
    color: var(--color-fg-tertiary);
    cursor: pointer;
    padding: 4px;
    border-radius: var(--radius-sm);
  }

  .back-btn:hover {
    background: var(--color-surface-tertiary);
    color: var(--color-fg-primary);
  }

  .header-icon {
    display: flex;
  }

  .header-title {
    font-family: var(--font-heading);
    font-size: 20px;
    font-weight: 600;
    color: var(--color-fg-primary);
    margin: 0;
  }

  .header-count {
    font-family: var(--font-ui);
    font-size: 12px;
    font-weight: 600;
    color: var(--color-fg-tertiary);
    background: var(--color-surface-tertiary);
    padding: 2px 8px;
    border-radius: 10px;
  }

  .new-btn {
    padding: 6px 16px;
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

  .new-btn:hover {
    background: var(--color-accent-gold-hover);
  }

  .search-bar {
    padding: 12px 24px;
    flex-shrink: 0;
  }

  .search-input {
    width: 100%;
    padding: 8px 14px;
    background: var(--color-surface-primary);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-md);
    font-family: var(--font-ui);
    font-size: 14px;
    color: var(--color-fg-primary);
    outline: none;
    box-sizing: border-box;
  }

  .search-input:focus {
    border-color: var(--color-accent-gold);
  }

  .search-input::placeholder {
    color: var(--color-fg-tertiary);
  }

  .loading {
    padding: 48px;
    text-align: center;
    font-family: var(--font-ui);
    font-size: 14px;
    color: var(--color-fg-tertiary);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: 16px;
    padding: 48px;
  }

  .empty-text {
    font-family: var(--font-ui);
    font-size: 16px;
    color: var(--color-fg-tertiary);
  }

  .empty-create {
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

  .card-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 16px;
    padding: 0 24px 24px;
    overflow-y: auto;
    flex: 1;
  }

  .card {
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-md);
    overflow: hidden;
    cursor: pointer;
    text-align: left;
    display: flex;
    flex-direction: column;
  }

  .card:hover {
    border-color: var(--color-accent-gold);
  }

  .card-image {
    height: 80px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-surface-tertiary);
    border-bottom: 2px solid var(--color-border-default);
  }

  .card-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .card-placeholder {
    opacity: 0.3;
  }

  .card-body {
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .card-title {
    font-family: var(--font-heading);
    font-size: 15px;
    font-weight: 600;
    color: var(--color-fg-primary);
    margin: 0;
  }

  .card-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 8px;
  }

  .meta-item {
    font-family: var(--font-ui);
    font-size: 11px;
    color: var(--color-fg-tertiary);
  }
</style>
