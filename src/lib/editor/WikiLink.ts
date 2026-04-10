import { Mark, mergeAttributes } from '@tiptap/core'

export interface WikiLinkOptions {
  HTMLAttributes: Record<string, any>
  onClickLink?: (pageId: string) => void
}

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    wikiLink: {
      setWikiLink: (attributes: { pageId: string; label: string }) => ReturnType
      unsetWikiLink: () => ReturnType
    }
  }
}

export const WikiLink = Mark.create<WikiLinkOptions>({
  name: 'wikiLink',

  addOptions() {
    return {
      HTMLAttributes: {},
      onClickLink: undefined,
    }
  },

  addAttributes() {
    return {
      pageId: {
        default: null,
        parseHTML: (element) => element.getAttribute('data-page-id'),
        renderHTML: (attributes) => ({
          'data-page-id': attributes.pageId,
        }),
      },
      label: {
        default: null,
        parseHTML: (element) => element.getAttribute('data-label'),
        renderHTML: (attributes) => ({
          'data-label': attributes.label,
        }),
      },
    }
  },

  parseHTML() {
    return [{ tag: 'span[data-page-id]' }]
  },

  renderHTML({ HTMLAttributes }) {
    return [
      'span',
      mergeAttributes(this.options.HTMLAttributes, HTMLAttributes, {
        class: 'wiki-link-mark',
        style: 'color: var(--color-accent-gold); cursor: pointer; text-decoration: none;',
      }),
      0,
    ]
  },

  addCommands() {
    return {
      setWikiLink:
        (attributes) =>
        ({ commands }) => {
          return commands.setMark(this.name, attributes)
        },
      unsetWikiLink:
        () =>
        ({ commands }) => {
          return commands.unsetMark(this.name)
        },
    }
  },
})
