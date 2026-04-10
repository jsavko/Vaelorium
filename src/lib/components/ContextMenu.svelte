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

  function handleClick(item: MenuItem) {
    item.action()
    onClose()
  }
</script>

<svelte:window onclick={onClose} />

<div class="context-menu" style:left="{x}px" style:top="{y}px" onclick={(e) => e.stopPropagation()}>
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
