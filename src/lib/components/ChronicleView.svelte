<script lang="ts">
  import { Clock, Plus, Calendar, Trash2, MoreHorizontal } from 'lucide-svelte'
  import InputModal from './InputModal.svelte'
  import ConfirmDialog from './ConfirmDialog.svelte'
  import ContextMenu from './ContextMenu.svelte'
  import { timelines, currentTimeline, currentEvents, loadTimelines, loadTimeline, createTimeline, addEvent, removeEvent, renameTimeline, deleteTimeline } from '../stores/timelineStore'
  import { loadPage, pageTree } from '../stores/pageStore'
  import { entityTypeMap } from '../stores/entityTypeStore'
  import { onMount } from 'svelte'

  interface Props {
    onClose: () => void
  }

  let { onClose }: Props = $props()

  let addingEvent = $state(false)
  let timelineNameModalOpen = $state(false)
  let renameTarget = $state<{ id: string; name: string } | null>(null)
  let deleteTarget = $state<{ id: string; name: string } | null>(null)
  let ctxMenu = $state<{ x: number; y: number } | null>(null)
  let newTitle = $state('')
  let newDate = $state('')
  let newDesc = $state('')
  let newPageId = $state('')
  let searchQuery = $state('')

  onMount(async () => {
    await loadTimelines()
    if ($timelines.length > 0) {
      await loadTimeline($timelines[0].id)
    }
  })

  async function handleCreateTimeline(name: string) {
    timelineNameModalOpen = false
    const tl = await createTimeline(name)
    await loadTimeline(tl.id)
  }

  async function handleAddEvent() {
    if (!newTitle.trim() || !newDate.trim() || !$currentTimeline) return
    await addEvent($currentTimeline.id, newTitle.trim(), newDate.trim(), newDesc.trim() || null, newPageId || null)
    newTitle = ''
    newDate = ''
    newDesc = ''
    newPageId = ''
    searchQuery = ''
    addingEvent = false
  }

  function openTimelineMenu(e: MouseEvent) {
    e.stopPropagation()
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
    ctxMenu = { x: rect.left, y: rect.bottom + 4 }
  }

  async function handleRenameTimeline(name: string) {
    if (!renameTarget) return
    const id = renameTarget.id
    renameTarget = null
    await renameTimeline(id, name)
  }

  async function handleDeleteTimeline() {
    if (!deleteTarget) return
    const id = deleteTarget.id
    deleteTarget = null
    await deleteTimeline(id)
    const next = $timelines[0]
    if (next) await loadTimeline(next.id)
  }

  async function handleDeleteEvent(eventId: string) {
    if (!$currentTimeline) return
    await removeEvent(eventId, $currentTimeline.id)
  }

  let filteredPages = $derived(
    searchQuery.length > 0
      ? $pageTree.filter((p) => p.title.toLowerCase().includes(searchQuery.toLowerCase())).slice(0, 6)
      : [],
  )

  function selectPage(pageId: string, title: string) {
    newPageId = pageId
    searchQuery = title
  }

  // Group events by date for display
  let groupedEvents = $derived.by(() => {
    const groups: { date: string; events: typeof $currentEvents }[] = []
    let currentDate = ''
    let currentGroup: typeof $currentEvents = []
    for (const evt of $currentEvents) {
      if (evt.date !== currentDate) {
        if (currentGroup.length > 0) {
          groups.push({ date: currentDate, events: currentGroup })
        }
        currentDate = evt.date
        currentGroup = [evt]
      } else {
        currentGroup.push(evt)
      }
    }
    if (currentGroup.length > 0) {
      groups.push({ date: currentDate, events: currentGroup })
    }
    return groups
  })

  function getEntityColor(typeId: string | null): string {
    if (!typeId) return 'var(--color-fg-tertiary)'
    return $entityTypeMap.get(typeId)?.color || 'var(--color-fg-tertiary)'
  }
</script>

<div class="chronicle-view">
  <header class="chronicle-header">
    <div class="header-left">
      <button class="back-btn" onclick={onClose} aria-label="Back">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="15 18 9 12 15 6"></polyline>
        </svg>
      </button>
      <Clock size={20} />
      <h2 class="header-title">Chronicle</h2>

      {#if $timelines.length > 1}
        <select class="timeline-select" onchange={(e) => loadTimeline((e.target as HTMLSelectElement).value)}>
          {#each $timelines as tl (tl.id)}
            <option value={tl.id} selected={tl.id === $currentTimeline?.id}>{tl.name}</option>
          {/each}
        </select>
      {:else if $currentTimeline}
        <span class="timeline-name">{$currentTimeline.name}</span>
      {/if}
      {#if $currentTimeline}
        <button class="tl-menu-btn" onclick={openTimelineMenu} aria-label="Timeline actions">
          <MoreHorizontal size={16} />
        </button>
      {/if}
    </div>
    <div class="header-right">
      <button class="tool-btn" onclick={() => timelineNameModalOpen = true}>+ New Timeline</button>
      {#if $currentTimeline}
        <button class="tool-btn accent" onclick={() => addingEvent = !addingEvent}>
          {addingEvent ? 'Cancel' : '+ Add Event'}
        </button>
      {/if}
    </div>
  </header>

  <div class="chronicle-content">
    {#if !$currentTimeline}
      <div class="empty-state">
        <Calendar size={48} />
        <p>No timelines yet</p>
        <button class="empty-create" onclick={() => timelineNameModalOpen = true}>Create your first timeline</button>
      </div>
    {:else if $currentEvents.length === 0 && !addingEvent}
      <div class="empty-state">
        <p>No events in this timeline</p>
        <button class="empty-create" onclick={() => addingEvent = true}>Add your first event</button>
      </div>
    {:else}
      {#if addingEvent}
        <div class="event-form">
          <div class="form-row">
            <input class="form-input date-input" bind:value={newDate} placeholder="Date (e.g. 1420-03-15)" />
            <input class="form-input" bind:value={newTitle} placeholder="Event title..." style="flex:1" />
          </div>
          <textarea class="form-textarea" bind:value={newDesc} placeholder="Description (optional)..." rows="2"></textarea>
          <div class="form-row">
            <div class="search-wrapper" style="flex:1">
              <input class="form-input" bind:value={searchQuery} placeholder="Link to page (optional)..." />
              {#if filteredPages.length > 0}
                <div class="search-results">
                  {#each filteredPages as page (page.id)}
                    <button class="search-result" onclick={() => selectPage(page.id, page.title)}>{page.title}</button>
                  {/each}
                </div>
              {/if}
            </div>
            <button class="save-btn" onclick={handleAddEvent} disabled={!newTitle.trim() || !newDate.trim()}>Add</button>
          </div>
        </div>
      {/if}

      <div class="timeline-line">
        {#each groupedEvents as group}
          <div class="date-group">
            <div class="date-marker">
              <span class="date-dot"></span>
              <span class="date-label">{group.date}</span>
            </div>
            {#each group.events as evt (evt.id)}
              <div class="event-card">
                <div class="event-header">
                  <h3 class="event-title">{evt.title}</h3>
                  <button class="event-delete" onclick={() => handleDeleteEvent(evt.id)} title="Delete">
                    <Trash2 size={12} />
                  </button>
                </div>
                {#if evt.description}
                  <p class="event-desc">{evt.description}</p>
                {/if}
                {#if evt.page_id}
                  {@const page = $pageTree.find((p) => p.id === evt.page_id)}
                  {#if page}
                    <button class="event-link" onclick={() => { loadPage(page.id); window.dispatchEvent(new CustomEvent('vaelorium:page-selected')) }}>
                      <span class="link-dot" style:background-color={getEntityColor(page.entity_type_id)}></span>
                      {page.title} →
                    </button>
                  {/if}
                {/if}
              </div>
            {/each}
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<InputModal
  open={timelineNameModalOpen}
  title="New Timeline"
  placeholder="Timeline name..."
  confirmLabel="Create"
  onConfirm={handleCreateTimeline}
  onCancel={() => timelineNameModalOpen = false}
/>

<InputModal
  open={renameTarget !== null}
  title="Rename Timeline"
  placeholder="Timeline name..."
  confirmLabel="Save"
  initialValue={renameTarget?.name ?? ''}
  onConfirm={handleRenameTimeline}
  onCancel={() => renameTarget = null}
/>

<ConfirmDialog
  open={deleteTarget !== null}
  title="Delete Timeline"
  message={`Delete "${deleteTarget?.name ?? ''}" and all its events? This cannot be undone.`}
  confirmLabel="Delete"
  danger
  onConfirm={handleDeleteTimeline}
  onCancel={() => deleteTarget = null}
/>

{#if ctxMenu && $currentTimeline}
  {@const tl = $currentTimeline}
  <ContextMenu
    x={ctxMenu.x}
    y={ctxMenu.y}
    items={[
      { label: 'Rename…', action: () => { renameTarget = { id: tl.id, name: tl.name } } },
      { label: 'Delete', danger: true, action: () => { deleteTarget = { id: tl.id, name: tl.name } } },
    ]}
    onClose={() => ctxMenu = null}
  />
{/if}

<style>
  .chronicle-view { flex: 1; display: flex; flex-direction: column; height: 100%; overflow: hidden; }

  .chronicle-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 10px 20px; background: var(--color-surface-secondary);
    border-bottom: 1px solid var(--color-border-subtle); flex-shrink: 0;
  }
  .header-left { display: flex; align-items: center; gap: 10px; color: var(--color-fg-tertiary); }
  .tl-menu-btn { background: none; border: none; color: var(--color-fg-tertiary); cursor: pointer; padding: 4px; border-radius: var(--radius-sm); display: inline-flex; align-items: center; }
  .tl-menu-btn:hover { background: var(--color-surface-tertiary); color: var(--color-fg-primary); }
  .header-right { display: flex; gap: 8px; }
  .back-btn { background: none; border: none; color: var(--color-fg-tertiary); cursor: pointer; padding: 4px; border-radius: var(--radius-sm); }
  .back-btn:hover { background: var(--color-surface-tertiary); color: var(--color-fg-primary); }
  .header-title { font-family: var(--font-heading); font-size: 18px; font-weight: 600; color: var(--color-fg-primary); margin: 0; }
  .timeline-name { font-family: var(--font-ui); font-size: 13px; color: var(--color-fg-tertiary); }
  .timeline-select { padding: 4px 10px; background: var(--color-surface-primary); border: 1px solid var(--color-border-default); border-radius: var(--radius-sm); font-family: var(--font-ui); font-size: 12px; color: var(--color-fg-primary); }
  .tool-btn { padding: 4px 14px; background: var(--color-surface-tertiary); border: 1px solid var(--color-border-default); border-radius: var(--radius-sm); font-family: var(--font-ui); font-size: 12px; color: var(--color-fg-secondary); cursor: pointer; }
  .tool-btn:hover { border-color: var(--color-accent-gold); }
  .tool-btn.accent { background: var(--color-accent-gold); color: var(--color-fg-inverse); border-color: var(--color-accent-gold); font-weight: 600; }

  .chronicle-content { flex: 1; overflow-y: auto; padding: 24px 40px; }

  .empty-state { display: flex; flex-direction: column; align-items: center; justify-content: center; flex: 1; gap: 12px; color: var(--color-fg-tertiary); padding: 48px; }
  .empty-create { padding: 8px 20px; background: var(--color-accent-gold); border: none; border-radius: var(--radius-sm); font-family: var(--font-ui); font-size: 13px; font-weight: 600; color: var(--color-fg-inverse); cursor: pointer; }

  .event-form { background: var(--color-surface-card); border: 1px solid var(--color-border-default); border-radius: var(--radius-md); padding: 16px; margin-bottom: 24px; display: flex; flex-direction: column; gap: 8px; }
  .form-row { display: flex; gap: 8px; align-items: start; }
  .form-input { padding: 6px 10px; background: var(--color-surface-primary); border: 1px solid var(--color-border-default); border-radius: var(--radius-sm); font-family: var(--font-ui); font-size: 13px; color: var(--color-fg-primary); outline: none; box-sizing: border-box; }
  .form-input:focus { border-color: var(--color-accent-gold); }
  .date-input { width: 160px; }
  .form-textarea { width: 100%; padding: 6px 10px; background: var(--color-surface-primary); border: 1px solid var(--color-border-default); border-radius: var(--radius-sm); font-family: var(--font-ui); font-size: 13px; color: var(--color-fg-primary); outline: none; resize: vertical; box-sizing: border-box; }
  .search-wrapper { position: relative; }
  .search-results { position: absolute; top: 100%; left: 0; right: 0; background: var(--color-surface-card); border: 1px solid var(--color-border-default); border-radius: var(--radius-sm); z-index: 10; max-height: 150px; overflow-y: auto; }
  .search-result { display: block; width: 100%; padding: 6px 10px; background: none; border: none; text-align: left; font-family: var(--font-ui); font-size: 12px; color: var(--color-fg-primary); cursor: pointer; }
  .search-result:hover { background: var(--color-surface-tertiary); }
  .save-btn { padding: 6px 16px; background: var(--color-accent-gold); border: none; border-radius: var(--radius-sm); font-family: var(--font-ui); font-size: 13px; font-weight: 600; color: var(--color-fg-inverse); cursor: pointer; }
  .save-btn:disabled { opacity: 0.4; cursor: not-allowed; }

  .timeline-line { position: relative; padding-left: 24px; border-left: 2px solid var(--color-border-default); margin-left: 12px; }

  .date-group { margin-bottom: 24px; }
  .date-marker { display: flex; align-items: center; gap: 8px; margin-bottom: 8px; margin-left: -33px; }
  .date-dot { width: 10px; height: 10px; border-radius: 50%; background: var(--color-accent-gold); border: 2px solid var(--color-surface-primary); flex-shrink: 0; }
  .date-label { font-family: var(--font-ui); font-size: 12px; font-weight: 700; color: var(--color-accent-gold); letter-spacing: 0.5px; }

  .event-card { background: var(--color-surface-card); border: 1px solid var(--color-border-default); border-radius: var(--radius-md); padding: 12px 16px; margin-bottom: 8px; }
  .event-card:hover .event-delete { opacity: 1; }
  .event-header { display: flex; align-items: center; justify-content: space-between; }
  .event-title { font-family: var(--font-heading); font-size: 15px; font-weight: 600; color: var(--color-fg-primary); margin: 0; }
  .event-delete { background: none; border: none; color: var(--color-fg-tertiary); cursor: pointer; opacity: 0; transition: opacity 0.1s; }
  .event-delete:hover { color: var(--color-status-error); }
  .event-desc { font-family: var(--font-body); font-size: 13px; color: var(--color-fg-secondary); margin: 6px 0 0; }
  .event-link { display: flex; align-items: center; gap: 6px; background: none; border: none; font-family: var(--font-ui); font-size: 12px; color: var(--color-accent-gold); cursor: pointer; padding: 0; margin-top: 8px; }
  .event-link:hover { text-decoration: underline; }
  .link-dot { width: 6px; height: 6px; border-radius: 50%; flex-shrink: 0; }
</style>
