<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { forceSimulation, forceLink, forceManyBody, forceCenter, forceCollide } from 'd3-force'
  import { loadPage, pageTree } from '../stores/pageStore'
  import { entityTypeMap, entityTypes } from '../stores/entityTypeStore'
  import { listAllRelations } from '../api/relations'
  import { listRelationTypes } from '../api/relations'
  import type { Relation, RelationType } from '../api/relations'
  import type { PageTreeNode } from '../api/pages'

  interface Props {
    onClose: () => void
  }

  let { onClose }: Props = $props()

  let canvas: HTMLCanvasElement
  let width = $state(800)
  let height = $state(600)
  let nodes = $state<any[]>([])
  let links = $state<any[]>([])
  let relTypes = $state<RelationType[]>([])
  let simulation: any = null
  let transform = $state({ x: 0, y: 0, scale: 1 })
  let dragging = $state<any>(null)
  let panning = $state(false)
  let panStart = { x: 0, y: 0 }
  let hoveredNode = $state<any>(null)
  let filterTypeId = $state<string>('')
  let connecting = $state<{ source: any; mouseX: number; mouseY: number } | null>(null)
  let connectPickerOpen = $state(false)
  let connectTarget = $state<any>(null)
  let connectSource = $state<any>(null)

  onMount(async () => {
    await loadGraphData()
    resizeCanvas()
    window.addEventListener('resize', resizeCanvas)
  })

  onDestroy(() => {
    simulation?.stop()
    window.removeEventListener('resize', resizeCanvas)
  })

  function resizeCanvas() {
    if (!canvas) return
    const rect = canvas.parentElement?.getBoundingClientRect()
    if (rect) {
      width = rect.width
      height = rect.height
      canvas.width = width
      canvas.height = height
      draw()
    }
  }

  async function loadGraphData() {
    const [allRelations, allRelTypes] = await Promise.all([
      listAllRelations(),
      listRelationTypes(),
    ])
    relTypes = allRelTypes

    const relTypeMap = new Map(allRelTypes.map((rt) => [rt.id, rt]))
    const tree = $pageTree

    // Filter pages by entity type if filter is active
    const filteredPages = filterTypeId
      ? tree.filter((p) => p.entity_type_id === filterTypeId)
      : tree

    const pageIds = new Set(filteredPages.map((p) => p.id))

    // Build nodes
    nodes = filteredPages.map((p) => ({
      id: p.id,
      title: p.title,
      entityTypeId: p.entity_type_id,
      color: p.entity_type_id ? ($entityTypeMap.get(p.entity_type_id)?.color || '#B8A690') : '#B8A690',
      x: Math.random() * width,
      y: Math.random() * height,
    }))

    // Build links (only between visible nodes)
    links = allRelations
      .filter((r) => pageIds.has(r.source_page_id) && pageIds.has(r.target_page_id))
      .map((r) => ({
        source: r.source_page_id,
        target: r.target_page_id,
        color: relTypeMap.get(r.relation_type_id)?.color || '#4A3F36',
        label: relTypeMap.get(r.relation_type_id)?.name || '',
      }))

    startSimulation()
  }

  function startSimulation() {
    simulation?.stop()

    simulation = forceSimulation(nodes)
      .force('link', forceLink(links).id((d: any) => d.id).distance(120))
      .force('charge', forceManyBody().strength(-200))
      .force('center', forceCenter(width / 2, height / 2))
      .force('collide', forceCollide(40))
      .on('tick', draw)
  }

  function draw() {
    if (!canvas) return
    const ctx = canvas.getContext('2d')
    if (!ctx) return

    ctx.clearRect(0, 0, width, height)
    ctx.save()
    ctx.translate(transform.x, transform.y)
    ctx.scale(transform.scale, transform.scale)

    // Draw edges
    for (const link of links) {
      const source = typeof link.source === 'object' ? link.source : nodes.find((n) => n.id === link.source)
      const target = typeof link.target === 'object' ? link.target : nodes.find((n) => n.id === link.target)
      if (!source || !target) continue

      ctx.beginPath()
      ctx.moveTo(source.x, source.y)
      ctx.lineTo(target.x, target.y)
      ctx.strokeStyle = link.color
      ctx.lineWidth = 1.5
      ctx.globalAlpha = 0.5
      ctx.stroke()
      ctx.globalAlpha = 1
    }

    // Draw nodes
    for (const node of nodes) {
      const isHovered = hoveredNode?.id === node.id
      const radius = isHovered ? 22 : 18

      // Node circle
      ctx.beginPath()
      ctx.arc(node.x, node.y, radius, 0, Math.PI * 2)
      ctx.fillStyle = '#332B25'
      ctx.fill()
      ctx.strokeStyle = isHovered ? '#C8A55C' : node.color
      ctx.lineWidth = isHovered ? 3 : 2
      ctx.stroke()

      // Node label
      ctx.fillStyle = '#E8DFD0'
      ctx.font = '11px Inter, system-ui, sans-serif'
      ctx.textAlign = 'center'
      ctx.textBaseline = 'top'
      ctx.fillText(node.title, node.x, node.y + radius + 4, 120)
    }

    // Draw connecting line if dragging to create relation
    if (connecting) {
      ctx.beginPath()
      ctx.moveTo(connecting.source.x, connecting.source.y)
      ctx.lineTo(connecting.mouseX, connecting.mouseY)
      ctx.strokeStyle = '#C8A55C'
      ctx.lineWidth = 2
      ctx.setLineDash([6, 4])
      ctx.stroke()
      ctx.setLineDash([])
    }

    ctx.restore()
  }

  function getNodeAt(x: number, y: number): any {
    const tx = (x - transform.x) / transform.scale
    const ty = (y - transform.y) / transform.scale
    for (const node of nodes) {
      const dx = tx - node.x
      const dy = ty - node.y
      if (dx * dx + dy * dy < 20 * 20) return node
    }
    return null
  }

  function handleMouseDown(e: MouseEvent) {
    const node = getNodeAt(e.offsetX, e.offsetY)
    if (node && e.shiftKey) {
      // Shift+drag from a node = start connecting
      connecting = { source: node, mouseX: (e.offsetX - transform.x) / transform.scale, mouseY: (e.offsetY - transform.y) / transform.scale }
    } else if (node) {
      dragging = node
      node.fx = node.x
      node.fy = node.y
      simulation?.alphaTarget(0.3).restart()
    } else {
      panning = true
      panStart = { x: e.offsetX - transform.x, y: e.offsetY - transform.y }
    }
  }

  function handleMouseMove(e: MouseEvent) {
    if (connecting) {
      connecting = { ...connecting, mouseX: (e.offsetX - transform.x) / transform.scale, mouseY: (e.offsetY - transform.y) / transform.scale }
      draw()
      return
    }
    if (dragging) {
      dragging.fx = (e.offsetX - transform.x) / transform.scale
      dragging.fy = (e.offsetY - transform.y) / transform.scale
    } else if (panning) {
      transform = { ...transform, x: e.offsetX - panStart.x, y: e.offsetY - panStart.y }
      draw()
    } else {
      const node = getNodeAt(e.offsetX, e.offsetY)
      if (node !== hoveredNode) {
        hoveredNode = node
        canvas.style.cursor = node ? 'pointer' : 'grab'
        draw()
      }
    }
  }

  function handleMouseUp(e: MouseEvent) {
    if (connecting) {
      const target = getNodeAt(e.offsetX, e.offsetY)
      if (target && target.id !== connecting.source.id) {
        connectSource = connecting.source
        connectTarget = target
        connectPickerOpen = true
      }
      connecting = null
      draw()
      return
    }
    if (dragging) {
      dragging.fx = null
      dragging.fy = null
      simulation?.alphaTarget(0)
      dragging = null
    }
    panning = false
  }

  let connectRelTypeId = $state('')

  async function confirmConnection() {
    if (!connectSource || !connectTarget) return
    const sourceId = connectSource.id
    const targetId = connectTarget.id
    if (!sourceId || !targetId || !connectRelTypeId) return

    const { createRelation } = await import('../api/relations')
    await createRelation(sourceId, targetId, connectRelTypeId)

    connectPickerOpen = false
    connectTarget = null
    connectRelTypeId = ''
    await loadGraphData()
  }

  function cancelConnection() {
    connectPickerOpen = false
    connectTarget = null
    connectRelTypeId = ''
  }

  function handleDblClick(e: MouseEvent) {
    const node = getNodeAt(e.offsetX, e.offsetY)
    if (node) {
      loadPage(node.id)
      window.dispatchEvent(new CustomEvent('vaelorium:page-selected'))
    }
  }

  function handleWheel(e: WheelEvent) {
    e.preventDefault()
    const factor = e.deltaY > 0 ? 0.9 : 1.1
    const newScale = Math.max(0.1, Math.min(5, transform.scale * factor))
    const mx = e.offsetX
    const my = e.offsetY
    transform = {
      scale: newScale,
      x: mx - (mx - transform.x) * (newScale / transform.scale),
      y: my - (my - transform.y) * (newScale / transform.scale),
    }
    draw()
  }

  function fitAll() {
    if (nodes.length === 0) return
    const minX = Math.min(...nodes.map((n) => n.x)) - 50
    const maxX = Math.max(...nodes.map((n) => n.x)) + 50
    const minY = Math.min(...nodes.map((n) => n.y)) - 50
    const maxY = Math.max(...nodes.map((n) => n.y)) + 50
    const gw = maxX - minX
    const gh = maxY - minY
    const scale = Math.min(width / gw, height / gh, 2)
    transform = {
      scale,
      x: (width - gw * scale) / 2 - minX * scale,
      y: (height - gh * scale) / 2 - minY * scale,
    }
    draw()
  }

  $effect(() => {
    // Re-filter when filterTypeId changes
    if (filterTypeId !== undefined) {
      loadGraphData()
    }
  })
</script>

<div class="graph-view">
  <header class="graph-header">
    <div class="header-left">
      <button class="back-btn" onclick={onClose} aria-label="Back to editor">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="15 18 9 12 15 6"></polyline>
        </svg>
      </button>
      <h2 class="header-title">Relations Graph</h2>
      <span class="header-count">{nodes.length} nodes · {links.length} edges</span>
    </div>
    <div class="header-right">
      <select class="filter-select" bind:value={filterTypeId}>
        <option value="">All types</option>
        {#each $entityTypes as type (type.id)}
          <option value={type.id}>{type.name}</option>
        {/each}
      </select>
      <span class="hint">Shift+drag to connect</span>
      <button class="tool-btn" onclick={fitAll} title="Fit all">Fit</button>
    </div>
  </header>

  <div class="canvas-container">
    <canvas
      bind:this={canvas}
      onmousedown={handleMouseDown}
      onmousemove={handleMouseMove}
      onmouseup={handleMouseUp}
      onmouseleave={handleMouseUp}
      ondblclick={handleDblClick}
      onwheel={handleWheel}
    ></canvas>

    {#if connectPickerOpen}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="connect-overlay" onclick={cancelConnection}>
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="connect-picker" onclick={(e) => e.stopPropagation()}>
          <h3 class="connect-title">
            Connect: {connectSource?.title} → {connectTarget?.title}
          </h3>
          <select class="connect-select" bind:value={connectRelTypeId}>
            <option value="">Select relation type...</option>
            {#each relTypes as rt (rt.id)}
              <option value={rt.id}>{rt.name}</option>
            {/each}
          </select>
          <div class="connect-actions">
            <button class="connect-cancel" onclick={cancelConnection}>Cancel</button>
            <button class="connect-confirm" disabled={!connectRelTypeId} onclick={confirmConnection}>Connect</button>
          </div>
        </div>
      </div>
    {/if}

    {#if nodes.length === 0}
      <div class="empty-overlay">
        <p>No entities to display</p>
        <p class="empty-hint">Create pages and add relations to see the graph</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .graph-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    background: var(--color-surface-primary);
  }

  .graph-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 20px;
    background: var(--color-surface-secondary);
    border-bottom: 1px solid var(--color-border-subtle);
    flex-shrink: 0;
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

  .header-title {
    font-family: var(--font-heading);
    font-size: 18px;
    font-weight: 600;
    color: var(--color-fg-primary);
    margin: 0;
  }

  .header-count {
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-fg-tertiary);
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .filter-select {
    padding: 4px 10px;
    background: var(--color-surface-primary);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-fg-primary);
  }

  .tool-btn {
    padding: 4px 12px;
    background: var(--color-surface-tertiary);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-fg-secondary);
    cursor: pointer;
  }

  .tool-btn:hover {
    border-color: var(--color-accent-gold);
  }

  .canvas-container {
    flex: 1;
    position: relative;
    overflow: hidden;
  }

  canvas {
    display: block;
    width: 100%;
    height: 100%;
    cursor: grab;
  }

  .empty-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--color-fg-tertiary);
    font-family: var(--font-ui);
    gap: 4px;
  }

  .hint {
    font-family: var(--font-ui);
    font-size: 11px;
    color: var(--color-fg-tertiary);
    opacity: 0.6;
  }

  .connect-overlay {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10;
  }

  .connect-picker {
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-lg);
    padding: 20px;
    width: 320px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  }

  .connect-title {
    font-family: var(--font-heading);
    font-size: 15px;
    font-weight: 600;
    color: var(--color-fg-primary);
    margin: 0;
  }

  .connect-select {
    padding: 6px 10px;
    background: var(--color-surface-primary);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-primary);
  }

  .connect-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .connect-cancel {
    padding: 6px 14px;
    background: var(--color-surface-tertiary);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-fg-secondary);
    cursor: pointer;
  }

  .connect-confirm {
    padding: 6px 14px;
    background: var(--color-accent-gold);
    border: none;
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 12px;
    font-weight: 600;
    color: var(--color-fg-inverse);
    cursor: pointer;
  }

  .connect-confirm:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .empty-hint {
    font-size: 12px;
    opacity: 0.6;
  }
</style>
