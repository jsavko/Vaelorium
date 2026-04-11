import { callCommand } from './bridge'
import { isTauri } from './bridge'

export interface ImageInfo {
  id: string
  filename: string
  mime_type: string
  created_at: string
}

export interface ImageData {
  id: string
  filename: string
  mime_type: string
  data: number[]
}

export async function uploadImage(path: string): Promise<ImageInfo> {
  return callCommand('upload_image', { path })
}

export async function uploadImageData(filename: string, data: number[]): Promise<ImageInfo> {
  return callCommand('upload_image_data', { filename, data })
}

export async function getImage(id: string): Promise<ImageData> {
  return callCommand('get_image', { id })
}

export async function deleteImage(id: string): Promise<void> {
  return callCommand('delete_image', { id })
}

export async function listImages(): Promise<ImageInfo[]> {
  return callCommand('list_images')
}

/**
 * Get a displayable URL for an image stored in the DB.
 * Fetches the blob and converts to a data URL.
 */
export async function getImageUrl(id: string): Promise<string> {
  const img = await getImage(id)
  const bytes = new Uint8Array(img.data)
  const blob = new Blob([bytes], { type: img.mime_type })
  return URL.createObjectURL(blob)
}

/**
 * Upload a File object (from browser file input or drag-and-drop).
 * Reads as array buffer and sends to backend.
 */
export async function uploadFileObject(file: File): Promise<ImageInfo> {
  const buffer = await file.arrayBuffer()
  const data = Array.from(new Uint8Array(buffer))
  return uploadImageData(file.name, data)
}

/**
 * Open a file picker and upload the selected image.
 * Returns null if the user cancels.
 */
export async function pickAndUploadImage(): Promise<ImageInfo | null> {
  if (isTauri) {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog')
      const result = await open({
        filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg', 'bmp'] }],
        multiple: false,
      })
      if (result) {
        return uploadImage(result as string)
      }
    } catch (e) {
      console.warn('Tauri dialog failed, falling back to browser:', e)
    }
    return null
  } else {
    // Browser: use hidden file input
    return new Promise((resolve) => {
      const input = document.createElement('input')
      input.type = 'file'
      input.accept = 'image/*'
      input.onchange = async () => {
        const file = input.files?.[0]
        if (file) {
          const info = await uploadFileObject(file)
          resolve(info)
        } else {
          resolve(null)
        }
      }
      input.click()
    })
  }
}
