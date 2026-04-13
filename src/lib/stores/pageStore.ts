import { writable, derived, get } from 'svelte/store'
import type { Page, PageTreeNode } from '../api/pages'
import * as pagesApi from '../api/pages'
import { updatePageTitleInLinks } from '../api/wikiLinks'

// Core state
export const pages = writable<Page[]>([])
export const pageTree = writable<PageTreeNode[]>([])
export const currentPageId = writable<string | null>(null)
export const currentPage = writable<Page | null>(null)
export const isLoading = writable(false)
export const recentPageIds = writable<string[]>([])

/** Increments whenever the page's Yjs content was replaced externally
 *  (e.g. by a version restore). The Editor subscribes to this in
 *  addition to currentPage.id and force-reloads its Y.Doc when the
 *  signal changes. Without this the editor keeps its in-memory Y.Doc
 *  and autosaves it back over the restored DB row. */
export const pageReloadSignal = writable(0)
export function triggerPageReload() {
  pageReloadSignal.update((n) => n + 1)
}

// Derived: build nested tree structure
export const nestedTree = derived(pageTree, ($pageTree) => {
  const roots = $pageTree.filter((n) => n.parent_id === null)
  return roots.sort((a, b) => a.sort_order - b.sort_order)
})

export function getChildren(parentId: string): PageTreeNode[] {
  const tree = get(pageTree)
  return tree
    .filter((n) => n.parent_id === parentId)
    .sort((a, b) => a.sort_order - b.sort_order)
}

// Actions
export async function loadPageTree() {
  const tree = await pagesApi.getPageTree()
  pageTree.set(tree)
}

export async function loadPage(id: string) {
  isLoading.set(true)
  try {
    const page = await pagesApi.getPage(id)
    currentPage.set(page)
    currentPageId.set(id)

    // Track recent pages
    recentPageIds.update((ids) => {
      const filtered = ids.filter((i) => i !== id)
      return [id, ...filtered].slice(0, 5)
    })
  } finally {
    isLoading.set(false)
  }
}

export async function createPage(title: string, parentId?: string | null, entityTypeId?: string | null) {
  const page = await pagesApi.createPage({ title, parent_id: parentId, entity_type_id: entityTypeId })
  await loadPageTree()
  await loadPage(page.id)
  return page
}

export async function updateCurrentPage(input: pagesApi.UpdatePageInput) {
  const id = get(currentPageId)
  if (!id) return
  // Capture old title as a string BEFORE the update mutates the object
  const oldTitle = get(currentPage)?.title
  const updated = await pagesApi.updatePage(id, input)
  currentPage.set(updated)
  await loadPageTree()

  // If title changed, update mentions across all linking pages
  if (input.title && oldTitle && input.title !== oldTitle) {
    try {
      await updatePageTitleInLinks(id, oldTitle, input.title)
    } catch (err) {
      console.warn('Failed to update mentions after rename:', err)
    }
  }
}

export async function deleteCurrentPage() {
  const id = get(currentPageId)
  if (!id) return
  await pagesApi.deletePage(id)
  currentPage.set(null)
  currentPageId.set(null)
  await loadPageTree()
}

export async function reorderPages(moves: pagesApi.ReorderMove[]) {
  await pagesApi.reorderPages(moves)
  await loadPageTree()
}
