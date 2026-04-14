import { writable } from 'svelte/store'
import type { MapInfo, MapPin } from '../api/maps'
import * as api from '../api/maps'

export const maps = writable<MapInfo[]>([])
export const currentMap = writable<MapInfo | null>(null)
export const currentMapPins = writable<MapPin[]>([])

export async function loadMaps() {
  const list = await api.listMaps()
  maps.set(list)
}

export async function loadMap(id: string) {
  const map = await api.getMap(id)
  currentMap.set(map)
  const pins = await api.getMapPins(id)
  currentMapPins.set(pins)
}

export async function createMap(title: string, imageId?: string | null) {
  const map = await api.createMap(title, imageId)
  await loadMaps()
  return map
}

export async function addPin(
  mapId: string,
  x: number,
  y: number,
  pageId?: string | null,
  label?: string | null,
  icon?: string | null,
  color?: string | null,
) {
  const pin = await api.createPin(mapId, x, y, pageId, label, icon, color)
  currentMapPins.update((pins) => [...pins, pin])
  return pin
}

export async function renameMap(id: string, title: string) {
  const updated = await api.updateMap(id, title)
  maps.update((list) => list.map((m) => (m.id === id ? updated : m)))
  currentMap.update((cur) => (cur && cur.id === id ? updated : cur))
}

export async function deleteMap(id: string) {
  await api.deleteMap(id)
  maps.update((list) => list.filter((m) => m.id !== id))
  currentMap.update((cur) => (cur && cur.id === id ? null : cur))
}

export async function removePin(id: string) {
  await api.deletePin(id)
  currentMapPins.update((pins) => pins.filter((p) => p.id !== id))
}
