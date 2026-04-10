import { writable, derived, get } from 'svelte/store'
import type { Page, PageTreeNode } from '../api/pages'
import * as pagesApi from '../api/pages'

// Core state
export const pages = writable<Page[]>([])
export const pageTree = writable<PageTreeNode[]>([])
export const currentPageId = writable<string | null>(null)
export const currentPage = writable<Page | null>(null)
export const isLoading = writable(false)
export const recentPageIds = writable<string[]>([])

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

export async function createPage(title: string, parentId?: string | null) {
  const page = await pagesApi.createPage({ title, parent_id: parentId })
  await loadPageTree()
  await loadPage(page.id)
  return page
}

export async function updateCurrentPage(input: pagesApi.UpdatePageInput) {
  const id = get(currentPageId)
  if (!id) return
  const updated = await pagesApi.updatePage(id, input)
  currentPage.set(updated)
  await loadPageTree()
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
