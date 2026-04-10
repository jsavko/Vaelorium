import { writable } from 'svelte/store'

export interface Toast {
  id: string
  message: string
  type: 'success' | 'error' | 'info'
}

export const toasts = writable<Toast[]>([])

let counter = 0

export function showToast(message: string, type: Toast['type'] = 'info') {
  const id = `toast-${counter++}`
  toasts.update((t) => [...t, { id, message, type }])
  setTimeout(() => {
    toasts.update((t) => t.filter((toast) => toast.id !== id))
  }, 3000)
}
