<script lang="ts">
  import { MapPin as MapPinIcon } from 'lucide-svelte'
  import { currentMap, currentMapPins, loadMap, addPin, removePin } from '../stores/mapStore'
  import { loadPage, pageTree } from '../stores/pageStore'
  import { entityTypeMap } from '../stores/entityTypeStore'
  import { getImageUrl } from '../api/images'
  import { onMount } from 'svelte'

  interface Props {
    mapId: string
    onClose: () => void
  }

  let { mapId, onClose }: Props = $props()

  let container: HTMLDivElement
  let imageUrl = $state('')
  let transform = $state({ x: 0, y: 0, scale: 1 })
  let panning = $state(false)
  let panStart = { x: 0, y: 0 }
  let addingPin = $state(false)
  let pinForm = $state<{ x: number; y: number; label: string; pageId: string } | null>(null)
  let searchQuery = $state('')

  let imgWidth = $state(800)
  let imgHeight = $state(600)
  let editingPin = $state<MapPin | null>(null)
  let editLabel = $state('')
  let editPageId = $state('')
  let editSearchQuery = $state('')
  let editColor = $state('#C8A55C')

  onMount(async () => {
    await loadMap(mapId)
    if ($currentMap?.image_id) {
      imageUrl = await getImageUrl($currentMap.image_id)
    }
  })

  function handleImgLoad(e: Event) {
    const img = e.target as HTMLImageElement
    imgWidth = img.naturalWidth
    imgHeight = img.naturalHeight
    fitToView()
  }

  function fitToView() {
    if (!container || !imgWidth || !imgHeight) return
    const rect = container.getBoundingClientRect()
    const scaleX = rect.width / imgWidth
    const scaleY = rect.height / imgHeight
    const scale = Math.min(scaleX, scaleY) * 0.95
    transform = {
      scale,
      x: (rect.width - imgWidth * scale) / 2,
      y: (rect.height - imgHeight * scale) / 2,
    }
  }

  function handleMouseDown(e: MouseEvent) {
    if (addingPin) return
    panning = true
    panStart = { x: e.clientX - transform.x, y: e.clientY - transform.y }
  }

  function handleMouseMove(e: MouseEvent) {
    if (panning) {
      transform = { ...transform, x: e.clientX - panStart.x, y: e.clientY - panStart.y }
    }
  }

  function handleMouseUp() {
    panning = false
  }

  function handleWheel(e: WheelEvent) {
    e.preventDefault()
    const factor = e.deltaY > 0 ? 0.9 : 1.1
    const newScale = Math.max(0.1, Math.min(10, transform.scale * factor))
    const mx = e.clientX - container.getBoundingClientRect().left
    const my = e.clientY - container.getBoundingClientRect().top
    transform = {
      scale: newScale,
      x: mx - (mx - transform.x) * (newScale / transform.scale),
      y: my - (my - transform.y) * (newScale / transform.scale),
    }
  }

  function clientToNormalized(clientX: number, clientY: number): { nx: number; ny: number } {
    const rect = container.getBoundingClientRect()
    const mx = clientX - rect.left
    const my = clientY - rect.top
    return {
      nx: (mx - transform.x) / (imgWidth * transform.scale),
      ny: (my - transform.y) / (imgHeight * transform.scale),
    }
  }

  function handleMapClick(e: MouseEvent) {
    if (editingPin) { editingPin = null; return }
    if (!addingPin) return
    const { nx, ny } = clientToNormalized(e.clientX, e.clientY)
    if (nx >= 0 && nx <= 1 && ny >= 0 && ny <= 1) {
      pinForm = { x: nx, y: ny, label: '', pageId: '', color: '#C8A55C' }
      addingPin = false
    }
  }

  async function savePinForm() {
    if (!pinForm || !$currentMap) return
    await addPin($currentMap.id, pinForm.x, pinForm.y, pinForm.pageId || null, pinForm.label || null, null, pinForm.color || null)
    pinForm = null
  }

  function cancelPinForm() {
    pinForm = null
  }

  function startEditPin(pin: any, e: MouseEvent) {
    e.stopPropagation()
    editingPin = pin
    editLabel = pin.label || ''
    editPageId = pin.page_id || ''
    editColor = pin.color || getPinColor(pin)
    editSearchQuery = ''
    if (pin.page_id) {
      const page = $pageTree.find((p: any) => p.id === pin.page_id)
      editSearchQuery = page?.title || ''
    }
  }

  async function saveEditPin() {
    if (!editingPin) return
    const { updatePin: updatePinApi } = await import('../api/maps')
    // Update label, page, and color
    await updatePinApi(editingPin.id, {
      label: editLabel || undefined,
      pageId: editPageId || undefined,
      color: editColor || undefined,
    })
    // Reload pins
    if ($currentMap) await loadMap($currentMap.id)
    editingPin = null
  }

  async function deleteEditPin() {
    if (!editingPin) return
    await removePin(editingPin.id)
    editingPin = null
  }

  function selectEditPage(pageId: string, title: string) {
    editPageId = pageId
    editSearchQuery = title
  }

  let filteredPages = $derived(
    searchQuery.length > 0
      ? $pageTree.filter((p) => p.title.toLowerCase().includes(searchQuery.toLowerCase())).slice(0, 6)
      : [],
  )

  let editFilteredPages = $derived(
    editSearchQuery.length > 0
      ? $pageTree.filter((p) => p.title.toLowerCase().includes(editSearchQuery.toLowerCase())).slice(0, 6)
      : [],
  )

  function selectPage(pageId: string, title: string) {
    if (pinForm) {
      pinForm = { ...pinForm, pageId, label: pinForm.label || title }
      searchQuery = title
    }
  }

  function getPinColor(pin: any): string {
    if (pin.color) return pin.color
    if (pin.page_id) {
      const page = $pageTree.find((p) => p.id === pin.page_id)
      if (page?.entity_type_id) {
        return $entityTypeMap.get(page.entity_type_id)?.color || '#C8A55C'
      }
    }
    return '#C8A55C'
  }
</script>

<div class="map-viewer">
  <header class="map-header">
    <div class="header-left">
      <button class="back-btn" onclick={onClose} aria-label="Back">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="15 18 9 12 15 6"></polyline>
        </svg>
      </button>
      <input
        class="header-title-input"
        value={$currentMap?.title || 'Map'}
        onblur={async (e) => {
          const val = (e.target as HTMLInputElement).value.trim()
          if (val && $currentMap && val !== $currentMap.title) {
            const { callCommand } = await import('../api/bridge')
            // Update map title via direct SQL — no dedicated command yet
            // For now just update local state
            currentMap.update(m => m ? { ...m, title: val } : m)
          }
        }}
      />
    </div>
    <div class="header-right">
      <button
        class="tool-btn"
        class:active={addingPin}
        onclick={() => addingPin = !addingPin}
      >
        {addingPin ? 'Cancel' : '+ Add Pin'}
      </button>
    </div>
  </header>

  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div
    class="map-canvas"
    bind:this={container}
    onmousedown={handleMouseDown}
    onmousemove={handleMouseMove}
    onmouseup={handleMouseUp}
    onmouseleave={handleMouseUp}
    onwheel={handleWheel}
    onclick={handleMapClick}
    style:cursor={addingPin ? 'crosshair' : panning ? 'grabbing' : 'grab'}
  >
    {#if imageUrl}
      <img
        class="map-image"
        src={imageUrl}
        alt={$currentMap?.title || 'Map'}
        style:transform="translate({transform.x}px, {transform.y}px) scale({transform.scale})"
        style:transform-origin="0 0"
        onload={handleImgLoad}
        draggable="false"
      />

      {#each $currentMapPins as pin (pin.id)}
        {@const px = transform.x + pin.x * imgWidth * transform.scale}
        {@const py = transform.y + pin.y * imgHeight * transform.scale}
        <button
          class="map-pin"
          style:left="{px}px"
          style:top="{py}px"
          style:--pin-color={getPinColor(pin)}
          onclick={(e) => startEditPin(pin, e)}
          ondblclick={(e) => { e.stopPropagation(); if (pin.page_id) { loadPage(pin.page_id); onClose() } }}
          title={pin.label || 'Click to edit, double-click to open page'}
        >
          <MapPinIcon size={20} />
          {#if pin.label}
            <span class="pin-label">{pin.label}</span>
          {/if}
        </button>
      {/each}
    {:else}
      <div class="no-image">No map image uploaded</div>
    {/if}

    {#if pinForm}
      {@const fx = transform.x + pinForm.x * imgWidth * transform.scale}
      {@const fy = transform.y + pinForm.y * imgHeight * transform.scale}
      <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
      <div class="pin-form" style:left="{fx + 16}px" style:top="{fy}px" onclick={(e) => e.stopPropagation()} onmousedown={(e) => e.stopPropagation()}>
        <input class="pin-input" bind:value={pinForm.label} placeholder="Pin label..." />
        <div class="color-row">
          <label class="color-label">Color</label>
          <input type="color" class="color-picker" bind:value={pinForm.color} />
        </div>
        <div class="pin-search-wrapper">
          <input class="pin-input" bind:value={searchQuery} placeholder="Link to page..." />
          {#if filteredPages.length > 0}
            <div class="pin-search-results">
              {#each filteredPages as page (page.id)}
                <button class="pin-search-result" onclick={() => selectPage(page.id, page.title)}>
                  {page.title}
                </button>
              {/each}
            </div>
          {/if}
        </div>
        <div class="pin-form-actions">
          <button class="pin-cancel" onclick={cancelPinForm}>Cancel</button>
          <button class="pin-save" onclick={savePinForm}>Add Pin</button>
        </div>
      </div>
    {/if}

    {#if editingPin}
      {@const ex = transform.x + editingPin.x * imgWidth * transform.scale}
      {@const ey = transform.y + editingPin.y * imgHeight * transform.scale}
      <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
      <div class="pin-form" style:left="{ex + 16}px" style:top="{ey}px" onclick={(e) => e.stopPropagation()} onmousedown={(e) => e.stopPropagation()}>
        <div class="pin-form-header">
          <span class="pin-form-title">Edit Pin</span>
          <button class="pin-delete" onclick={deleteEditPin} title="Delete pin">×</button>
        </div>
        <input class="pin-input" bind:value={editLabel} placeholder="Pin label..." />
        <div class="color-row">
          <label class="color-label">Color</label>
          <input type="color" class="color-picker" bind:value={editColor} />
        </div>
        <div class="pin-search-wrapper">
          <input class="pin-input" bind:value={editSearchQuery} placeholder="Link to page..." />
          {#if editFilteredPages.length > 0}
            <div class="pin-search-results">
              {#each editFilteredPages as page (page.id)}
                <button class="pin-search-result" onclick={() => selectEditPage(page.id, page.title)}>
                  {page.title}
                </button>
              {/each}
            </div>
          {/if}
        </div>
        <div class="pin-form-actions">
          <button class="pin-danger" onclick={deleteEditPin}>Delete</button>
          <span style="flex:1"></span>
          <button class="pin-cancel" onclick={() => editingPin = null}>Cancel</button>
          <button class="pin-save" onclick={saveEditPin}>Save</button>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .map-viewer {
    flex: 1;
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .map-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 20px;
    background: var(--color-surface-secondary);
    border-bottom: 1px solid var(--color-border-subtle);
    flex-shrink: 0;
  }

  .header-left { display: flex; align-items: center; gap: 10px; }

  .back-btn {
    background: none; border: none; color: var(--color-fg-tertiary); cursor: pointer; padding: 4px; border-radius: var(--radius-sm);
  }
  .back-btn:hover { background: var(--color-surface-tertiary); color: var(--color-fg-primary); }

  .header-title-input {
    font-family: var(--font-heading); font-size: 18px; font-weight: 600; color: var(--color-fg-primary);
    background: none; border: none; outline: none; padding: 0; margin: 0;
  }
  .header-title-input:focus { border-bottom: 1px solid var(--color-accent-gold); }
  .header-right { display: flex; gap: 8px; }

  .tool-btn {
    padding: 4px 14px; background: var(--color-surface-tertiary); border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm); font-family: var(--font-ui); font-size: 12px; color: var(--color-fg-secondary); cursor: pointer;
  }
  .tool-btn:hover { border-color: var(--color-accent-gold); }
  .tool-btn.active { background: var(--color-accent-gold-subtle); color: var(--color-accent-gold); border-color: var(--color-accent-gold); }

  .map-canvas {
    flex: 1; position: relative; overflow: hidden; background: var(--color-surface-primary);
  }

  .map-image {
    position: absolute; top: 0; left: 0; max-width: none; user-select: none; pointer-events: none;
  }

  .map-pin {
    position: absolute; background: none; border: none;
    color: var(--pin-color, #C8A55C); cursor: pointer; display: flex; flex-direction: column; align-items: center; z-index: 5;
    filter: drop-shadow(0 2px 4px rgba(0,0,0,0.5));
    /* Offset so the pin tip lands on the coordinate */
    margin-left: -10px;
    margin-top: -20px;
  }
  .map-pin:hover { transform: scale(1.2); transform-origin: bottom center; }

  .pin-label {
    font-family: var(--font-ui); font-size: 10px; color: var(--color-fg-primary);
    background: var(--color-surface-card); padding: 1px 6px; border-radius: 4px; margin-top: 2px;
    white-space: nowrap; box-shadow: 0 1px 4px rgba(0,0,0,0.3);
  }

  .pin-form {
    position: absolute; z-index: 20; background: var(--color-surface-card);
    border: 1px solid var(--color-border-default); border-radius: var(--radius-md);
    padding: 12px; width: 220px; display: flex; flex-direction: column; gap: 8px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.4);
  }

  .pin-form-header {
    display: flex; align-items: center; justify-content: space-between;
  }
  .pin-form-title { font-family: var(--font-ui); font-size: 12px; font-weight: 600; color: var(--color-fg-tertiary); }
  .pin-delete {
    background: none; border: none; color: var(--color-fg-tertiary); font-size: 18px; cursor: pointer; padding: 0 2px;
  }
  .pin-delete:hover { color: var(--color-status-error); }
  .pin-danger {
    padding: 4px 10px; background: none; border: 1px solid var(--color-status-error);
    border-radius: var(--radius-sm); font-family: var(--font-ui); font-size: 12px;
    color: var(--color-status-error); cursor: pointer;
  }
  .pin-danger:hover { background: rgba(184, 92, 92, 0.15); }

  .color-row {
    display: flex; align-items: center; gap: 8px;
  }
  .color-label {
    font-family: var(--font-ui); font-size: 12px; color: var(--color-fg-tertiary);
  }
  .color-picker {
    width: 32px; height: 28px; border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm); background: none; cursor: pointer; padding: 2px;
  }

  .pin-input {
    width: 100%; padding: 6px 10px; background: var(--color-surface-primary);
    border: 1px solid var(--color-border-default); border-radius: var(--radius-sm);
    font-family: var(--font-ui); font-size: 13px; color: var(--color-fg-primary); outline: none; box-sizing: border-box;
  }
  .pin-input:focus { border-color: var(--color-accent-gold); }

  .pin-search-wrapper { position: relative; }
  .pin-search-results {
    position: absolute; top: 100%; left: 0; right: 0; background: var(--color-surface-card);
    border: 1px solid var(--color-border-default); border-radius: var(--radius-sm);
    max-height: 150px; overflow-y: auto; z-index: 10;
  }
  .pin-search-result {
    display: block; width: 100%; padding: 6px 10px; background: none; border: none; text-align: left;
    font-family: var(--font-ui); font-size: 12px; color: var(--color-fg-primary); cursor: pointer;
  }
  .pin-search-result:hover { background: var(--color-surface-tertiary); }

  .pin-form-actions { display: flex; justify-content: flex-end; gap: 6px; }
  .pin-cancel {
    padding: 4px 10px; background: var(--color-surface-tertiary); border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm); font-family: var(--font-ui); font-size: 12px; color: var(--color-fg-secondary); cursor: pointer;
  }
  .pin-save {
    padding: 4px 10px; background: var(--color-accent-gold); border: none;
    border-radius: var(--radius-sm); font-family: var(--font-ui); font-size: 12px; font-weight: 600;
    color: var(--color-fg-inverse); cursor: pointer;
  }

  .no-image {
    display: flex; align-items: center; justify-content: center; height: 100%;
    font-family: var(--font-ui); font-size: 16px; color: var(--color-fg-tertiary);
  }
</style>
