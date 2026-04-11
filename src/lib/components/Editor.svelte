<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { Editor } from '@tiptap/core'
  import { createEditorExtensions } from '../editor/EditorConfig'
  import { LocalYjsProvider } from '../editor/YjsProvider'
  import { currentPage, updateCurrentPage, loadPage, pageTree } from '../stores/pageStore'
  import { entityTypeMap } from '../stores/entityTypeStore'
  import IconPicker from './IconPicker.svelte'
  import { get } from 'svelte/store'
  import { pickAndUploadImage, getImageUrl } from '../api/images'
  import type { Page } from '../api/pages'

  let editorElement: HTMLDivElement
  let editor: Editor | null = null
  let provider: LocalYjsProvider | null = null
  let titleInput: HTMLInputElement
  let currentLoadedPageId: string | null = null
  let embedPickerOpen = $state(false)
  let embedPickerEditor: Editor | null = null

  // React to page changes
  $effect(() => {
    const page = $currentPage
    if (page && page.id !== currentLoadedPageId) {
      loadEditor(page)
    }
  })

  async function loadEditor(page: Page) {
    // Save and destroy previous editor
    if (provider) {
      await provider.save() // Save with callbacks still intact
      await provider.destroy()
      provider = null
    }
    if (editor) {
      editor.destroy()
      editor = null
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

    // Wire save callbacks for link extraction + search indexing
    provider.setCallbacks({
      getHTML: () => editor?.getHTML() || '',
      getPlainText: () => editor?.getText() || '',
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

  async function handleEditorDrop(e: DragEvent) {
    if (!editor) return

    // Check for dropped image files
    const files = e.dataTransfer?.files
    if (files && files.length > 0) {
      const file = files[0]
      if (file.type.startsWith('image/')) {
        e.preventDefault()
        const { uploadFileObject } = await import('../api/images')
        const info = await uploadFileObject(file)
        const url = await getImageUrl(info.id)
        editor.chain().focus().setImage({ src: url }).run()
        return
      }
    }

    // Check for page drag (wiki link)
    const pageId = e.dataTransfer?.getData('text/plain')
    if (!pageId) return

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

  // Listen for embed request from slash command
  function handleEmbedRequest(e: Event) {
    const detail = (e as CustomEvent).detail
    embedPickerEditor = detail.editor
    embedPickerOpen = true
  }

  // Listen for navigation from embedded page "Open" button
  function handleEmbedNavigate(e: Event) {
    const detail = (e as CustomEvent).detail
    loadPage(detail.pageId)
  }

  function selectPageForEmbed(pageId: string, pageTitle: string) {
    if (embedPickerEditor) {
      ;(embedPickerEditor.commands as any).insertPageEmbed({ pageId, pageTitle })
    }
    embedPickerOpen = false
    embedPickerEditor = null
  }

  $effect(() => {
    window.addEventListener('vaelorium:embed-request', handleEmbedRequest)
    window.addEventListener('vaelorium:navigate', handleEmbedNavigate)
    return () => {
      window.removeEventListener('vaelorium:embed-request', handleEmbedRequest)
      window.removeEventListener('vaelorium:navigate', handleEmbedNavigate)
    }
  })

  onDestroy(async () => {
    if (editor) editor.destroy()
    if (provider) await provider.destroy()
  })
</script>

{#if $currentPage}
  <div class="editor-wrapper">
    <div class="page-header">
      {#if $currentPage.entity_type_id}
        {@const entityType = $entityTypeMap.get($currentPage.entity_type_id)}
        {#if entityType}
          <span class="entity-badge" style:--badge-color={entityType.color || 'var(--color-fg-tertiary)'}>
            <span class="badge-dot"></span>
            {entityType.name}
          </span>
        {/if}
      {/if}

      <div class="title-row">
        <IconPicker
          currentIcon={$currentPage.icon}
          onSelect={(icon) => updateCurrentPage({ icon: icon || '' })}
        />
        <input
          bind:this={titleInput}
          class="page-title"
          value={$currentPage.title}
          onblur={handleTitleBlur}
          onkeydown={handleTitleKeydown}
          placeholder="Untitled"
        />
      </div>

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
      <button class="toolbar-btn" onmousedown={(e) => e.preventDefault()} onclick={() => editor?.chain().focus().toggleUnderline?.().run()}>
        <u>U</u>
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
      <button class="toolbar-btn" onmousedown={(e) => e.preventDefault()} onclick={() => { const url = prompt('Enter link URL:'); if (url) editor?.chain().focus().setLink({ href: url }).run(); }}>
        Link
      </button>
      <button class="toolbar-btn" onmousedown={(e) => e.preventDefault()} onclick={async () => {
        const info = await pickAndUploadImage()
        if (info) {
          const url = await getImageUrl(info.id)
          editor?.chain().focus().setImage({ src: url }).run()
        }
      }}>
        Img
      </button>
      <button class="toolbar-btn" onmousedown={(e) => e.preventDefault()} onclick={() => editor?.chain().focus().insertTable({ rows: 3, cols: 3 }).run()}>
        Table
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

{#if embedPickerOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="embed-overlay" onclick={() => { embedPickerOpen = false; embedPickerEditor = null }} role="dialog" aria-modal="true">
    <div class="embed-picker" onclick={(e) => e.stopPropagation()}>
      <h3 class="embed-picker-title">Embed a Page</h3>
      <div class="embed-page-list">
        {#each $pageTree.filter((p) => p.id !== $currentPage?.id) as node (node.id)}
          <button class="embed-page-item" onclick={() => selectPageForEmbed(node.id, node.title)}>
            {node.title}
          </button>
        {/each}
        {#if $pageTree.filter((p) => p.id !== $currentPage?.id).length === 0}
          <p class="embed-empty">No other pages to embed.</p>
        {/if}
      </div>
    </div>
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
    color: var(--badge-color, var(--color-fg-tertiary));
    width: fit-content;
  }

  .badge-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--badge-color, var(--color-fg-tertiary));
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 12px;
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

  /* Embed picker overlay */
  .embed-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 300;
  }

  .embed-picker {
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-lg);
    padding: 20px;
    width: 360px;
    max-height: 400px;
    overflow-y: auto;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.4);
  }

  .embed-picker-title {
    font-family: var(--font-heading);
    font-size: 16px;
    font-weight: 600;
    color: var(--color-fg-primary);
    margin: 0 0 12px;
  }

  .embed-page-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .embed-page-item {
    padding: 8px 12px;
    background: none;
    border: none;
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 14px;
    color: var(--color-fg-primary);
    text-align: left;
    cursor: pointer;
    width: 100%;
  }

  .embed-page-item:hover {
    background: var(--color-surface-tertiary);
  }

  .embed-empty {
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-tertiary);
    margin: 0;
  }

  /* Page embed node styles */
  .editor-container :global(.page-embed-wrapper) {
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-md);
    margin: 16px 0;
    overflow: hidden;
    background: var(--color-surface-tertiary);
  }

  .editor-container :global(.page-embed-header) {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 14px;
    background: var(--color-surface-card);
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .editor-container :global(.page-embed-title) {
    font-family: var(--font-heading);
    font-size: 14px;
    font-weight: 600;
    color: var(--color-accent-gold);
  }

  .editor-container :global(.page-embed-open) {
    background: none;
    border: none;
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-accent-gold);
    cursor: pointer;
    opacity: 0.7;
  }

  .editor-container :global(.page-embed-open:hover) {
    opacity: 1;
  }

  .editor-container :global(.page-embed-content) {
    padding: 12px 14px;
    font-family: var(--font-body);
    font-size: 14px;
    line-height: 1.6;
    color: var(--color-fg-secondary);
  }

  .editor-container :global(.page-embed-loading) {
    color: var(--color-fg-tertiary);
    font-style: italic;
  }

  .editor-container :global(.page-embed-error) {
    color: var(--color-status-error);
    font-style: italic;
  }

  .editor-container :global(.page-embed-empty) {
    color: var(--color-fg-tertiary);
    font-style: italic;
  }
</style>
