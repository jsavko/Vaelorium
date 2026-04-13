import { Node, mergeAttributes, type RawCommands } from '@tiptap/core'

export type CalloutType = 'info' | 'warning' | 'note'

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    callout: {
      setCallout: (type?: CalloutType) => ReturnType
    }
  }
}

export const CalloutBlock = Node.create({
  name: 'callout',
  group: 'block',
  content: 'block+',
  defining: true,

  addAttributes() {
    return {
      type: {
        default: 'info',
        parseHTML: (el) => el.getAttribute('data-callout-type') || 'info',
        renderHTML: (attrs) => ({ 'data-callout-type': attrs.type }),
      },
    }
  },

  parseHTML() {
    return [{ tag: 'div[data-callout]' }]
  },

  renderHTML({ HTMLAttributes }) {
    return [
      'div',
      mergeAttributes(HTMLAttributes, {
        'data-callout': '',
        class: `callout callout-${HTMLAttributes['data-callout-type'] || 'info'}`,
      }),
      0,
    ]
  },

  addCommands(): Partial<RawCommands> {
    return {
      setCallout:
        (type: CalloutType = 'info') =>
        ({ commands }: { commands: any }) => {
          return commands.wrapIn(this.name, { type })
        },
    }
  },
})
