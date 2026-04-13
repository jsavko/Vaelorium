import { Extension } from '@tiptap/core'
import Suggestion from '@tiptap/suggestion'
import { PluginKey } from '@tiptap/pm/state'
import type { Editor } from '@tiptap/core'
import type { PageTreeNode } from '../api/pages'
import { get } from 'svelte/store'
import { pageTree } from '../stores/pageStore'
import { getMentionMenuRenderer } from './MentionExtension'

/**
 * [[wiki link]] syntax — triggers the same mention dropdown as @,
 * but activated by typing [[ instead.
 */
export const WikiLinkSyntax = Extension.create({
  name: 'wikiLinkSyntax',

  addOptions() {
    return {
      suggestion: {
        char: '[[',
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
          const tree = get(pageTree)
          if (!query) return tree.slice(0, 8)
          return tree
            .filter((p) => p.title.toLowerCase().includes(query.toLowerCase()))
            .slice(0, 8)
        },
        render: () => {
          return {
            onStart(props: any) {
              getMentionMenuRenderer()?.onStart({
                items: props.items,
                command: (item) => props.command(item as unknown as PageTreeNode),
                clientRect: props.clientRect,
              })
            },
            onUpdate(props: any) {
              getMentionMenuRenderer()?.onUpdate({
                items: props.items,
                command: (item) => props.command(item as unknown as PageTreeNode),
                clientRect: props.clientRect,
              })
            },
            onKeyDown(props: any) {
              if (props.event.key === 'Escape') {
                getMentionMenuRenderer()?.onExit()
                return true
              }
              return false
            },
            onExit() {
              getMentionMenuRenderer()?.onExit()
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
        pluginKey: new PluginKey('wikiLinkSyntaxSuggestion'),
        ...this.options.suggestion,
      }),
    ]
  },
})
