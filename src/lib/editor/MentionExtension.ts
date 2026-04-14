import { Extension } from '@tiptap/core'
import Suggestion from '@tiptap/suggestion'
import { PluginKey } from '@tiptap/pm/state'
import type { Editor } from '@tiptap/core'
import type { PageTreeNode } from '../api/pages'
import { get } from 'svelte/store'
import { pageTree } from '../stores/pageStore'
import { maps } from '../stores/mapStore'
import { timelines } from '../stores/timelineStore'

export interface MentionItem {
  id: string
  title: string
  category: 'page' | 'map' | 'timeline'
  entity_type_id?: string | null
  icon?: string | null
  color?: string
}

export type MentionMenuRenderer = {
  onStart: (props: { items: MentionItem[]; command: (item: MentionItem) => void; clientRect: (() => DOMRect | null) | null }) => void
  onUpdate: (props: { items: MentionItem[]; command: (item: MentionItem) => void; clientRect: (() => DOMRect | null) | null }) => void
  onExit: () => void
}

let menuRenderer: MentionMenuRenderer | null = null

export function setMentionMenuRenderer(renderer: MentionMenuRenderer) {
  menuRenderer = renderer
}

export function getMentionMenuRenderer(): MentionMenuRenderer | null {
  return menuRenderer
}

export const MentionExtension = Extension.create({
  name: 'mentionSuggestion',

  addOptions() {
    return {
      suggestion: {
        char: '@',
        command: ({ editor, range, props }: { editor: Editor; range: any; props: MentionItem }) => {
          const href = props.category === 'map' ? `#map:${props.id}`
            : props.category === 'timeline' ? `#timeline:${props.id}`
            : `#page:${props.id}`
          editor
            .chain()
            .focus()
            .deleteRange(range)
            .insertContent({
              type: 'text',
              marks: [
                {
                  type: 'link',
                  attrs: {
                    href,
                    class: 'wiki-link-inline',
                  },
                },
              ],
              text: props.title,
            })
            .unsetMark('link')
            .run()
        },
        items: ({ query }: { query: string }): MentionItem[] => {
          const q = query.toLowerCase()
          const tree = get(pageTree)
          const mapList = get(maps)
          const timelineList = get(timelines)

          const pageItems: MentionItem[] = tree.map((p) => ({
            id: p.id, title: p.title, category: 'page' as const,
            entity_type_id: p.entity_type_id, icon: p.icon,
          }))
          const mapItems: MentionItem[] = mapList.map((m) => ({
            id: m.id, title: m.title, category: 'map' as const,
            color: '#4A8C6A',
          }))
          const tlItems: MentionItem[] = timelineList.map((t) => ({
            id: t.id, title: t.name, category: 'timeline' as const,
            color: '#C8A55C',
          }))

          const all = [...pageItems, ...mapItems, ...tlItems]
          if (!q) return all.slice(0, 10)
          return all
            .filter((item) => item.title.toLowerCase().includes(q))
            .slice(0, 10)
        },
        render: () => {
          return {
            onStart(props: any) {
              menuRenderer?.onStart({
                items: props.items,
                command: (item: MentionItem) => props.command(item),
                clientRect: props.clientRect,
              })
            },
            onUpdate(props: any) {
              menuRenderer?.onUpdate({
                items: props.items,
                command: (item: MentionItem) => props.command(item),
                clientRect: props.clientRect,
              })
            },
            onKeyDown(props: any) {
              if (props.event.key === 'Escape') {
                menuRenderer?.onExit()
                return true
              }
              return false
            },
            onExit() {
              menuRenderer?.onExit()
            },
          }
        },
      },
    }
  },

  addProseMirrorPlugins() {
    return [
      Suggestion({
        editor: this.editor,
        pluginKey: new PluginKey('mentionSuggestion'),
        ...this.options.suggestion,
      }),
    ]
  },
})
