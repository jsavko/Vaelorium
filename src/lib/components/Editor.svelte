<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { Editor } from '@tiptap/core'
  import { createEditorExtensions } from '../editor/EditorConfig'
  import { LocalYjsProvider } from '../editor/YjsProvider'
  import { currentPage, updateCurrentPage, loadPage, pageTree } from '../stores/pageStore'
  import { get } from 'svelte/store'
  import type { Page } from '../api/pages'

  let editorElement: HTMLDivElement
  let editor: Editor | null = null
  let provider: LocalYjsProvider | null = null
  let titleInput: HTMLInputElement
  let currentLoadedPageId: string | null = null

  // React to page changes
  $effect(() => {
    const page = $currentPage
    if (page && page.id !== currentLoadedPageId) {
      loadEditor(page)
    }
  })

  async function loadEditor(page: Page) {
    // Destroy previous editor
    if (editor) {
      editor.destroy()
      editor = null
    }
    if (provider) {
      await provider.destroy()
      provider = null
    }

    currentLoadedPageId = page.id

    // Create new Yjs provider and load content
    provider = new LocalYjsProvider(page.id)
    await provider.load()

    // Create editor
    editor = new Editor({
      element: editorElement,
      extensions: createEditorExtensions(provider.doc),
      editorProps: {
        attributes: {
          class: 'editor-content',
        },
      },
    })
  }

  async function handleTitleBlur() {
    if (!titleInput || !$currentPage) return
    const newTitle = titleInput.value.trim()
    if (newTitle && newTitle !== $currentPage.title) {
      await updateCurrentPage({ title: newTitle })
    }
  }

  function handleTitleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault()
      titleInput.blur()
      editor?.commands.focus()
    }
  }

  function handleEditorClick(e: MouseEvent) {
    const target = e.target as HTMLElement
    const link = target.closest('a[href^="#page:"]')
    if (link) {
      e.preventDefault()
      const href = link.getAttribute('href')
      if (href) {
        const pageId = href.replace('#page:', '')
        loadPage(pageId)
      }
    }
  }

  function handleEditorDrop(e: DragEvent) {
    const pageId = e.dataTransfer?.getData('text/plain')
    if (!pageId || !editor) return

    if (pageId.match(/^[0-9a-f-]{36}$/)) {
      e.preventDefault()
      const tree = get(pageTree)
      const page = tree.find((p) => p.id === pageId)
      const title = page?.title || 'Linked Page'
      editor.chain().focus().insertContent({
        type: 'text',
        marks: [{
          type: 'link',
          attrs: { href: `#page:${pageId}`, class: 'wiki-link-inline' },
        }],
        text: title,
      }).run()
    }
  }

  onDestroy(async () => {
    if (editor) editor.destroy()
    if (provider) await provider.destroy()
  })
</script>

{#if $currentPage}
  <div class="editor-wrapper">
    <div class="page-header">
      {#if $currentPage.entity_type_id}
        <span class="entity-badge">
          {$currentPage.entity_type_id}
        </span>
      {/if}

      <input
        bind:this={titleInput}
        class="page-title"
        value={$currentPage.title}
        onblur={handleTitleBlur}
        onkeydown={handleTitleKeydown}
        placeholder="Untitled"
      />

      <div class="page-meta">
        <span class="meta-text">Last edited {new Date($currentPage.updated_at).toLocaleDateString()}</span>
      </div>
    </div>

    <div class="editor-toolbar">
      <button class="toolbar-btn" onmousedown={(e) => e.preventDefault()} onclick={() => editor?.chain().focus().toggleBold().run()}>
        <strong>B</strong>
      </button>
      <button class="toolbar-btn" onmousedown={(e) => e.preventDefault()} onclick={() => editor?.chain().focus().toggleItalic().run()}>
        <em>I</em>
      </button>
      <span class="toolbar-sep"></span>
      <button class="toolbar-btn" onmousedown={(e) => e.preventDefault()} onclick={() => editor?.chain().focus().toggleHeading({ level: 1 }).run()}>
        H1
      </button>
      <button class="toolbar-btn" onmousedown={(e) => e.preventDefault()} onclick={() => editor?.chain().focus().toggleHeading({ level: 2 }).run()}>
        H2
      </button>
      <button class="toolbar-btn" onmousedown={(e) => e.preventDefault()} onclick={() => editor?.chain().focus().toggleHeading({ level: 3 }).run()}>
        H3
      </button>
      <span class="toolbar-sep"></span>
      <button class="toolbar-btn" onmousedown={(e) => e.preventDefault()} onclick={() => editor?.chain().focus().toggleBulletList().run()}>
        List
      </button>
    </div>

    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div
      class="editor-container"
      bind:this={editorElement}
      onclick={handleEditorClick}
      ondrop={handleEditorDrop}
      ondragover={(e) => e.preventDefault()}
    ></div>
  </div>
{:else}
  <div class="welcome">
    <h1 class="welcome-title">Welcome to Vaelorium</h1>
    <p class="welcome-subtitle">The Arcane Library awaits. Create your first page to begin.</p>
  </div>
{/if}

<style>
  .editor-wrapper {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    width: 100%;
    padding: 40px 60px;
  }

  .page-header {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 24px;
  }

  .entity-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    font-family: var(--font-ui);
    font-size: 12px;
    font-weight: 500;
    border-radius: 20px;
    background: var(--color-surface-tertiary);
    color: var(--color-fg-tertiary);
    width: fit-content;
  }

  .page-title {
    font-family: var(--font-heading);
    font-size: 36px;
    font-weight: 700;
    color: var(--color-fg-primary);
    background: none;
    border: none;
    outline: none;
    padding: 0;
    width: 100%;
  }

  .page-title::placeholder {
    color: var(--color-fg-tertiary);
  }

  .page-meta {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .meta-text {
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-fg-tertiary);
  }

  .editor-toolbar {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 8px 12px;
    background: var(--color-surface-card);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border-subtle);
    margin-bottom: 24px;
  }

  .toolbar-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 6px 8px;
    border: none;
    background: transparent;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-secondary);
  }

  .toolbar-btn:hover {
    background: var(--color-surface-tertiary);
  }

  .toolbar-sep {
    width: 1px;
    height: 20px;
    background: var(--color-border-default);
    margin: 0 4px;
  }

  .editor-container {
    flex: 1;
    overflow-y: auto;
  }

  /* TipTap editor styling */
  .editor-container :global(.editor-content) {
    outline: none;
    font-family: var(--font-body);
    font-size: 16px;
    line-height: 1.7;
    color: var(--color-fg-secondary);
    min-height: 300px;
  }

  .editor-container :global(.editor-content h1) {
    font-family: var(--font-heading);
    font-size: 32px;
    font-weight: 700;
    color: var(--color-fg-primary);
    margin: 24px 0 12px;
  }

  .editor-container :global(.editor-content h2) {
    font-family: var(--font-heading);
    font-size: 24px;
    font-weight: 600;
    color: var(--color-fg-primary);
    margin: 20px 0 10px;
  }

  .editor-container :global(.editor-content h3) {
    font-family: var(--font-heading);
    font-size: 18px;
    font-weight: 600;
    color: var(--color-fg-primary);
    margin: 16px 0 8px;
  }

  .editor-container :global(.editor-content p) {
    margin: 0 0 12px;
  }

  .editor-container :global(.editor-content a) {
    color: var(--color-accent-gold);
    text-decoration: none;
  }

  .editor-container :global(.editor-content a:hover) {
    text-decoration: underline;
  }

  .editor-container :global(.editor-content ul),
  .editor-container :global(.editor-content ol) {
    padding-left: 24px;
    margin: 0 0 12px;
  }

  .editor-container :global(.editor-content blockquote) {
    border-left: 3px solid var(--color-accent-gold);
    padding-left: 16px;
    margin: 12px 0;
    color: var(--color-fg-tertiary);
    font-style: italic;
  }

  .editor-container :global(.editor-content code) {
    background: var(--color-surface-tertiary);
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 14px;
  }

  .editor-container :global(.editor-content table) {
    border-collapse: collapse;
    width: 100%;
    margin: 12px 0;
  }

  .editor-container :global(.editor-content th),
  .editor-container :global(.editor-content td) {
    border: 1px solid var(--color-border-default);
    padding: 8px 12px;
    text-align: left;
  }

  .editor-container :global(.editor-content th) {
    background: var(--color-surface-tertiary);
    font-weight: 600;
  }

  .editor-container :global(.editor-content .is-empty::before) {
    content: attr(data-placeholder);
    float: left;
    color: var(--color-fg-tertiary);
    pointer-events: none;
    height: 0;
  }

  .welcome {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    text-align: center;
  }

  .welcome-title {
    font-family: var(--font-heading);
    font-size: 40px;
    font-weight: 700;
    color: var(--color-fg-primary);
    margin-bottom: 12px;
  }

  .welcome-subtitle {
    font-family: var(--font-body);
    font-size: 18px;
    color: var(--color-fg-secondary);
  }
</style>
