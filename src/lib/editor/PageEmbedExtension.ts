import { Node, mergeAttributes } from '@tiptap/core'
import { getPageContent } from '../api/pages'
import { getPage } from '../api/pages'
import { callCommand } from '../api/bridge'
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
      const doc = new Y.Doc()
      Y.applyUpdate(doc, new Uint8Array(contentArray))
      const xmlFragment = doc.getXmlFragment('content')
      html = xmlFragmentToHtml(xmlFragment)
    }

    return { title: page.title, html, entityTypeId: page.entity_type_id }
  } catch {
    return { title: 'Page not found', html: '<p>This page could not be loaded.</p>', entityTypeId: null }
  }
}

function xmlFragmentToHtml(fragment: Y.XmlFragment): string {
  let html = ''
  fragment.forEach((item) => {
    if (item instanceof Y.XmlElement) {
      const tag = item.nodeName
      const attrs = item.getAttributes()
      let attrStr = ''
      for (const [key, value] of Object.entries(attrs)) {
        attrStr += ` ${key}="${value}"`
      }
      const inner = xmlFragmentToHtml(item as unknown as Y.XmlFragment)
      html += `<${tag}${attrStr}>${inner}</${tag}>`
    } else if (item instanceof Y.XmlText) {
      html += item.toString()
    }
  })
  return html
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

  addCommands() {
    return {
      insertPageEmbed:
        (attrs: { pageId: string; pageTitle: string }) =>
        ({ commands }) => {
          return commands.insertContent({
            type: this.name,
            attrs,
          })
        },
    }
  },
})
