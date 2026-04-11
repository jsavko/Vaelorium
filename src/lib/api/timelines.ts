import { callCommand } from './bridge'

export interface Timeline {
  id: string
  name: string
  description: string | null
  sort_order: number
  created_at: string
  updated_at: string
}

export interface TimelineEvent {
  id: string
  timeline_id: string
  title: string
  description: string | null
  date: string
  end_date: string | null
  page_id: string | null
  color: string | null
  sort_order: number
  created_at: string
}

export async function createTimeline(name: string, description?: string | null): Promise<Timeline> {
  return callCommand('create_timeline', { name, description })
}

export async function listTimelines(): Promise<Timeline[]> {
  return callCommand('list_timelines')
}

export async function deleteTimeline(id: string): Promise<void> {
  return callCommand('delete_timeline', { id })
}

export async function createTimelineEvent(
  timelineId: string,
  title: string,
  date: string,
  description?: string | null,
  endDate?: string | null,
  pageId?: string | null,
  color?: string | null,
): Promise<TimelineEvent> {
  return callCommand('create_timeline_event', { timelineId, title, date, description, endDate, pageId, color })
}

export async function updateTimelineEvent(
  id: string,
  updates: { title?: string; date?: string; description?: string; endDate?: string; pageId?: string; color?: string },
): Promise<TimelineEvent> {
  return callCommand('update_timeline_event', { id, ...updates })
}

export async function deleteTimelineEvent(id: string): Promise<void> {
  return callCommand('delete_timeline_event', { id })
}

export async function getTimelineEvents(timelineId: string): Promise<TimelineEvent[]> {
  return callCommand('get_timeline_events', { timelineId })
}
