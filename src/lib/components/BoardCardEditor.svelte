<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { Editor } from '@tiptap/core'
  import StarterKit from '@tiptap/starter-kit'
  import Link from '@tiptap/extension-link'
  import Placeholder from '@tiptap/extension-placeholder'
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
    // Cmd/Ctrl+Enter saves.
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault()
      save()
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="card-editor"
  bind:this={element}
  onkeydown={handleKeyDown}
  onblur={save}
  onclick={(e) => e.stopPropagation()}
  onmousedown={(e) => e.stopPropagation()}
></div>

<style>
  .card-editor {
    width: 100%;
    min-height: 60px;
    padding: 10px 12px;
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-primary);
    outline: none;
  }

  .card-editor :global(.ProseMirror) {
    outline: none;
    min-height: 50px;
  }

  .card-editor :global(.ProseMirror p) {
    margin: 0 0 4px;
  }

  .card-editor :global(.ProseMirror p:last-child) {
    margin-bottom: 0;
  }

  .card-editor :global(.ProseMirror ul),
  .card-editor :global(.ProseMirror ol) {
    margin: 0 0 4px;
    padding-left: 18px;
  }

  .card-editor :global(.ProseMirror code) {
    font-family: var(--font-mono, monospace);
    font-size: 12px;
    padding: 1px 4px;
    background: var(--color-surface-tertiary);
    border-radius: 3px;
  }

  .card-editor :global(.ProseMirror a) {
    color: var(--color-accent-gold);
    text-decoration: underline;
  }

  .card-editor :global(.ProseMirror p.is-editor-empty:first-child::before) {
    content: attr(data-placeholder);
    float: left;
    color: var(--color-fg-tertiary);
    pointer-events: none;
    height: 0;
  }
</style>
