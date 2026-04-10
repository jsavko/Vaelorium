<script lang="ts">
  import type { Editor } from '@tiptap/core'

  interface Props {
    editor: Editor
    onClose: () => void
  }

  let { editor, onClose }: Props = $props()
  let selectedIndex = $state(0)

  const commands = [
    { label: 'Heading 1', icon: 'H1', action: () => editor.chain().focus().toggleHeading({ level: 1 }).run() },
    { label: 'Heading 2', icon: 'H2', action: () => editor.chain().focus().toggleHeading({ level: 2 }).run() },
    { label: 'Heading 3', icon: 'H3', action: () => editor.chain().focus().toggleHeading({ level: 3 }).run() },
    { label: 'Bullet List', icon: '•', action: () => editor.chain().focus().toggleBulletList().run() },
    { label: 'Ordered List', icon: '1.', action: () => editor.chain().focus().toggleOrderedList().run() },
    { label: 'Blockquote', icon: '"', action: () => editor.chain().focus().toggleBlockquote().run() },
    { label: 'Code Block', icon: '<>', action: () => editor.chain().focus().toggleCodeBlock().run() },
    { label: 'Divider', icon: '—', action: () => editor.chain().focus().setHorizontalRule().run() },
    { label: 'Table', icon: '▦', action: () => editor.chain().focus().insertTable({ rows: 3, cols: 3 }).run() },
  ]

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault()
      selectedIndex = (selectedIndex + 1) % commands.length
    } else if (e.key === 'ArrowUp') {
      e.preventDefault()
      selectedIndex = (selectedIndex - 1 + commands.length) % commands.length
    } else if (e.key === 'Enter') {
      e.preventDefault()
      executeCommand(selectedIndex)
    } else if (e.key === 'Escape') {
      onClose()
    }
  }

  function executeCommand(index: number) {
    commands[index].action()
    onClose()
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="slash-menu">
  {#each commands as cmd, index}
    <button
      class="slash-item"
      class:selected={index === selectedIndex}
      onclick={() => executeCommand(index)}
    >
      <span class="slash-icon">{cmd.icon}</span>
      <span class="slash-label">{cmd.label}</span>
    </button>
  {/each}
</div>

<style>
  .slash-menu {
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-md);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
    padding: 4px;
    min-width: 200px;
    max-height: 300px;
    overflow-y: auto;
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
