import Image from '@tiptap/extension-image'

export type ImageAlignment = 'center' | 'left' | 'right'

export const FloatImage = Image.extend({
  addAttributes() {
    return {
      ...this.parent?.(),
      alignment: {
        default: 'center',
        parseHTML: (element) => element.getAttribute('data-alignment') || 'center',
        renderHTML: (attributes) => {
          return {
            'data-alignment': attributes.alignment,
            class: `img-align-${attributes.alignment}`,
          }
        },
      },
    }
  },

  addCommands() {
    return {
      ...this.parent?.(),
      setImageAlignment:
        (alignment: ImageAlignment) =>
        ({ commands }: { commands: any }) => {
          return commands.updateAttributes(this.name, { alignment })
        },
    }
  },
})
