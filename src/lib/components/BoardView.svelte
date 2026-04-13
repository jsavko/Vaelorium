<script lang="ts">
  import { Layout, Plus, Trash2 } from 'lucide-svelte'
  import { currentBoard, currentCards, currentConnectors, loadBoard, addCard, moveCard, removeCard, addConnector, removeConnector } from '../stores/boardStore'
  import { loadPage, pageTree } from '../stores/pageStore'
  import { entityTypeMap } from '../stores/entityTypeStore'
  import { onMount } from 'svelte'
  import InputModal from './InputModal.svelte'

  interface Props {
    boardId: string
    onClose: () => void
  }

  let { boardId, onClose }: Props = $props()

  let container: HTMLDivElement
  let transform = $state({ x: 0, y: 0, scale: 1 })
  let panning = $state(false)
  let panStart = { x: 0, y: 0 }
  let draggingCard = $state<string | null>(null)
  let dragOffset = { x: 0, y: 0 }
  let connecting = $state<string | null>(null)
  let mousePos = $state({ x: 0, y: 0 })
  let cardModalOpen = $state(false)
  let pendingCardPos = $state({ x: 0, y: 0 })

  onMount(() => loadBoard(boardId))

  function screenToWorld(sx: number, sy: number) {
    const rect = container.getBoundingClientRect()
    return { x: (sx - rect.left - transform.x) / transform.scale, y: (sy - rect.top - transform.y) / transform.scale }
  }

  function handleMouseDown(e: MouseEvent) {
    if (draggingCard || connecting) return
    panning = true
    panStart = { x: e.clientX - transform.x, y: e.clientY - transform.y }
  }

  function handleMouseMove(e: MouseEvent) {
    mousePos = { x: e.clientX, y: e.clientY }
    if (draggingCard) {
      const pos = screenToWorld(e.clientX, e.clientY)
      moveCard(draggingCard, pos.x - dragOffset.x, pos.y - dragOffset.y)
    } else if (panning) {
      transform = { ...transform, x: e.clientX - panStart.x, y: e.clientY - panStart.y }
    }
  }

  function handleMouseUp() {
    draggingCard = null
    panning = false
  }

  function handleWheel(e: WheelEvent) {
    e.preventDefault()
    const factor = e.deltaY > 0 ? 0.9 : 1.1
    const newScale = Math.max(0.1, Math.min(5, transform.scale * factor))
    const rect = container.getBoundingClientRect()
    const mx = e.clientX - rect.left
    const my = e.clientY - rect.top
    transform = { scale: newScale, x: mx - (mx - transform.x) * (newScale / transform.scale), y: my - (my - transform.y) * (newScale / transform.scale) }
  }

  function handleCanvasClick(e: MouseEvent) {
    if (connecting) { connecting = null; return }
  }

  function handleCanvasDblClick(e: MouseEvent) {
    const pos = screenToWorld(e.clientX, e.clientY)
    pendingCardPos = pos
    cardModalOpen = true
  }

  function startDragCard(cardId: string, e: MouseEvent) {
    e.stopPropagation()
    if (e.shiftKey) {
      connecting = cardId
      return
    }
    const card = $currentCards.find((c) => c.id === cardId)
    if (!card) return
    const pos = screenToWorld(e.clientX, e.clientY)
    dragOffset = { x: pos.x - card.x, y: pos.y - card.y }
    draggingCard = cardId
  }

  function startConnect(cardId: string, e: MouseEvent) {
    e.stopPropagation()
    connecting = cardId
  }

  async function finishConnect(targetId: string, e: MouseEvent) {
    e.stopPropagation()
    if (connecting && connecting !== targetId && $currentBoard) {
      await addConnector($currentBoard.id, connecting, targetId)
    }
    connecting = null
  }

  async function handleCreateCard(text: string) {
    cardModalOpen = false
    if ($currentBoard) {
      await addCard($currentBoard.id, pendingCardPos.x, pendingCardPos.y, text)
    }
  }

  function getCardColor(card: any): string {
    if (card.color) return card.color
    if (card.page_id) {
      const page = $pageTree.find((p) => p.id === card.page_id)
      if (page?.entity_type_id) return $entityTypeMap.get(page.entity_type_id)?.color || '#4A3F36'
    }
    return '#4A3F36'
  }

  function getCardCenter(card: any): { x: number; y: number } {
    return { x: card.x + card.width / 2, y: card.y + card.height / 2 }
  }
</script>

<div class="board-view">
  <header class="board-header">
    <div class="header-left">
      <button class="back-btn" onclick={onClose} aria-label="Back">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"></polyline></svg>
      </button>
      <Layout size={20} />
      <h2 class="header-title">{$currentBoard?.name || 'Board'}</h2>
    </div>
    <div class="header-hint">Double-click to add card · Shift+drag card edge to connect</div>
  </header>

  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="board-canvas"
    bind:this={container}
    onmousedown={handleMouseDown}
    onmousemove={handleMouseMove}
    onmouseup={handleMouseUp}
    onmouseleave={handleMouseUp}
    onwheel={handleWheel}
    onclick={handleCanvasClick}
    ondblclick={handleCanvasDblClick}
    style:cursor={panning ? 'grabbing' : connecting ? 'crosshair' : 'grab'}
  >
    <!-- SVG connectors -->
    <svg class="connector-layer" style:transform="translate({transform.x}px, {transform.y}px) scale({transform.scale})" style:transform-origin="0 0">
      {#each $currentConnectors as conn (conn.id)}
        {@const src = $currentCards.find((c) => c.id === conn.source_card_id)}
        {@const tgt = $currentCards.find((c) => c.id === conn.target_card_id)}
        {#if src && tgt}
          {@const s = getCardCenter(src)}
          {@const t = getCardCenter(tgt)}
          <line x1={s.x} y1={s.y} x2={t.x} y2={t.y} stroke={conn.color || '#C8A55C'} stroke-width="2" opacity="0.6" />
          {#if conn.label}
            <text x={(s.x + t.x) / 2} y={(s.y + t.y) / 2 - 6} fill="#B8A690" font-size="11" text-anchor="middle" font-family="Inter, sans-serif">{conn.label}</text>
          {/if}
        {/if}
      {/each}
      {#if connecting}
        {@const srcCard = $currentCards.find((c) => c.id === connecting)}
        {#if srcCard}
          {@const s = getCardCenter(srcCard)}
          {@const m = screenToWorld(mousePos.x, mousePos.y)}
          <line x1={s.x} y1={s.y} x2={m.x} y2={m.y} stroke="#C8A55C" stroke-width="2" stroke-dasharray="6 4" opacity="0.8" />
        {/if}
      {/if}
    </svg>

    <!-- Cards -->
    {#each $currentCards as card (card.id)}
      {@const cx = transform.x + card.x * transform.scale}
      {@const cy = transform.y + card.y * transform.scale}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="board-card"
        style:left="{cx}px"
        style:top="{cy}px"
        style:width="{card.width * transform.scale}px"
        style:--card-color={getCardColor(card)}
        onmousedown={(e) => startDragCard(card.id, e)}
        onmouseup={(e) => { if (connecting && connecting !== card.id) { e.stopPropagation(); finishConnect(card.id, e) } }}
        onclick={(e) => e.stopPropagation()}
        ondblclick={(e) => {
          e.stopPropagation()
          if (card.page_id) { loadPage(card.page_id); window.dispatchEvent(new CustomEvent('vaelorium:page-selected')) }
        }}
      >
        <div class="card-content">
          {card.content || ''}
          {#if card.page_id}
            {@const page = $pageTree.find((p) => p.id === card.page_id)}
            {#if page}
              <span class="card-page-link">📄 {page.title}</span>
            {/if}
          {/if}
        </div>
        <div class="card-actions">
          <button class="card-connect" onmousedown={(e) => { if (e.shiftKey) startConnect(card.id, e) }} onclick={(e) => { if (connecting) finishConnect(card.id, e) }} title="Shift+drag to connect">⟷</button>
          <button class="card-delete" onclick={(e) => { e.stopPropagation(); removeCard(card.id) }} title="Delete">×</button>
        </div>
      </div>
    {/each}

    {#if $currentCards.length === 0}
      <div class="empty-overlay">
        <p>Double-click anywhere to add your first card</p>
      </div>
    {/if}
  </div>
</div>

<InputModal
  open={cardModalOpen}
  title="Add Card"
  placeholder="Card text..."
  confirmLabel="Add"
  onConfirm={handleCreateCard}
  onCancel={() => cardModalOpen = false}
/>

<style>
  .board-view { flex: 1; display: flex; flex-direction: column; height: 100%; overflow: hidden; }
  .board-header { display: flex; align-items: center; justify-content: space-between; padding: 10px 20px; background: var(--color-surface-secondary); border-bottom: 1px solid var(--color-border-subtle); flex-shrink: 0; }
  .header-left { display: flex; align-items: center; gap: 10px; color: var(--color-fg-tertiary); }
  .back-btn { background: none; border: none; color: var(--color-fg-tertiary); cursor: pointer; padding: 4px; border-radius: var(--radius-sm); }
  .back-btn:hover { background: var(--color-surface-tertiary); color: var(--color-fg-primary); }
  .header-title { font-family: var(--font-heading); font-size: 18px; font-weight: 600; color: var(--color-fg-primary); margin: 0; }
  .header-hint { font-family: var(--font-ui); font-size: 11px; color: var(--color-fg-tertiary); opacity: 0.6; }

  .board-canvas { flex: 1; position: relative; overflow: hidden; background: var(--color-surface-primary); }

  .connector-layer { position: absolute; top: 0; left: 0; width: 100%; height: 100%; pointer-events: none; overflow: visible; }

  .board-card {
    position: absolute; background: var(--color-surface-card); border: 1px solid var(--color-border-default);
    border-left: 3px solid var(--card-color, #4A3F36); border-radius: var(--radius-md);
    cursor: grab; user-select: none; display: flex; flex-direction: column; min-height: 60px;
  }
  .board-card:hover { border-color: var(--color-accent-gold); }
  .board-card:hover .card-actions { opacity: 1; }

  .card-content {
    padding: 10px 12px; font-family: var(--font-ui); font-size: 13px; color: var(--color-fg-primary);
    flex: 1; overflow: hidden;
  }
  .card-page-link { display: block; margin-top: 6px; font-size: 11px; color: var(--color-accent-gold); }

  .card-actions {
    display: flex; justify-content: space-between; padding: 4px 8px;
    opacity: 0; transition: opacity 0.1s; border-top: 1px solid var(--color-border-subtle);
  }
  .card-connect, .card-delete {
    background: none; border: none; color: var(--color-fg-tertiary); cursor: pointer; font-size: 14px; padding: 2px 4px;
  }
  .card-connect:hover { color: var(--color-accent-gold); }
  .card-delete:hover { color: var(--color-status-error); }

  .empty-overlay { position: absolute; inset: 0; display: flex; align-items: center; justify-content: center; color: var(--color-fg-tertiary); font-family: var(--font-ui); pointer-events: none; }
</style>
