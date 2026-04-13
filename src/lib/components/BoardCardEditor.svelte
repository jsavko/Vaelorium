<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { Editor } from '@tiptap/core'
  import StarterKit from '@tiptap/starter-kit'
  import Link from '@tiptap/extension-link'
  import Placeholder from '@tiptap/extension-placeholder'
  import { Bold, Italic, Strikethrough, Code as CodeIcon, List, ListOrdered, Link as LinkIcon, AtSign } from 'lucide-svelte'
  import { MentionExtension } from '../editor/MentionExtension'

  interface Props {
    initialHtml: string
    onSave: (html: string) => void
    onCancel?: () => void
    placeholder?: string
  }

  let { initialHtml, onSave, onCancel, placeholder = 'Write notes, or @mention a page...' }: Props = $props()

  let element: HTMLDivElement
  let editor: Editor | undefined
  // Bumped on every tx so toolbar active states reflect the cursor.
  let tick = $state(0)
  // Suppress the "blur = save" behavior while the user is clicking a
  // toolbar button or the mention suggestion menu (either briefly moves
  // focus out of ProseMirror).
  let suppressBlur = $state(false)

  onMount(() => {
    editor = new Editor({
      element,
      extensions: [
        // Tight set for card-scale content: paragraphs, lists, marks,
        // hard breaks, history. Deliberately drops headings, code blocks,
        // blockquotes, horizontal rules, tables — overkill for sticky notes.
        StarterKit.configure({
          heading: false,
          codeBlock: false,
          blockquote: false,
          horizontalRule: false,
        }),
        Link.configure({ openOnClick: false, autolink: true }),
        Placeholder.configure({ placeholder }),
        MentionExtension,
      ],
      content: initialHtml || '',
      autofocus: 'end',
      onUpdate: () => { tick++ },
      onSelectionUpdate: () => { tick++ },
      onBlur: () => {
        // Defer so a toolbar-click that refocuses doesn't trigger a save.
        setTimeout(() => {
          if (!suppressBlur) save()
        }, 80)
      },
    })
  })

  onDestroy(() => {
    editor?.destroy()
  })

  export function save() {
    if (!editor) return
    const html = editor.getHTML()
    onSave(html)
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault()
      e.stopPropagation()
      if (onCancel) onCancel()
      else save()
    }
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault()
      save()
    }
  }

  // Toolbar buttons use onmousedown + preventDefault so they never pull
  // focus out of ProseMirror (same trick Editor.svelte uses at lines
  // 293+). The click handler then calls the editor command and refocuses.
  function toolbarBtn(e: MouseEvent) {
    suppressBlur = true
    e.preventDefault()
    e.stopPropagation()
    setTimeout(() => { suppressBlur = false }, 120)
  }

  function isActive(name: string, attrs?: Record<string, unknown>): boolean {
    // Re-compute when tick changes.
    void tick
    return editor?.isActive(name, attrs) ?? false
  }

  function insertMentionAt() {
    if (!editor) return
    editor.chain().focus().insertContent('@').run()
  }

  async function addLink() {
    if (!editor) return
    const url = prompt('Link URL')
    if (!url) return
    editor.chain().focus().extendMarkRange('link').setLink({ href: url }).run()
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="card-editor-wrap" onkeydown={handleKeyDown} onclick={(e) => e.stopPropagation()} onmousedown={(e) => e.stopPropagation()} role="group">
  <div class="toolbar">
    <button class="tbtn" class:active={isActive('bold')} onmousedown={toolbarBtn} onclick={() => editor?.chain().focus().toggleBold().run()} title="Bold (Ctrl+B)"><Bold size={13} /></button>
    <button class="tbtn" class:active={isActive('italic')} onmousedown={toolbarBtn} onclick={() => editor?.chain().focus().toggleItalic().run()} title="Italic (Ctrl+I)"><Italic size={13} /></button>
    <button class="tbtn" class:active={isActive('strike')} onmousedown={toolbarBtn} onclick={() => editor?.chain().focus().toggleStrike().run()} title="Strikethrough"><Strikethrough size={13} /></button>
    <button class="tbtn" class:active={isActive('code')} onmousedown={toolbarBtn} onclick={() => editor?.chain().focus().toggleCode().run()} title="Inline code"><CodeIcon size={13} /></button>
    <div class="sep"></div>
    <button class="tbtn" class:active={isActive('bulletList')} onmousedown={toolbarBtn} onclick={() => editor?.chain().focus().toggleBulletList().run()} title="Bulleted list"><List size={13} /></button>
    <button class="tbtn" class:active={isActive('orderedList')} onmousedown={toolbarBtn} onclick={() => editor?.chain().focus().toggleOrderedList().run()} title="Numbered list"><ListOrdered size={13} /></button>
    <div class="sep"></div>
    <button class="tbtn" class:active={isActive('link')} onmousedown={toolbarBtn} onclick={addLink} title="Link"><LinkIcon size={13} /></button>
    <button class="tbtn" onmousedown={toolbarBtn} onclick={insertMentionAt} title="Mention a page (@)"><AtSign size={13} /></button>
  </div>

  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="card-editor" bind:this={element}></div>

  <div class="editor-hint">Esc to save · Cmd/Ctrl+Enter to save · Markdown shortcuts (**bold**, *italic*, - bullet, 1. list) work</div>
</div>

<style>
  .card-editor-wrap {
    display: flex;
    flex-direction: column;
    width: 100%;
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 4px 6px;
    border-bottom: 1px solid var(--color-border-subtle);
    background: var(--color-surface-secondary);
    flex-wrap: wrap;
  }
  .tbtn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 4px 6px;
    background: none;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    color: var(--color-fg-tertiary);
    cursor: pointer;
  }
  .tbtn:hover { color: var(--color-fg-primary); background: var(--color-surface-tertiary); }
  .tbtn.active { color: var(--color-accent-gold); background: var(--color-surface-tertiary); }
  .sep { width: 1px; height: 14px; background: var(--color-border-subtle); margin: 0 4px; }

  .card-editor {
    width: 100%;
    min-height: 60px;
    padding: 10px 12px;
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-primary);
    outline: none;
  }
  .card-editor :global(.ProseMirror) { outline: none; min-height: 50px; }
  .card-editor :global(.ProseMirror p) { margin: 0 0 4px; }
  .card-editor :global(.ProseMirror p:last-child) { margin-bottom: 0; }
  .card-editor :global(.ProseMirror ul),
  .card-editor :global(.ProseMirror ol) { margin: 0 0 4px; padding-left: 18px; }
  .card-editor :global(.ProseMirror code) {
    font-family: var(--font-mono, monospace); font-size: 12px;
    padding: 1px 4px; background: var(--color-surface-tertiary); border-radius: 3px;
  }
  .card-editor :global(.ProseMirror a) { color: var(--color-accent-gold); text-decoration: underline; }
  .card-editor :global(.ProseMirror p.is-editor-empty:first-child::before) {
    content: attr(data-placeholder);
    float: left; color: var(--color-fg-tertiary); pointer-events: none; height: 0;
  }

  .editor-hint {
    padding: 4px 10px;
    font-family: var(--font-ui);
    font-size: 10px;
    color: var(--color-fg-tertiary);
    opacity: 0.7;
    border-top: 1px solid var(--color-border-subtle);
  }
</style>
