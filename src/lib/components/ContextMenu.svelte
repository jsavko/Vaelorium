<script lang="ts">
  interface MenuItem {
    label: string
    action: () => void
    danger?: boolean
  }

  interface Props {
    x: number
    y: number
    items: MenuItem[]
    onClose: () => void
  }

  let { x, y, items, onClose }: Props = $props()

  // Clamp the menu inside the viewport. Right-clicking near the right
  // or bottom edge used to push the menu off-screen with items
  // unreachable. `measured` is the post-layout clamped position; the
  // template falls back to the raw click coords for the initial frame
  // so the menu appears immediately (effects run after DOM commit).
  let menuEl: HTMLDivElement | undefined = $state()
  let measured = $state<{ x: number; y: number } | null>(null)
  let adjX = $derived(measured?.x ?? x)
  let adjY = $derived(measured?.y ?? y)
  const EDGE_MARGIN = 6

  $effect(() => {
    // Re-measure whenever the incoming click coords change. Svelte 5
    // effects run after the DOM is updated, so the menu element is
    // laid out by the time we read it.
    if (!menuEl) return
    const rect = menuEl.getBoundingClientRect()
    const vw = window.innerWidth
    const vh = window.innerHeight
    const nx = Math.max(EDGE_MARGIN, Math.min(x, vw - rect.width - EDGE_MARGIN))
    const ny = Math.max(EDGE_MARGIN, Math.min(y, vh - rect.height - EDGE_MARGIN))
    measured = { x: nx, y: ny }
  })

  function handleClick(item: MenuItem) {
    item.action()
    onClose()
  }
</script>

<svelte:window onclick={onClose} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  bind:this={menuEl}
  class="context-menu"
  style:left="{adjX}px"
  style:top="{adjY}px"
  onclick={(e) => e.stopPropagation()}
>
  {#each items as item}
    <button
      class="menu-item"
      class:danger={item.danger}
      onclick={() => handleClick(item)}
    >
      {item.label}
    </button>
  {/each}
</div>

<style>
  .context-menu {
    position: fixed;
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-md);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
    padding: 4px;
    min-width: 160px;
    z-index: 200;
  }

  .menu-item {
    display: block;
    width: 100%;
    padding: 8px 12px;
    border: none;
    background: transparent;
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-primary);
    text-align: left;
    cursor: pointer;
    border-radius: var(--radius-sm);
  }

  .menu-item:hover {
    background: var(--color-surface-tertiary);
  }

  .menu-item.danger {
    color: var(--color-status-error);
  }

  .menu-item.danger:hover {
    background: rgba(184, 92, 92, 0.15);
  }
</style>
