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

  function handleMapClick(e: MouseEvent) {
    if (!addingPin) return
    const rect = container.getBoundingClientRect()
    const mx = e.clientX - rect.left
    const my = e.clientY - rect.top
    // Convert to normalized coords
    const nx = (mx - transform.x) / (imgWidth * transform.scale)
    const ny = (my - transform.y) / (imgHeight * transform.scale)
    if (nx >= 0 && nx <= 1 && ny >= 0 && ny <= 1) {
      pinForm = { x: nx, y: ny, label: '', pageId: '' }
      addingPin = false
    }
  }

  async function savePinForm() {
    if (!pinForm || !$currentMap) return
    await addPin($currentMap.id, pinForm.x, pinForm.y, pinForm.pageId || null, pinForm.label || null)
    pinForm = null
  }

  function cancelPinForm() {
    pinForm = null
  }

  let filteredPages = $derived(
    searchQuery.length > 0
      ? $pageTree.filter((p) => p.title.toLowerCase().includes(searchQuery.toLowerCase())).slice(0, 6)
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
      <h2 class="header-title">{$currentMap?.title || 'Map'}</h2>
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
          onclick={(e) => { e.stopPropagation(); if (pin.page_id) loadPage(pin.page_id); onClose() }}
          title={pin.label || 'Pin'}
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
      <div class="pin-form" style:left="{fx + 16}px" style:top="{fy}px">
        <input class="pin-input" bind:value={pinForm.label} placeholder="Pin label..." />
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

  .header-title { font-family: var(--font-heading); font-size: 18px; font-weight: 600; color: var(--color-fg-primary); margin: 0; }
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
    position: absolute; transform: translate(-50%, -100%); background: none; border: none;
    color: var(--pin-color, #C8A55C); cursor: pointer; display: flex; flex-direction: column; align-items: center; z-index: 5;
    filter: drop-shadow(0 2px 4px rgba(0,0,0,0.5));
  }
  .map-pin:hover { transform: translate(-50%, -100%) scale(1.2); }

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
