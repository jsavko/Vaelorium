import { callCommand } from './bridge'

export interface MapInfo {
  id: string
  title: string
  image_id: string | null
  parent_map_id: string | null
  sort_order: number
  created_at: string
  updated_at: string
}

export interface MapPin {
  id: string
  map_id: string
  page_id: string | null
  label: string | null
  x: number
  y: number
  icon: string | null
  color: string | null
  created_at: string
}

export async function createMap(title: string, imageId?: string | null): Promise<MapInfo> {
  return callCommand('create_map', { title, imageId })
}

export async function listMaps(): Promise<MapInfo[]> {
  return callCommand('list_maps')
}

export async function getMap(id: string): Promise<MapInfo> {
  return callCommand('get_map', { id })
}

export async function deleteMap(id: string): Promise<void> {
  return callCommand('delete_map', { id })
}

export async function createPin(
  mapId: string,
  x: number,
  y: number,
  pageId?: string | null,
  label?: string | null,
  icon?: string | null,
  color?: string | null,
): Promise<MapPin> {
  return callCommand('create_pin', { mapId, x, y, pageId, label, icon, color })
}

export async function updatePin(
  id: string,
  updates: { x?: number; y?: number; pageId?: string; label?: string },
): Promise<MapPin> {
  return callCommand('update_pin', { id, ...updates })
}

export async function deletePin(id: string): Promise<void> {
  return callCommand('delete_pin', { id })
}

export async function getMapPins(mapId: string): Promise<MapPin[]> {
  return callCommand('get_map_pins', { mapId })
}
