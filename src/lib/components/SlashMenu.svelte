<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { setSlashMenuRenderer, type SlashCommandItem } from '../editor/SlashCommands'

  let visible = $state(false)
  let items = $state<SlashCommandItem[]>([])
  let selectedIndex = $state(0)
  let commandFn: ((item: SlashCommandItem) => void) | null = null
  let menuX = $state(0)
  let menuY = $state(0)

  function executeCommand(index: number) {
    const item = items[index]
    if (item && commandFn) {
      commandFn(item)
    }
    visible = false
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!visible) return
    if (e.key === 'ArrowDown') {
      e.preventDefault()
      selectedIndex = (selectedIndex + 1) % items.length
    } else if (e.key === 'ArrowUp') {
      e.preventDefault()
      selectedIndex = (selectedIndex - 1 + items.length) % items.length
    } else if (e.key === 'Enter') {
      e.preventDefault()
      executeCommand(selectedIndex)
    }
  }

  onMount(() => {
    setSlashMenuRenderer({
      onStart(props) {
        items = props.items
        commandFn = props.command
        selectedIndex = 0
        if (props.clientRect) {
          const rect = props.clientRect()
          if (rect) {
            menuX = rect.left
            menuY = rect.bottom + 4
          }
        }
        visible = true
      },
      onUpdate(props) {
        items = props.items
        commandFn = props.command
        selectedIndex = 0
        if (props.clientRect) {
          const rect = props.clientRect()
          if (rect) {
            menuX = rect.left
            menuY = rect.bottom + 4
          }
        }
      },
      onExit() {
        visible = false
      },
    })

    window.addEventListener('keydown', handleKeydown)
  })

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown)
  })
</script>

{#if visible && items.length > 0}
  <div class="slash-menu" style:left="{menuX}px" style:top="{menuY}px" data-testid="slash-menu">
    {#each items as cmd, index}
      <button
        class="slash-item"
        class:selected={index === selectedIndex}
        onmousedown={(e) => e.preventDefault()}
        onclick={() => executeCommand(index)}
      >
        <span class="slash-icon">{cmd.icon}</span>
        <span class="slash-label">{cmd.label}</span>
      </button>
    {/each}
  </div>
{/if}

<style>
  .slash-menu {
    position: fixed;
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-md);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
    padding: 4px;
    min-width: 200px;
    max-height: 300px;
    overflow-y: auto;
    z-index: 50;
  }

  .slash-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    border: none;
    background: transparent;
    border-radius: var(--radius-sm);
    cursor: pointer;
    width: 100%;
    text-align: left;
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-primary);
  }

  .slash-item:hover,
  .slash-item.selected {
    background: var(--color-accent-gold-subtle);
  }

  .slash-icon {
    width: 24px;
    text-align: center;
    font-weight: 600;
    color: var(--color-fg-tertiary);
  }
</style>
