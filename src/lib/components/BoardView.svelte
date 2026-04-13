<script lang="ts">
  import { Layout } from 'lucide-svelte'
  import {
    currentBoard,
    currentCards,
    currentConnectors,
    loadBoard,
    addCard,
    moveCard,
    resizeCard,
    removeCard,
    addConnector,
    updateCardContent,
  } from '../stores/boardStore'
  import { loadPage, pageTree } from '../stores/pageStore'
  import { entityTypeMap } from '../stores/entityTypeStore'
  import { onMount } from 'svelte'
  import BoardCardEditor from './BoardCardEditor.svelte'

  interface Props {
    boardId: string
    onClose: () => void
  }

  let { boardId, onClose }: Props = $props()

  const MIN_CARD_W = 160
  const MIN_CARD_H = 100
  const DEFAULT_CARD_W = 240
  const DEFAULT_CARD_H = 140

  let container: HTMLDivElement
  let transform = $state({ x: 0, y: 0, scale: 1 })
  let panning = $state(false)
  let panStart = { x: 0, y: 0 }
  let draggingCard = $state<string | null>(null)
  let dragOffset = { x: 0, y: 0 }
  let resizingCard = $state<string | null>(null)
  let resizeStart = { w: 0, h: 0, mx: 0, my: 0 }
  let connecting = $state<string | null>(null)
  let mousePos = $state({ x: 0, y: 0 })
  let editingCard = $state<string | null>(null)

  onMount(() => loadBoard(boardId))

  function screenToWorld(sx: number, sy: number) {
    const rect = container.getBoundingClientRect()
    return { x: (sx - rect.left - transform.x) / transform.scale, y: (sy - rect.top - transform.y) / transform.scale }
  }

  function handleMouseDown(e: MouseEvent) {
    if (draggingCard || connecting || resizingCard || editingCard) return
    panning = true
    panStart = { x: e.clientX - transform.x, y: e.clientY - transform.y }
  }

  function handleMouseMove(e: MouseEvent) {
    mousePos = { x: e.clientX, y: e.clientY }
    if (draggingCard) {
      const pos = screenToWorld(e.clientX, e.clientY)
      moveCard(draggingCard, pos.x - dragOffset.x, pos.y - dragOffset.y)
    } else if (resizingCard) {
      const dx = (e.clientX - resizeStart.mx) / transform.scale
      const dy = (e.clientY - resizeStart.my) / transform.scale
      const nw = Math.max(MIN_CARD_W, resizeStart.w + dx)
      const nh = Math.max(MIN_CARD_H, resizeStart.h + dy)
      resizeCard(resizingCard, nw, nh)
    } else if (panning) {
      transform = { ...transform, x: e.clientX - panStart.x, y: e.clientY - panStart.y }
    }
  }

  function handleMouseUp() {
    draggingCard = null
    panning = false
    resizingCard = null
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

  function handleCanvasClick(_e: MouseEvent) {
    if (connecting) { connecting = null; return }
  }

  async function handleCanvasDblClick(e: MouseEvent) {
    if (editingCard) return
    const pos = screenToWorld(e.clientX, e.clientY)
    if ($currentBoard) {
      const card = await addCard(
        $currentBoard.id,
        pos.x - DEFAULT_CARD_W / 2,
        pos.y - DEFAULT_CARD_H / 2,
        '',
      )
      editingCard = card.id
    }
  }

  function startDragCard(cardId: string, e: MouseEvent) {
    if (editingCard === cardId) return
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

  function startResize(cardId: string, e: MouseEvent) {
    e.stopPropagation()
    e.preventDefault()
    const card = $currentCards.find((c) => c.id === cardId)
    if (!card) return
    resizingCard = cardId
    resizeStart = { w: card.width, h: card.height, mx: e.clientX, my: e.clientY }
  }

  async function finishConnect(targetId: string, e: MouseEvent) {
    e.stopPropagation()
    if (connecting && connecting !== targetId && $currentBoard) {
      await addConnector($currentBoard.id, connecting, targetId)
    }
    connecting = null
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

  // Clicks on a rendered mention/wiki-link inside a card navigate to the
  // referenced entity. Mirrors the handler in Editor.svelte so card links
  // feel identical to page-body links.
  function handleCardBodyClick(e: MouseEvent) {
    const target = e.target as HTMLElement
    const link = target.closest('a[href^="#page:"], a[href^="#map:"], a[href^="#timeline:"]')
    if (!link) return
    e.preventDefault()
    e.stopPropagation()
    const href = link.getAttribute('href')
    if (!href) return
    if (href.startsWith('#page:')) {
      const pageId = href.replace('#page:', '')
      loadPage(pageId)
      window.dispatchEvent(new CustomEvent('vaelorium:page-selected'))
    } else if (href.startsWith('#map:')) {
      const mapId = href.replace('#map:', '')
      window.dispatchEvent(new CustomEvent('vaelorium:open-map', { detail: { mapId } }))
    } else if (href.startsWith('#timeline:')) {
      const timelineId = href.replace('#timeline:', '')
      window.dispatchEvent(new CustomEvent('vaelorium:open-timeline', { detail: { timelineId } }))
    }
  }

  function startEditCard(cardId: string, e: MouseEvent) {
    e.stopPropagation()
    editingCard = cardId
  }

  async function saveEdit(cardId: string, html: string) {
    await updateCardContent(cardId, html)
    editingCard = null
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
    <div class="header-hint">Double-click canvas to add card · Double-click card to edit · @-type to link a page · Shift+drag a card to connect</div>
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
    style:cursor={panning ? 'grabbing' : connecting ? 'crosshair' : resizingCard ? 'nwse-resize' : 'grab'}
  >
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

    {#each $currentCards as card (card.id)}
      {@const cx = transform.x + card.x * transform.scale}
      {@const cy = transform.y + card.y * transform.scale}
      {@const cw = Math.max(MIN_CARD_W, card.width) * transform.scale}
      {@const ch = Math.max(MIN_CARD_H, card.height) * transform.scale}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="board-card"
        class:editing={editingCard === card.id}
        style:left="{cx}px"
        style:top="{cy}px"
        style:width="{cw}px"
        style:min-height="{ch}px"
        style:--card-color={getCardColor(card)}
        onmousedown={(e) => startDragCard(card.id, e)}
        onmouseup={(e) => { if (connecting && connecting !== card.id) { e.stopPropagation(); finishConnect(card.id, e) } }}
        onclick={(e) => e.stopPropagation()}
        ondblclick={(e) => startEditCard(card.id, e)}
      >
        {#if card.page_id}
          {@const page = $pageTree.find((p) => p.id === card.page_id)}
          {#if page}
            <button
              class="card-page-pill"
              onmousedown={(e) => e.stopPropagation()}
              onclick={(e) => { e.stopPropagation(); if (card.page_id) { loadPage(card.page_id); window.dispatchEvent(new CustomEvent('vaelorium:page-selected')) } }}
              title="Open {page.title}"
            >
              📄 {page.title}
            </button>
          {/if}
        {/if}

        {#if editingCard === card.id}
          <BoardCardEditor
            initialHtml={card.content || ''}
            onSave={(html) => saveEdit(card.id, html)}
            onCancel={() => saveEdit(card.id, card.content || '')}
          />
        {:else}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <div class="card-content" onclick={handleCardBodyClick}>
            {#if card.content && card.content.trim() !== '' && card.content !== '<p></p>'}
              <!-- eslint-disable-next-line svelte/no-at-html-tags -->
              {@html card.content}
            {:else}
              <span class="card-placeholder">Double-click to edit…</span>
            {/if}
          </div>
        {/if}

        <div class="card-actions">
          <button class="card-connect" onmousedown={(e) => { e.stopPropagation(); if (e.shiftKey) { connecting = card.id } }} onclick={(e) => { if (connecting) finishConnect(card.id, e) }} title="Shift+drag to connect">⟷</button>
          <button class="card-delete" onclick={(e) => { e.stopPropagation(); removeCard(card.id) }} title="Delete">×</button>
        </div>

        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div class="card-resize" onmousedown={(e) => startResize(card.id, e)} title="Drag to resize"></div>
      </div>
    {/each}

    {#if $currentCards.length === 0}
      <div class="empty-overlay">
        <p>Double-click anywhere to add your first card</p>
      </div>
    {/if}
  </div>
</div>

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
    cursor: grab; user-select: none; display: flex; flex-direction: column;
  }
  .board-card:hover { border-color: var(--color-accent-gold); }
  .board-card:hover .card-actions,
  .board-card:hover .card-resize { opacity: 1; }
  .board-card.editing { cursor: default; border-color: var(--color-accent-gold); box-shadow: 0 0 0 2px var(--color-accent-gold); user-select: text; }

  .card-page-pill {
    align-self: flex-start;
    margin: 8px 8px 0 12px;
    padding: 2px 8px;
    background: transparent;
    border: 1px solid var(--color-border-default);
    border-radius: 10px;
    font-family: var(--font-ui);
    font-size: 11px;
    color: var(--color-accent-gold);
    cursor: pointer;
  }
  .card-page-pill:hover { border-color: var(--color-accent-gold); background: var(--color-surface-tertiary); }

  .card-content {
    padding: 10px 12px;
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-primary);
    flex: 1;
    overflow: hidden;
    line-height: 1.4;
  }
  .card-content :global(p) { margin: 0 0 4px; }
  .card-content :global(p:last-child) { margin-bottom: 0; }
  .card-content :global(h1) { font-family: var(--font-heading); font-size: 17px; font-weight: 700; margin: 4px 0 4px; color: var(--color-fg-primary); }
  .card-content :global(h2) { font-family: var(--font-heading); font-size: 15px; font-weight: 700; margin: 4px 0 4px; color: var(--color-fg-primary); }
  .card-content :global(h3) { font-family: var(--font-heading); font-size: 13px; font-weight: 700; margin: 4px 0 4px; color: var(--color-fg-primary); text-transform: uppercase; letter-spacing: 0.5px; }
  .card-content :global(ul),
  .card-content :global(ol) { margin: 0 0 4px; padding-left: 18px; }
  .card-content :global(a) { color: var(--color-accent-gold); text-decoration: none; border-bottom: 1px dotted currentColor; cursor: pointer; }
  .card-content :global(a:hover) { border-bottom-style: solid; }
  .card-content :global(code) { font-family: var(--font-mono, monospace); font-size: 12px; padding: 1px 4px; background: var(--color-surface-tertiary); border-radius: 3px; }
  .card-placeholder { color: var(--color-fg-tertiary); opacity: 0.6; font-style: italic; }

  .card-actions {
    display: flex; justify-content: space-between; padding: 4px 8px;
    opacity: 0; transition: opacity 0.1s; border-top: 1px solid var(--color-border-subtle);
  }
  .card-connect, .card-delete {
    background: none; border: none; color: var(--color-fg-tertiary); cursor: pointer; font-size: 14px; padding: 2px 4px;
  }
  .card-connect:hover { color: var(--color-accent-gold); }
  .card-delete:hover { color: var(--color-status-error); }

  .card-resize {
    position: absolute;
    right: 0;
    bottom: 0;
    width: 14px;
    height: 14px;
    cursor: nwse-resize;
    opacity: 0;
    transition: opacity 0.1s;
    background: linear-gradient(135deg, transparent 50%, var(--color-fg-tertiary) 50%);
    border-bottom-right-radius: var(--radius-md);
  }

  .empty-overlay { position: absolute; inset: 0; display: flex; align-items: center; justify-content: center; color: var(--color-fg-tertiary); font-family: var(--font-ui); pointer-events: none; }
</style>
