import { writable } from 'svelte/store'
import type { RelationType, PageRelation } from '../api/relations'
import * as api from '../api/relations'

export const relationTypes = writable<RelationType[]>([])
export const currentPageRelations = writable<PageRelation[]>([])

export async function loadRelationTypes() {
  const types = await api.listRelationTypes()
  relationTypes.set(types)
}

export async function loadPageRelations(pageId: string) {
  const rels = await api.getPageRelations(pageId)
  currentPageRelations.set(rels)
}

export async function addRelation(
  sourcePageId: string,
  targetPageId: string,
  relationTypeId: string,
  description?: string | null,
) {
  await api.createRelation(sourcePageId, targetPageId, relationTypeId, description)
  await loadPageRelations(sourcePageId)
}

export async function removeRelation(id: string, currentPageId: string) {
  await api.deleteRelation(id)
  await loadPageRelations(currentPageId)
}
