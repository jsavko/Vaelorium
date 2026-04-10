import { callCommand } from './bridge'
import { getPageContent, savePageContent } from './pages'
import * as Y from 'yjs'

interface BacklinkResult {
  page_id: string
  title: string
  entity_type_id: string | null
}

/**
 * When a page is renamed, update all @mention links pointing to it
 * across all other pages that reference it.
 *
 * Strategy: load each linking page's Yjs doc, get its XML fragment,
 * walk the tree to find text nodes inside links with matching href,
 * and update the text content.
 */
export async function updatePageTitleInLinks(
  pageId: string,
  oldTitle: string,
  newTitle: string,
): Promise<number> {
  const backlinks: BacklinkResult[] = await callCommand('get_backlinks', { pageId })
  let updatedCount = 0

  for (const backlink of backlinks) {
    try {
      const yjsState = await getPageContent(backlink.page_id)
      if (!yjsState || yjsState.length === 0) continue

      const doc = new Y.Doc()
      Y.applyUpdate(doc, new Uint8Array(yjsState))

      const fragment = doc.getXmlFragment('content')
      const updated = walkAndUpdateLinks(fragment, pageId, oldTitle, newTitle)

      if (updated) {
        const newState = Y.encodeStateAsUpdate(doc)
        await savePageContent(backlink.page_id, Array.from(newState))
        updatedCount++
      }

      doc.destroy()
    } catch (err) {
      console.warn(`Failed to update links in page ${backlink.page_id}:`, err)
    }
  }

  return updatedCount
}

/**
 * Walk a Yjs XmlFragment tree and update text nodes inside links
 * that point to the renamed page.
 */
function walkAndUpdateLinks(
  node: any,
  pageId: string,
  oldTitle: string,
  newTitle: string,
): boolean {
  let updated = false
  const targetHref = `#page:${pageId}`

  // Iterate all elements in the fragment
  try {
    const elements = node.toArray ? node.toArray() : []
    for (const el of elements) {
      // Recursively walk child elements
      if (el.toArray) {
        if (walkAndUpdateLinks(el, pageId, oldTitle, newTitle)) {
          updated = true
        }
      }

      // Check if this is a YText with formatting (marks/attributes)
      if (el instanceof Y.Text || (el.toDelta && typeof el.toDelta === 'function')) {
        const delta = el.toDelta()
        let offset = 0
        for (const op of delta) {
          if (
            typeof op.insert === 'string' &&
            op.attributes?.link?.href === targetHref &&
            op.insert === oldTitle
          ) {
            // Found matching link text — replace it
            el.delete(offset, oldTitle.length)
            el.insert(offset, newTitle, op.attributes)
            updated = true
            break // Delta is invalidated after mutation
          }
          offset += typeof op.insert === 'string' ? op.insert.length : 1
        }
      }
    }
  } catch (err) {
    // Silently handle iteration errors on different Yjs types
  }

  return updated
}
