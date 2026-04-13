<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { Editor } from '@tiptap/core'
  import StarterKit from '@tiptap/starter-kit'
  import { Table } from '@tiptap/extension-table'
  import Image from '@tiptap/extension-image'
  import Link from '@tiptap/extension-link'
  import * as Y from 'yjs'
  import { getPageContent } from '../api/pages'
  import { currentPage } from '../stores/pageStore'
  import type { Page } from '../api/pages'

  // svelte-ignore non_reactive_update -- assigned by bind:this at mount only
  let viewerElement: HTMLDivElement
  let editor: Editor | null = null
  let currentLoadedId: string | null = null

  $effect(() => {
    const page = $currentPage
    if (page && page.id !== currentLoadedId) {
      loadReadingView(page)
    }
  })

  async function loadReadingView(page: Page) {
    if (editor) {
      editor.destroy()
      editor = null
    }
    currentLoadedId = page.id

    const stateArray = await getPageContent(page.id)
    const doc = new Y.Doc()
    if (stateArray && stateArray.length > 0) {
      Y.applyUpdate(doc, new Uint8Array(stateArray))
    }

    // Create read-only editor to render the content
    const fragment = doc.getXmlFragment('default')

    editor = new Editor({
      element: viewerElement,
      extensions: [
        StarterKit.configure({ undoRedo: false }),
        Table.configure({ resizable: false }),
        Image,
        Link.configure({ openOnClick: true }),
      ],
      editable: false,
      editorProps: {
        attributes: { class: 'reading-content' },
      },
    })

    // Set content from Yjs doc JSON
    const json = editor.getJSON()
    if (json) {
      editor.commands.setContent(json)
    }
  }

  onDestroy(() => {
    if (editor) editor.destroy()
  })
</script>

{#if $currentPage}
  <div class="reading-wrapper">
    {#if $currentPage.featured_image_path}
      <div class="hero-image">
        <img src={$currentPage.featured_image_path} alt="" />
      </div>
    {/if}

    <div class="reading-header">
      {#if $currentPage.entity_type_id}
        <span class="entity-badge">{$currentPage.entity_type_id}</span>
      {/if}
      <h1 class="page-title">{$currentPage.title}</h1>
      <div class="page-meta">
        <span>Last edited {new Date($currentPage.updated_at).toLocaleDateString()}</span>
      </div>
      <div class="title-divider"></div>
    </div>

    <div class="reading-body" bind:this={viewerElement}></div>
  </div>
{/if}

<style>
  .reading-wrapper {
    flex: 1;
    max-width: 800px;
    margin: 0 auto;
    width: 100%;
    padding: 0 60px 40px;
    overflow-y: auto;
  }

  .hero-image {
    width: calc(100% + 120px);
    margin: 0 -60px;
    height: 220px;
    overflow: hidden;
    margin-bottom: 32px;
  }

  .hero-image img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .reading-header {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-top: 32px;
    margin-bottom: 24px;
  }

  .entity-badge {
    display: inline-flex;
    padding: 4px 10px;
    font-family: var(--font-ui);
    font-size: 12px;
    font-weight: 500;
    border-radius: 20px;
    background: var(--color-surface-tertiary);
    color: var(--color-fg-tertiary);
    width: fit-content;
    text-transform: capitalize;
  }

  .page-title {
    font-family: var(--font-heading);
    font-size: 40px;
    font-weight: 700;
    color: var(--color-fg-primary);
    margin: 0;
  }

  .page-meta {
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-tertiary);
  }

  .title-divider {
    height: 1px;
    background: var(--color-border-subtle);
  }

  .reading-body :global(.reading-content) {
    font-family: var(--font-body);
    font-size: 17px;
    line-height: 1.8;
    color: var(--color-fg-secondary);
    outline: none;
  }

  .reading-body :global(.reading-content h1) {
    font-family: var(--font-heading);
    font-size: 26px;
    font-weight: 600;
    color: var(--color-fg-primary);
    margin: 24px 0 12px;
  }

  .reading-body :global(.reading-content h2) {
    font-family: var(--font-heading);
    font-size: 22px;
    font-weight: 600;
    color: var(--color-fg-primary);
    margin: 20px 0 10px;
  }

  .reading-body :global(.reading-content p) {
    margin: 0 0 16px;
  }

  .reading-body :global(.reading-content a) {
    color: var(--color-accent-gold);
  }
</style>
