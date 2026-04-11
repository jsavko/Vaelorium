import { Extension } from '@tiptap/core'
import Suggestion from '@tiptap/suggestion'
import { PluginKey } from '@tiptap/pm/state'
import type { Editor } from '@tiptap/core'

export interface SlashCommandItem {
  label: string
  icon: string
  action: (editor: Editor) => void
}

export const slashCommandItems: SlashCommandItem[] = [
  { label: 'Heading 1', icon: 'H1', action: (editor) => editor.chain().focus().toggleHeading({ level: 1 }).run() },
  { label: 'Heading 2', icon: 'H2', action: (editor) => editor.chain().focus().toggleHeading({ level: 2 }).run() },
  { label: 'Heading 3', icon: 'H3', action: (editor) => editor.chain().focus().toggleHeading({ level: 3 }).run() },
  { label: 'Bullet List', icon: '•', action: (editor) => editor.chain().focus().toggleBulletList().run() },
  { label: 'Ordered List', icon: '1.', action: (editor) => editor.chain().focus().toggleOrderedList().run() },
  { label: 'Blockquote', icon: '"', action: (editor) => editor.chain().focus().toggleBlockquote().run() },
  { label: 'Code Block', icon: '<>', action: (editor) => editor.chain().focus().toggleCodeBlock().run() },
  { label: 'Divider', icon: '—', action: (editor) => editor.chain().focus().setHorizontalRule().run() },
  { label: 'Table', icon: '▦', action: (editor) => editor.chain().focus().insertTable({ rows: 3, cols: 3 }).run() },
  { label: 'Page Embed', icon: '📄', action: (editor) => {
    // Dispatch event for Editor.svelte to handle page selection
    window.dispatchEvent(new CustomEvent('vaelorium:embed-request', { detail: { editor } }))
  }},
]

export type SlashMenuRenderer = {
  onStart: (props: { items: SlashCommandItem[]; command: (item: SlashCommandItem) => void; clientRect: (() => DOMRect | null) | null }) => void
  onUpdate: (props: { items: SlashCommandItem[]; command: (item: SlashCommandItem) => void; clientRect: (() => DOMRect | null) | null }) => void
  onExit: () => void
}

let menuRenderer: SlashMenuRenderer | null = null

export function setSlashMenuRenderer(renderer: SlashMenuRenderer) {
  menuRenderer = renderer
}

export const SlashCommands = Extension.create({
  name: 'slashCommands',

  addOptions() {
    return {
      suggestion: {
        char: '/',
        startOfLine: false,
        command: ({ editor, range, props }: { editor: Editor; range: any; props: SlashCommandItem }) => {
          // Delete the `/` trigger text
          editor.chain().focus().deleteRange(range).run()
          // Execute the selected command
          props.action(editor)
        },
        items: ({ query }: { query: string }) => {
          return slashCommandItems.filter((item) =>
            item.label.toLowerCase().includes(query.toLowerCase())
          )
        },
        render: () => {
          return {
            onStart(props: any) {
              menuRenderer?.onStart({
                items: props.items,
                command: (item: SlashCommandItem) => props.command(item),
                clientRect: props.clientRect,
              })
            },
            onUpdate(props: any) {
              menuRenderer?.onUpdate({
                items: props.items,
                command: (item: SlashCommandItem) => props.command(item),
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
        pluginKey: new PluginKey('slashCommandsSuggestion'),
        ...this.options.suggestion,
      }),
    ]
  },
})
