import { Extension } from '@tiptap/core'
import Suggestion from '@tiptap/suggestion'
import { PluginKey } from '@tiptap/pm/state'
import type { Editor } from '@tiptap/core'
import type { PageTreeNode } from '../api/pages'
import { callCommand } from '../api/bridge'
import { get } from 'svelte/store'
import { pageTree } from '../stores/pageStore'

export type MentionMenuRenderer = {
  onStart: (props: { items: PageTreeNode[]; command: (item: PageTreeNode) => void; clientRect: (() => DOMRect | null) | null }) => void
  onUpdate: (props: { items: PageTreeNode[]; command: (item: PageTreeNode) => void; clientRect: (() => DOMRect | null) | null }) => void
  onExit: () => void
}

let menuRenderer: MentionMenuRenderer | null = null

export function setMentionMenuRenderer(renderer: MentionMenuRenderer) {
  menuRenderer = renderer
}

export const MentionExtension = Extension.create({
  name: 'mentionSuggestion',

  addOptions() {
    return {
      suggestion: {
        char: '@',
        command: ({ editor, range, props }: { editor: Editor; range: any; props: PageTreeNode }) => {
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
                    href: `#page:${props.id}`,
                    class: 'wiki-link-inline',
                  },
                },
              ],
              text: props.title,
            })
            .run()
        },
        items: ({ query }: { query: string }) => {
          // Use the reactive store synchronously — pageTree is already loaded
          const tree = get(pageTree)
          if (!query) return tree.slice(0, 8)
          return tree
            .filter((p) => p.title.toLowerCase().includes(query.toLowerCase()))
            .slice(0, 8)
        },
        render: () => {
          return {
            onStart(props: any) {
              menuRenderer?.onStart({
                items: props.items,
                command: (item: PageTreeNode) => props.command(item),
                clientRect: props.clientRect,
              })
            },
            onUpdate(props: any) {
              menuRenderer?.onUpdate({
                items: props.items,
                command: (item: PageTreeNode) => props.command(item),
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
