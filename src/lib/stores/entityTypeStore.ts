import { writable, derived, get } from 'svelte/store'
import type { EntityType, EntityTypeField, FieldValue } from '../api/entityTypes'
import * as api from '../api/entityTypes'

// Core state
export const entityTypes = writable<EntityType[]>([])
export const currentPageFields = writable<EntityTypeField[]>([])
export const currentPageFieldValues = writable<FieldValue[]>([])

// Derived: map of type id -> type for fast lookups
export const entityTypeMap = derived(entityTypes, ($types) => {
  const map = new Map<string, EntityType>()
  for (const t of $types) {
    map.set(t.id, t)
  }
  return map
})

// Derived: built-in types only
export const builtinTypes = derived(entityTypes, ($types) =>
  $types.filter((t) => t.is_builtin),
)

// Derived: custom types only
export const customTypes = derived(entityTypes, ($types) =>
  $types.filter((t) => !t.is_builtin),
)

// Actions

export async function loadEntityTypes() {
  const types = await api.listEntityTypes()
  entityTypes.set(types)
}

export async function loadFieldsForType(entityTypeId: string) {
  const fields = await api.listEntityTypeFields(entityTypeId)
  currentPageFields.set(fields)
}

export async function loadFieldValues(pageId: string) {
  const values = await api.getPageFieldValues(pageId)
  currentPageFieldValues.set(values)
}

export async function loadPageEntityData(pageId: string, entityTypeId: string | null) {
  if (entityTypeId) {
    const [fields, values] = await Promise.all([
      api.listEntityTypeFields(entityTypeId),
      api.getPageFieldValues(pageId),
    ])
    currentPageFields.set(fields)
    currentPageFieldValues.set(values)
  } else {
    currentPageFields.set([])
    currentPageFieldValues.set([])
  }
}

export async function setFieldValue(pageId: string, fieldId: string, value: string | null) {
  const fv = await api.setFieldValue(pageId, fieldId, value)
  currentPageFieldValues.update((vals) => {
    const idx = vals.findIndex((v) => v.field_id === fieldId)
    if (idx >= 0) {
      vals[idx] = fv
      return [...vals]
    }
    return [...vals, fv]
  })
  return fv
}

export async function deleteFieldValue(pageId: string, fieldId: string) {
  await api.deleteFieldValue(pageId, fieldId)
  currentPageFieldValues.update((vals) => vals.filter((v) => v.field_id !== fieldId))
}

export async function createEntityType(name: string, icon?: string | null, color?: string | null) {
  const type = await api.createEntityType(name, icon, color)
  await loadEntityTypes()
  return type
}

export async function updateEntityType(id: string, name?: string, icon?: string, color?: string) {
  const type = await api.updateEntityType(id, name, icon, color)
  await loadEntityTypes()
  return type
}

export async function deleteEntityType(id: string) {
  await api.deleteEntityType(id)
  await loadEntityTypes()
}

export function getEntityTypeById(id: string): EntityType | undefined {
  return get(entityTypeMap).get(id)
}
