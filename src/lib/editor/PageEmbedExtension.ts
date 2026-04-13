import { Node, mergeAttributes, Editor, type RawCommands } from '@tiptap/core'

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    pageEmbed: {
      insertPageEmbed: (attrs: { pageId: string; pageTitle: string }) => ReturnType
    }
  }
}
import StarterKit from '@tiptap/starter-kit'
import { Table } from '@tiptap/extension-table'
import { TableRow } from '@tiptap/extension-table-row'
import { TableHeader } from '@tiptap/extension-table-header'
import { TableCell } from '@tiptap/extension-table-cell'
import ImageExt from '@tiptap/extension-image'
import Link from '@tiptap/extension-link'
import Collaboration from '@tiptap/extension-collaboration'
import { getPageContent } from '../api/pages'
import { getPage } from '../api/pages'
import * as Y from 'yjs'

// Track which pages are embedded in the current render to prevent recursion
const renderingStack = new Set<string>()

async function getPageHtml(pageId: string): Promise<{ title: string; html: string; entityTypeId: string | null }> {
  try {
    const [page, contentArray] = await Promise.all([
      getPage(pageId),
      getPageContent(pageId),
    ])

    let html = ''
    if (contentArray && contentArray.length > 0) {
      // Create a temporary Yjs doc and headless TipTap editor to render content
      const doc = new Y.Doc()
      Y.applyUpdate(doc, new Uint8Array(contentArray))

      const tempEditor = new Editor({
        extensions: [
          StarterKit.configure({ heading: { levels: [1, 2, 3] }, undoRedo: false }),
          Table, TableRow, TableHeader, TableCell,
          ImageExt.configure({ inline: false }),
          Link.configure({ openOnClick: false }),
          Collaboration.configure({ document: doc, field: 'content' }),
        ],
      })

      html = tempEditor.getHTML()
      tempEditor.destroy()
    }

    return { title: page.title, html, entityTypeId: page.entity_type_id }
  } catch (e) {
    console.warn('Failed to load embed:', e)
    return { title: 'Page not found', html: '<p>This page could not be loaded.</p>', entityTypeId: null }
  }
}

export const PageEmbedNode = Node.create({
  name: 'pageEmbed',
  group: 'block',
  atom: true,
  draggable: true,

  addAttributes() {
    return {
      pageId: { default: null },
      pageTitle: { default: 'Embedded Page' },
    }
  },

  parseHTML() {
    return [
      {
        tag: 'div[data-page-embed]',
        getAttrs: (el) => ({
          pageId: (el as HTMLElement).getAttribute('data-page-id'),
          pageTitle: (el as HTMLElement).getAttribute('data-page-title'),
        }),
      },
    ]
  },

  renderHTML({ HTMLAttributes }) {
    return [
      'div',
      mergeAttributes(HTMLAttributes, {
        'data-page-embed': '',
        'data-page-id': HTMLAttributes.pageId,
        'data-page-title': HTMLAttributes.pageTitle,
        class: 'page-embed-wrapper',
      }),
      ['div', { class: 'page-embed-header' }, HTMLAttributes.pageTitle || 'Embedded Page'],
      ['div', { class: 'page-embed-content' }, 'Loading...'],
    ]
  },

  addNodeView() {
    return ({ node, editor }) => {
      const dom = document.createElement('div')
      dom.classList.add('page-embed-wrapper')
      dom.contentEditable = 'false'
      dom.setAttribute('data-page-embed', '')

      const pageId = node.attrs.pageId
      const pageTitle = node.attrs.pageTitle || 'Embedded Page'

      // Header
      const header = document.createElement('div')
      header.classList.add('page-embed-header')

      const titleSpan = document.createElement('span')
      titleSpan.classList.add('page-embed-title')
      titleSpan.textContent = pageTitle

      const openBtn = document.createElement('button')
      openBtn.classList.add('page-embed-open')
      openBtn.textContent = 'Open →'
      openBtn.onclick = (e) => {
        e.preventDefault()
        e.stopPropagation()
        // Navigate to the embedded page
        window.dispatchEvent(new CustomEvent('vaelorium:navigate', { detail: { pageId } }))
      }

      header.appendChild(titleSpan)
      header.appendChild(openBtn)

      // Content area
      const content = document.createElement('div')
      content.classList.add('page-embed-content')
      content.innerHTML = '<p class="page-embed-loading">Loading embedded content...</p>'

      dom.appendChild(header)
      dom.appendChild(content)

      // Check for recursive embedding
      if (renderingStack.has(pageId)) {
        content.innerHTML = '<p class="page-embed-error">Recursive embed detected — cannot embed a page within itself.</p>'
      } else if (pageId) {
        renderingStack.add(pageId)
        getPageHtml(pageId).then(({ title, html, entityTypeId }) => {
          titleSpan.textContent = title
          content.innerHTML = html || '<p class="page-embed-empty">Empty page</p>'
          renderingStack.delete(pageId)
        }).catch(() => {
          content.innerHTML = '<p class="page-embed-error">Failed to load embedded page.</p>'
          renderingStack.delete(pageId)
        })
      }

      return { dom }
    }
  },

  addCommands(): Partial<RawCommands> {
    return {
      insertPageEmbed:
        (attrs: { pageId: string; pageTitle: string }) =>
        ({ commands }: { commands: any }) => {
          return commands.insertContent({
            type: this.name,
            attrs,
          })
        },
    }
  },
})
