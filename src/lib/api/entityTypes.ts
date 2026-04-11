import { callCommand } from './bridge'

// ── Entity Types ──

export interface EntityType {
  id: string
  name: string
  icon: string | null
  color: string | null
  is_builtin: boolean
  sort_order: number
  created_at: string
  updated_at: string
}

export async function listEntityTypes(): Promise<EntityType[]> {
  return callCommand('list_entity_types')
}

export async function getEntityType(id: string): Promise<EntityType> {
  return callCommand('get_entity_type', { id })
}

export async function createEntityType(
  name: string,
  icon?: string | null,
  color?: string | null,
): Promise<EntityType> {
  return callCommand('create_entity_type', { name, icon, color })
}

export async function updateEntityType(
  id: string,
  name?: string,
  icon?: string,
  color?: string,
): Promise<EntityType> {
  return callCommand('update_entity_type', { id, name, icon, color })
}

export async function deleteEntityType(id: string): Promise<void> {
  return callCommand('delete_entity_type', { id })
}

// ── Entity Type Fields ──

export interface EntityTypeField {
  id: string
  entity_type_id: string
  name: string
  field_type: 'text' | 'number' | 'select' | 'multi_select' | 'long_text' | 'boolean' | 'page_reference'
  sort_order: number
  is_required: boolean
  default_value: string | null
  options: string | null
  reference_type_id: string | null
  created_at: string
}

export interface ReorderFieldMove {
  id: string
  sort_order: number
}

export async function listEntityTypeFields(entityTypeId: string): Promise<EntityTypeField[]> {
  return callCommand('list_entity_type_fields', { entity_type_id: entityTypeId })
}

export async function createEntityTypeField(
  entityTypeId: string,
  name: string,
  fieldType: string,
  options?: string | null,
  isRequired?: boolean,
  defaultValue?: string | null,
  referenceTypeId?: string | null,
): Promise<EntityTypeField> {
  return callCommand('create_entity_type_field', {
    entity_type_id: entityTypeId,
    name,
    field_type: fieldType,
    options,
    is_required: isRequired,
    default_value: defaultValue,
    reference_type_id: referenceTypeId,
  })
}

export async function updateEntityTypeField(
  id: string,
  updates: {
    name?: string
    field_type?: string
    is_required?: boolean
    default_value?: string
    options?: string
    reference_type_id?: string
  },
): Promise<EntityTypeField> {
  return callCommand('update_entity_type_field', { id, ...updates })
}

export async function deleteEntityTypeField(id: string): Promise<void> {
  return callCommand('delete_entity_type_field', { id })
}

export async function reorderEntityTypeFields(moves: ReorderFieldMove[]): Promise<void> {
  return callCommand('reorder_entity_type_fields', { moves })
}

// ── Field Values ──

export interface FieldValue {
  id: string
  page_id: string
  field_id: string
  value: string | null
}

export interface PageByFieldResult {
  id: string
  title: string
  icon: string | null
  entity_type_id: string | null
}

export async function getPageFieldValues(pageId: string): Promise<FieldValue[]> {
  return callCommand('get_page_field_values', { page_id: pageId })
}

export async function setFieldValue(
  pageId: string,
  fieldId: string,
  value: string | null,
): Promise<FieldValue> {
  return callCommand('set_field_value', { page_id: pageId, field_id: fieldId, value })
}

export async function deleteFieldValue(pageId: string, fieldId: string): Promise<void> {
  return callCommand('delete_field_value', { page_id: pageId, field_id: fieldId })
}

export async function queryPagesByField(
  fieldId: string,
  value: string,
): Promise<PageByFieldResult[]> {
  return callCommand('query_pages_by_field', { field_id: fieldId, value })
}
