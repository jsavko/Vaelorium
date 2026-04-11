import { Extension } from '@tiptap/core'
import { Plugin, PluginKey } from '@tiptap/pm/state'
import { uploadFileObject, getImageUrl } from '../api/images'

/**
 * TipTap extension that handles pasting images from the clipboard.
 * Intercepts paste events containing image data, uploads the image,
 * and inserts it into the editor.
 */
export const ImagePastePlugin = Extension.create({
  name: 'imagePaste',

  addProseMirrorPlugins() {
    const editor = this.editor

    return [
      new Plugin({
        key: new PluginKey('imagePaste'),
        props: {
          handlePaste(_view, event) {
            const items = event.clipboardData?.items
            if (!items) return false

            for (const item of items) {
              if (item.type.startsWith('image/')) {
                event.preventDefault()
                const file = item.getAsFile()
                if (file) {
                  // Upload and insert asynchronously
                  ;(async () => {
                    try {
                      const info = await uploadFileObject(file)
                      const url = await getImageUrl(info.id)
                      editor.chain().focus().setImage({ src: url }).run()
                    } catch (e) {
                      console.warn('Failed to paste image:', e)
                    }
                  })()
                }
                return true
              }
            }

            return false
          },
        },
      }),
    ]
  },
})
