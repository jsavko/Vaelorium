import { callCommand } from './bridge'

export interface RelationType {
  id: string
  name: string
  inverse_name: string | null
  color: string | null
  is_builtin: boolean
  created_at: string
}

export interface Relation {
  id: string
  source_page_id: string
  target_page_id: string
  relation_type_id: string
  description: string | null
  created_at: string
}

export interface PageRelation {
  id: string
  page_id: string
  page_title: string
  page_icon: string | null
  page_entity_type_id: string | null
  relation_type_id: string
  relation_label: string
  description: string | null
  direction: 'outgoing' | 'incoming'
}

export async function listRelationTypes(): Promise<RelationType[]> {
  return callCommand('list_relation_types')
}

export async function createRelationType(
  name: string,
  inverseName?: string | null,
  color?: string | null,
): Promise<RelationType> {
  return callCommand('create_relation_type', { name, inverseName, color })
}

export async function createRelation(
  sourcePageId: string,
  targetPageId: string,
  relationTypeId: string,
  description?: string | null,
): Promise<Relation> {
  return callCommand('create_relation', { sourcePageId, targetPageId, relationTypeId, description })
}

export async function deleteRelation(id: string): Promise<void> {
  return callCommand('delete_relation', { id })
}

export async function getPageRelations(pageId: string): Promise<PageRelation[]> {
  return callCommand('get_page_relations', { pageId })
}

export async function listAllRelations(): Promise<Relation[]> {
  return callCommand('list_all_relations')
}
