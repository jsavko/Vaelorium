import { writable, get } from 'svelte/store'
import type { RecentTome, TomeInfo, TomeMetadata } from '../api/tomes'
import * as tomesApi from '../api/tomes'
import { currentPageId, currentPage, pageTree, pages } from './pageStore'

/** Wipe cross-tome state so opening a new Tome doesn't render the
 *  previous Tome's last-open page. Previously `currentPageId` leaked
 *  across openTome/closeTome calls → the new Tome's UI picked up a
 *  stale id and tried to render content from the prior database. */
function resetTomeScopedStores() {
  currentPageId.set(null)
  currentPage.set(null)
  pageTree.set([])
  pages.set([])
}

export const isTomeOpen = writable(false)
export const currentTome = writable<TomeInfo | null>(null)
export const currentTomeMetadata = writable<TomeMetadata | null>(null)
export const recentTomes = writable<RecentTome[]>([])

export async function loadRecentTomes() {
  const state = await tomesApi.getAppState()
  recentTomes.set(state.recent_tomes)
}

export async function createTome(path: string, name: string, description?: string | null) {
  resetTomeScopedStores()
  const tome = await tomesApi.createTome(path, name, description)
  currentTome.set(tome)
  isTomeOpen.set(true)
  const meta = await tomesApi.getTomeMetadata()
  currentTomeMetadata.set(meta)
  await loadRecentTomes()
  return tome
}

export async function openTome(path: string) {
  resetTomeScopedStores()
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
  resetTomeScopedStores()
  currentTome.set(null)
  currentTomeMetadata.set(null)
  isTomeOpen.set(false)
}

export async function updateTomeName(name: string) {
  await tomesApi.updateTomeMetadata('name', name)
  currentTomeMetadata.update((m) => (m ? { ...m, name } : m))
  currentTome.update((t) => (t ? { ...t, name } : t))
}
