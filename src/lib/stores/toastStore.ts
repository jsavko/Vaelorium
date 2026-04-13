import { get, writable } from 'svelte/store'

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

/**
 * Global Ctrl/Cmd+Z binding that fires the most-recent action-toast's
 * onClick while any action-toast is visible. Skipped when the user is
 * typing in an editable surface so TipTap's text-level undo keeps
 * working. Safe to install once per app lifetime.
 */
if (typeof window !== 'undefined') {
  window.addEventListener('keydown', (e) => {
    if (!(e.ctrlKey || e.metaKey) || e.shiftKey) return
    if (e.key !== 'z' && e.key !== 'Z') return
    // Don't hijack text-level undo inside editors, inputs, or textareas.
    const t = e.target as HTMLElement | null
    if (t) {
      if (t.isContentEditable) return
      const tag = t.tagName
      if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return
      if (t.closest?.('.ProseMirror')) return
    }
    const current = get(toasts)
    // Most-recent-first: pop the latest action toast.
    for (let i = current.length - 1; i >= 0; i--) {
      const toast = current[i]
      if (toast.action) {
        e.preventDefault()
        toast.action.onClick()
        dismissToast(toast.id)
        return
      }
    }
  })
}
