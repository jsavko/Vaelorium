import { writable } from 'svelte/store'
import type { Timeline, TimelineEvent } from '../api/timelines'
import * as api from '../api/timelines'

export const timelines = writable<Timeline[]>([])
export const currentTimeline = writable<Timeline | null>(null)
export const currentEvents = writable<TimelineEvent[]>([])

export async function loadTimelines() {
  const list = await api.listTimelines()
  timelines.set(list)
}

export async function loadTimeline(id: string) {
  const events = await api.getTimelineEvents(id)
  currentEvents.set(events)
  const list = await api.listTimelines()
  const tl = list.find((t) => t.id === id)
  currentTimeline.set(tl || null)
}

export async function createTimeline(name: string, description?: string | null) {
  const tl = await api.createTimeline(name, description)
  await loadTimelines()
  return tl
}

export async function addEvent(
  timelineId: string,
  title: string,
  date: string,
  description?: string | null,
  pageId?: string | null,
) {
  const evt = await api.createTimelineEvent(timelineId, title, date, description, null, pageId)
  currentEvents.update((events) => [...events, evt].sort((a, b) => a.date.localeCompare(b.date)))
  return evt
}

export async function renameTimeline(id: string, name: string) {
  const updated = await api.updateTimeline(id, name)
  timelines.update((list) => list.map((t) => (t.id === id ? updated : t)))
  currentTimeline.update((cur) => (cur && cur.id === id ? updated : cur))
}

export async function deleteTimeline(id: string) {
  await api.deleteTimeline(id)
  timelines.update((list) => list.filter((t) => t.id !== id))
  currentTimeline.update((cur) => (cur && cur.id === id ? null : cur))
}

export async function editEvent(
  id: string,
  updates: { title?: string; date?: string; description?: string | null; pageId?: string | null },
) {
  const apiUpdates: { title?: string; date?: string; description?: string; pageId?: string } = {}
  if (updates.title !== undefined) apiUpdates.title = updates.title
  if (updates.date !== undefined) apiUpdates.date = updates.date
  if (updates.description !== undefined) apiUpdates.description = updates.description ?? ''
  if (updates.pageId !== undefined) apiUpdates.pageId = updates.pageId ?? ''
  const updated = await api.updateTimelineEvent(id, apiUpdates)
  currentEvents.update((events) => events.map((e) => (e.id === id ? updated : e)).sort((a, b) => a.date.localeCompare(b.date)))
  return updated
}

export async function removeEvent(id: string, timelineId: string) {
  await api.deleteTimelineEvent(id)
  currentEvents.update((events) => events.filter((e) => e.id !== id))
}
