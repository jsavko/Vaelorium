import { Extension } from '@tiptap/core'
import { Plugin, PluginKey } from '@tiptap/pm/state'
import { uploadFileObject, uploadImageData, getImageUrl } from '../api/images'

/**
 * Download an image from a URL and return it as a File-like object.
 */
async function fetchImageAsData(url: string): Promise<{ data: number[]; filename: string } | null> {
  try {
    const response = await fetch(url)
    if (!response.ok) return null
    const contentType = response.headers.get('content-type') || 'image/png'
    const buffer = await response.arrayBuffer()
    const ext = contentType.split('/')[1]?.split(';')[0] || 'png'
    return {
      data: Array.from(new Uint8Array(buffer)),
      filename: `pasted-image.${ext}`,
    }
  } catch {
    return null
  }
}

/**
 * TipTap extension that handles pasting images from the clipboard.
 * Handles both raw image data (screenshots) and HTML with img tags (copied from websites).
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

            // Check for raw image data first (screenshots, copied image files)
            for (const item of items) {
              if (item.type.startsWith('image/')) {
                event.preventDefault()
                const file = item.getAsFile()
                if (file) {
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

            // Check for HTML with an img tag (right-click → copy image from website)
            const html = event.clipboardData?.getData('text/html')
            if (html) {
              const match = html.match(/<img[^>]+src=["']([^"']+)["']/i)
              if (match && match[1]) {
                const imgUrl = match[1]
                // Only handle if it looks like an actual image URL
                if (imgUrl.startsWith('http') || imgUrl.startsWith('data:image')) {
                  event.preventDefault()
                  ;(async () => {
                    try {
                      if (imgUrl.startsWith('data:image')) {
                        // Data URL — decode and upload
                        const response = await fetch(imgUrl)
                        const blob = await response.blob()
                        const file = new File([blob], 'pasted-image.png', { type: blob.type })
                        const info = await uploadFileObject(file)
                        const url = await getImageUrl(info.id)
                        editor.chain().focus().setImage({ src: url }).run()
                      } else {
                        // Remote URL — try to download and store locally
                        const result = await fetchImageAsData(imgUrl)
                        if (result) {
                          const info = await uploadImageData(result.filename, result.data)
                          const url = await getImageUrl(info.id)
                          editor.chain().focus().setImage({ src: url }).run()
                        } else {
                          // Fallback: insert as external URL
                          editor.chain().focus().setImage({ src: imgUrl }).run()
                        }
                      }
                    } catch (e) {
                      console.warn('Failed to paste image from HTML:', e)
                      // Fallback: insert the raw URL
                      editor.chain().focus().setImage({ src: imgUrl }).run()
                    }
                  })()
                  return true
                }
              }
            }

            return false
          },
        },
      }),
    ]
  },
})
