import { writable } from 'svelte/store'

export interface ToastAction {
  label: string
  onClick: () => void
}

export interface Toast {
  id: string
  message: string
  type: 'success' | 'error' | 'info'
  action?: ToastAction
}

export const toasts = writable<Toast[]>([])

let counter = 0

export function showToast(
  message: string,
  type: Toast['type'] = 'info',
  opts?: { action?: ToastAction; durationMs?: number },
) {
  const id = `toast-${counter++}`
  toasts.update((t) => [...t, { id, message, type, action: opts?.action }])
  // Default 3s for plain toasts, 5s for action toasts so users have time.
  const duration = opts?.durationMs ?? (opts?.action ? 5000 : 3000)
  setTimeout(() => {
    toasts.update((t) => t.filter((toast) => toast.id !== id))
  }, duration)
}

export function dismissToast(id: string) {
  toasts.update((t) => t.filter((toast) => toast.id !== id))
}
