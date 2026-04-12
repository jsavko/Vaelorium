<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { setMentionMenuRenderer } from '../editor/MentionExtension'
  import type { MentionItem } from '../editor/MentionExtension'
  import { entityTypeMap } from '../stores/entityTypeStore'

  let visible = $state(false)
  let items = $state<MentionItem[]>([])
  let selectedIndex = $state(0)
  let commandFn: ((item: MentionItem) => void) | null = null
  let menuX = $state(0)
  let menuY = $state(0)

  function getEntityColor(typeId: string | null): string {
    if (!typeId) return 'var(--color-fg-tertiary)'
    const type = $entityTypeMap.get(typeId)
    return type?.color || 'var(--color-fg-tertiary)'
  }

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
    setMentionMenuRenderer({
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
  <div class="mention-dropdown" style:left="{menuX}px" style:top="{menuY}px" data-testid="mention-menu">
    <div class="mention-header">LINK TO</div>
    {#each items as item, index (item.id + item.category)}
      <button
        class="mention-item"
        class:selected={index === selectedIndex}
        onmousedown={(e) => e.preventDefault()}
        onclick={() => executeCommand(index)}
      >
        <span
          class="mention-dot"
          style:background-color={item.category === 'page' ? getEntityColor(item.entity_type_id || null) : item.color || 'var(--color-fg-tertiary)'}
        ></span>
        <span class="mention-title">{item.title}</span>
        <span class="mention-spacer"></span>
        <span class="mention-type">
          {item.category === 'map' ? '📍 Map' : item.category === 'timeline' ? '📅 Timeline' : ''}
        </span>
      </button>
    {/each}
  </div>
{/if}

<style>
  .mention-dropdown {
    position: fixed;
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-md);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
    padding: 6px;
    min-width: 280px;
    max-height: 300px;
    overflow-y: auto;
    z-index: 50;
  }

  .mention-header {
    font-family: var(--font-ui);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 2px;
    color: var(--color-fg-tertiary);
    padding: 6px 10px;
  }

  .mention-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    border: none;
    background: transparent;
    cursor: pointer;
    width: 100%;
    text-align: left;
    font-family: var(--font-ui);
  }

  .mention-item:hover,
  .mention-item.selected {
    background: var(--color-accent-gold-subtle);
  }

  .mention-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .mention-title {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-fg-primary);
  }

  .mention-spacer {
    flex: 1;
  }

  .mention-type {
    font-size: 11px;
    color: var(--color-fg-tertiary);
    text-transform: capitalize;
  }
</style>
