<script lang="ts">
  import { Map as MapIcon, Plus } from 'lucide-svelte'
  import { maps, loadMaps, createMap } from '../stores/mapStore'
  import { pickAndUploadImage } from '../api/images'
  import { onMount } from 'svelte'

  interface Props {
    onOpenMap: (mapId: string) => void
    onClose: () => void
  }

  let { onOpenMap, onClose }: Props = $props()

  onMount(() => { loadMaps() })

  async function handleCreateMap() {
    const name = prompt('Map name:')
    if (!name) return

    const imageInfo = await pickAndUploadImage()
    const map = await createMap(name, imageInfo?.id)
    onOpenMap(map.id)
  }
</script>

<div class="map-list-view">
  <header class="list-header">
    <div class="header-left">
      <button class="back-btn" onclick={onClose} aria-label="Back">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="15 18 9 12 15 6"></polyline>
        </svg>
      </button>
      <MapIcon size={20} />
      <h2 class="header-title">Atlas</h2>
      <span class="header-count">{$maps.length}</span>
    </div>
    <button class="new-btn" onclick={handleCreateMap}>+ New Map</button>
  </header>

  {#if $maps.length === 0}
    <div class="empty-state">
      <MapIcon size={48} />
      <p class="empty-text">No maps yet</p>
      <button class="empty-create" onclick={handleCreateMap}>Upload your first map</button>
    </div>
  {:else}
    <div class="map-grid">
      {#each $maps as map (map.id)}
        <button class="map-card" onclick={() => onOpenMap(map.id)}>
          <div class="card-cover">
            <MapIcon size={28} />
          </div>
          <div class="card-body">
            <h3 class="card-title">{map.title}</h3>
          </div>
        </button>
      {/each}

      <button class="map-card new-card" onclick={handleCreateMap}>
        <div class="new-content">
          <Plus size={24} />
          <span>New Map</span>
        </div>
      </button>
    </div>
  {/if}
</div>

<style>
  .map-list-view { flex: 1; display: flex; flex-direction: column; height: 100%; overflow: hidden; }

  .list-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 12px 24px; background: var(--color-surface-secondary);
    border-bottom: 1px solid var(--color-border-subtle); flex-shrink: 0;
  }
  .header-left { display: flex; align-items: center; gap: 10px; color: var(--color-fg-tertiary); }
  .back-btn { background: none; border: none; color: var(--color-fg-tertiary); cursor: pointer; padding: 4px; border-radius: var(--radius-sm); }
  .back-btn:hover { background: var(--color-surface-tertiary); color: var(--color-fg-primary); }
  .header-title { font-family: var(--font-heading); font-size: 20px; font-weight: 600; color: var(--color-fg-primary); margin: 0; }
  .header-count { font-family: var(--font-ui); font-size: 12px; color: var(--color-fg-tertiary); background: var(--color-surface-tertiary); padding: 2px 8px; border-radius: 10px; }
  .new-btn { padding: 6px 16px; background: var(--color-accent-gold); border: none; border-radius: var(--radius-sm); font-family: var(--font-ui); font-size: 13px; font-weight: 600; color: var(--color-fg-inverse); cursor: pointer; }

  .empty-state { display: flex; flex-direction: column; align-items: center; justify-content: center; flex: 1; gap: 12px; color: var(--color-fg-tertiary); }
  .empty-text { font-family: var(--font-ui); font-size: 16px; margin: 0; }
  .empty-create { padding: 8px 20px; background: var(--color-accent-gold); border: none; border-radius: var(--radius-sm); font-family: var(--font-ui); font-size: 13px; font-weight: 600; color: var(--color-fg-inverse); cursor: pointer; }

  .map-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px; padding: 24px; overflow-y: auto; flex: 1; }
  .map-card { background: var(--color-surface-card); border: 1px solid var(--color-border-default); border-radius: var(--radius-md); overflow: hidden; cursor: pointer; text-align: left; display: flex; flex-direction: column; }
  .map-card:hover { border-color: var(--color-accent-gold); }
  .card-cover { height: 100px; display: flex; align-items: center; justify-content: center; background: var(--color-surface-tertiary); color: var(--color-fg-tertiary); opacity: 0.3; }
  .card-body { padding: 12px; }
  .card-title { font-family: var(--font-heading); font-size: 15px; font-weight: 600; color: var(--color-fg-primary); margin: 0; }
  .new-card { border-style: dashed; }
  .new-content { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; flex: 1; padding: 24px; color: var(--color-accent-gold); font-family: var(--font-ui); font-size: 13px; font-weight: 600; }
</style>
