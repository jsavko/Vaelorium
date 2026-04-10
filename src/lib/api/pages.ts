import { invoke } from '@tauri-apps/api/core'

export interface Page {
  id: string
  title: string
  icon: string | null
  featured_image_path: string | null
  parent_id: string | null
  sort_order: number
  entity_type_id: string | null
  visibility: string
  created_at: string
  updated_at: string
  created_by: string | null
  updated_by: string | null
}

export interface PageTreeNode {
  id: string
  title: string
  icon: string | null
  entity_type_id: string | null
  parent_id: string | null
  sort_order: number
  children_count: number
}

export interface CreatePageInput {
  title: string
  parent_id?: string | null
  icon?: string | null
  entity_type_id?: string | null
}

export interface UpdatePageInput {
  title?: string
  icon?: string
  parent_id?: string
  sort_order?: number
  visibility?: string
  featured_image_path?: string
  entity_type_id?: string
}

export interface ReorderMove {
  id: string
  parent_id: string | null
  sort_order: number
}

export async function createPage(input: CreatePageInput): Promise<Page> {
  return invoke('create_page', { input })
}

export async function getPage(id: string): Promise<Page> {
  return invoke('get_page', { id })
}

export async function updatePage(id: string, input: UpdatePageInput): Promise<Page> {
  return invoke('update_page', { id, input })
}

export async function deletePage(id: string): Promise<void> {
  return invoke('delete_page', { id })
}

export async function listPages(): Promise<Page[]> {
  return invoke('list_pages')
}

export async function getPageTree(): Promise<PageTreeNode[]> {
  return invoke('get_page_tree')
}

export async function savePageContent(pageId: string, yjsState: number[]): Promise<void> {
  return invoke('save_page_content', { pageId, yjsState })
}

export async function getPageContent(pageId: string): Promise<number[]> {
  return invoke('get_page_content', { pageId })
}

export async function reorderPages(moves: ReorderMove[]): Promise<void> {
  return invoke('reorder_pages', { moves })
}
