<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { Editor } from '@tiptap/core'
  import { createEditorExtensions } from '../editor/EditorConfig'
  import { LocalYjsProvider } from '../editor/YjsProvider'
  import { currentPage, updateCurrentPage, loadPage, pageTree, pageReloadSignal } from '../stores/pageStore'
  import { entityTypeMap } from '../stores/entityTypeStore'
  import { currentPageRelations } from '../stores/relationStore'
  import IconPicker from './IconPicker.svelte'
  import InputModal from './InputModal.svelte'
  import { get } from 'svelte/store'
  import { pickAndUploadImage, getImageUrl } from '../api/images'
  import type { Page } from '../api/pages'

  let editorElement: HTMLDivElement
  let editor: Editor | null = null
  let provider: LocalYjsProvider | null = null
  let titleInput: HTMLInputElement
  let currentLoadedPageId: string | null = null
  let embedPickerOpen = $state(false)
  let linkModalOpen = $state(false)
  let embedPickerEditor: Editor | null = null
  let imageToolbar = $state<{ x: number; y: number; alignment: string } | null>(null)

  // Track image selection for toolbar
  function updateImageToolbar() {
    if (!editor) { imageToolbar = null; return }
    const { from } = editor.state.selection
    const node = editor.state.doc.nodeAt(from)
    if (node?.type.name === 'image') {
      const dom = editor.view.nodeDOM(from) as HTMLElement | null
      if (dom) {
        const rect = dom.getBoundingClientRect()
        const containerRect = editorElement.getBoundingClientRect()
        imageToolbar = {
          x: rect.left + rect.width / 2 - containerRect.left,
          y: rect.top - containerRect.top - 8,
          alignment: node.attrs.alignment || 'center',
        }
        return
      }
    }
    imageToolbar = null
  }

  function setImageAlignment(alignment: string) {
    if (!editor) return
    ;(editor.commands as any).setImageAlignment(alignment)
    imageToolbar = imageToolbar ? { ...imageToolbar, alignment } : null
  }

  function deleteSelectedImage() {
    if (!editor) return
    editor.chain().focus().deleteSelection().run()
    imageToolbar = null
  }

  // React to page changes (new page) or explicit reload signals (version
  // restore). Tracking pageReloadSignal here means restore-current-page
  // triggers a full editor re-init — otherwise the in-memory Y.Doc keeps
  // its pre-restore state and autosaves it back over the restored DB row.
  let lastReloadSignal = -1
  $effect(() => {
    const page = $currentPage
    const signal = $pageReloadSignal
    if (!page) return
    const signalChanged = signal !== lastReloadSignal
    const pageChanged = page.id !== currentLoadedPageId
    if (pageChanged || signalChanged) {
      lastReloadSignal = signal
      // DB was just replaced externally (version restore) — discard the
      // stale in-memory Y.Doc without calling destroy(), because destroy()
      // runs a final save() that would clobber the fresh DB content.
      loadEditor(page, { discardExisting: signalChanged && !pageChanged })
    }
  })

  async function loadEditor(page: Page, opts: { discardExisting?: boolean } = {}) {
    // Tear down the previous editor. Normal path saves in-memory Y.Doc first
    // (so tab-switches don't lose unsaved edits). Discard path skips the
    // save — used when the DB was externally replaced (restore) and saving
    // our stale in-memory state would silently undo the replacement.
    if (provider) {
      if (opts.discardExisting) {
        provider.discard()
      } else {
        await provider.save()
        await provider.destroy()
      }
      provider = null
    }
    if (editor) {
      editor.destroy()
      editor = null
    }

    currentLoadedPageId = null // force next page-effect to accept this as fresh
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

    // Track image selection for floating toolbar
    editor.on('selectionUpdate', updateImageToolbar)
    editor.on('blur', () => { imageToolbar = null })

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
    const link = target.closest('a[href^="#page:"], a[href^="#map:"], a[href^="#timeline:"]')
    if (link) {
      e.preventDefault()
      const href = link.getAttribute('href')
      if (!href) return
      if (href.startsWith('#page:')) {
        const pageId = href.replace('#page:', '')
        loadPage(pageId)
        window.dispatchEvent(new CustomEvent('vaelorium:page-selected'))
      } else if (href.startsWith('#map:')) {
        const mapId = href.replace('#map:', '')
        window.dispatchEvent(new CustomEvent('vaelorium:open-map', { detail: { mapId } }))
      } else if (href.startsWith('#timeline:')) {
        const timelineId = href.replace('#timeline:', '')
        window.dispatchEvent(new CustomEvent('vaelorium:open-timeline', { detail: { timelineId } }))
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

  // Prevent browser from opening dropped files — allow our handler to work
  $effect(() => {
    function preventDefaultDrag(e: DragEvent) {
      e.preventDefault()
    }
    function handleWindowDrop(e: DragEvent) {
      // Only prevent default — the editor's own handler will process the drop
      e.preventDefault()
    }
    window.addEventListener('dragover', preventDefaultDrag)
    window.addEventListener('drop', handleWindowDrop)
    return () => {
      window.removeEventListener('dragover', preventDefaultDrag)
      window.removeEventListener('drop', handleWindowDrop)
    }
  })

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
        {#if $currentPageRelations.length > 0}
          <span class="meta-sep">·</span>
          <span class="meta-text">{$currentPageRelations.length} connection{$currentPageRelations.length !== 1 ? 's' : ''}</span>
        {/if}
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
      <button class="toolbar-btn" onmousedown={(e) => e.preventDefault()} onclick={() => linkModalOpen = true}>
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

    <div class="editor-container-wrapper">
      <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
      <div
        class="editor-container"
        bind:this={editorElement}
        onclick={handleEditorClick}
        ondrop={handleEditorDrop}
        ondragover={(e) => e.preventDefault()}
      ></div>

      {#if imageToolbar}
        <div
          class="image-toolbar"
          style:left="{imageToolbar.x}px"
          style:top="{imageToolbar.y}px"
        >
          <button
            class="img-tool-btn"
            class:active={imageToolbar.alignment === 'left'}
            onclick={() => setImageAlignment('left')}
            title="Float left"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="8" height="8"></rect>
              <line x1="15" y1="5" x2="21" y2="5"></line>
              <line x1="15" y1="9" x2="21" y2="9"></line>
              <line x1="3" y1="15" x2="21" y2="15"></line>
              <line x1="3" y1="19" x2="21" y2="19"></line>
            </svg>
          </button>
          <button
            class="img-tool-btn"
            class:active={imageToolbar.alignment === 'center'}
            onclick={() => setImageAlignment('center')}
            title="Center"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="6" y="3" width="12" height="8"></rect>
              <line x1="3" y1="15" x2="21" y2="15"></line>
              <line x1="3" y1="19" x2="21" y2="19"></line>
            </svg>
          </button>
          <button
            class="img-tool-btn"
            class:active={imageToolbar.alignment === 'right'}
            onclick={() => setImageAlignment('right')}
            title="Float right"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="13" y="3" width="8" height="8"></rect>
              <line x1="3" y1="5" x2="9" y2="5"></line>
              <line x1="3" y1="9" x2="9" y2="9"></line>
              <line x1="3" y1="15" x2="21" y2="15"></line>
              <line x1="3" y1="19" x2="21" y2="19"></line>
            </svg>
          </button>
          <span class="img-tool-sep"></span>
          <button class="img-tool-btn danger" onclick={deleteSelectedImage} title="Delete">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>
      {/if}
    </div>
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

<InputModal
  open={linkModalOpen}
  title="Insert Link"
  placeholder="https://..."
  confirmLabel="Insert"
  onConfirm={(url) => { linkModalOpen = false; editor?.chain().focus().setLink({ href: url }).run() }}
  onCancel={() => linkModalOpen = false}
/>

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

  .meta-sep {
    color: var(--color-fg-tertiary);
    opacity: 0.5;
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
    min-height: 100%;
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

  .editor-container-wrapper {
    position: relative;
    flex: 1;
    overflow-y: auto;
  }

  /* Image toolbar */
  .image-toolbar {
    position: absolute;
    transform: translate(-50%, -100%);
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 4px;
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-md);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    z-index: 20;
  }

  .img-tool-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    background: none;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--color-fg-secondary);
    cursor: pointer;
  }

  .img-tool-btn:hover {
    background: var(--color-surface-tertiary);
  }

  .img-tool-btn.active {
    background: var(--color-accent-gold-subtle);
    color: var(--color-accent-gold);
  }

  .img-tool-btn.danger:hover {
    background: rgba(184, 92, 92, 0.15);
    color: var(--color-status-error);
  }

  .img-tool-sep {
    width: 1px;
    height: 16px;
    background: var(--color-border-default);
    margin: 0 2px;
  }

  /* Callout blocks */
  .editor-container :global(.callout) {
    border-left: 3px solid; padding: 12px 16px; margin: 12px 0;
    border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
    background: var(--color-surface-tertiary);
  }
  .editor-container :global(.callout-info) { border-color: var(--color-status-info); }
  .editor-container :global(.callout-warning) { border-color: var(--color-status-warning); }
  .editor-container :global(.callout-note) { border-color: var(--color-accent-gold); }

  /* Image alignment */
  .editor-container :global(.editor-content img) {
    max-width: 100%;
    border-radius: var(--radius-md);
    cursor: pointer;
  }

  .editor-container :global(.editor-content img.img-align-center) {
    display: block;
    margin: 16px auto;
  }

  .editor-container :global(.editor-content img.img-align-left) {
    float: left;
    margin: 4px 20px 12px 0;
    max-width: 50%;
  }

  .editor-container :global(.editor-content img.img-align-right) {
    float: right;
    margin: 4px 0 12px 20px;
    max-width: 50%;
  }

  .editor-container :global(.editor-content img.ProseMirror-selectednode) {
    outline: 2px solid var(--color-accent-gold);
    outline-offset: 2px;
  }

  /* Clear floats after paragraphs that follow floated images */
  .editor-container :global(.editor-content p::after) {
    content: '';
    display: table;
    clear: both;
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
