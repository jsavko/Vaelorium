import { writable, get } from 'svelte/store'
import type { RecentTome, TomeInfo, TomeMetadata } from '../api/tomes'
import * as tomesApi from '../api/tomes'

export const isTomeOpen = writable(false)
export const currentTome = writable<TomeInfo | null>(null)
export const currentTomeMetadata = writable<TomeMetadata | null>(null)
export const recentTomes = writable<RecentTome[]>([])

export async function loadRecentTomes() {
  const state = await tomesApi.getAppState()
  recentTomes.set(state.recent_tomes)
}

export async function createTome(path: string, name: string, description?: string | null) {
  const tome = await tomesApi.createTome(path, name, description)
  currentTome.set(tome)
  isTomeOpen.set(true)
  const meta = await tomesApi.getTomeMetadata()
  currentTomeMetadata.set(meta)
  await loadRecentTomes()
  return tome
}

export async function openTome(path: string) {
  const tome = await tomesApi.openTome(path)
  currentTome.set(tome)
  isTomeOpen.set(true)
  const meta = await tomesApi.getTomeMetadata()
  currentTomeMetadata.set(meta)
  await loadRecentTomes()
  return tome
}

export async function closeTome() {
  await tomesApi.closeTome()
  currentTome.set(null)
  currentTomeMetadata.set(null)
  isTomeOpen.set(false)
}

export async function updateTomeName(name: string) {
  await tomesApi.updateTomeMetadata('name', name)
  currentTomeMetadata.update((m) => (m ? { ...m, name } : m))
  currentTome.update((t) => (t ? { ...t, name } : t))
}
