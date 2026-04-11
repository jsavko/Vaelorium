export interface ParsedKeybind {
  ctrl: boolean
  shift: boolean
  alt: boolean
  key: string // lowercase single char or key name like 'Escape', 'Enter'
}

/**
 * Parse a keybind string like "Ctrl+K" or "Ctrl+Shift+N" into a structured object.
 */
export function parseKeybind(combo: string): ParsedKeybind {
  const parts = combo.split('+')
  const result: ParsedKeybind = { ctrl: false, shift: false, alt: false, key: '' }

  for (const part of parts) {
    const trimmed = part.trim()
    const lower = trimmed.toLowerCase()
    if (lower === 'ctrl' || lower === 'cmd' || lower === 'meta') {
      result.ctrl = true
    } else if (lower === 'shift') {
      result.shift = true
    } else if (lower === 'alt' || lower === 'option') {
      result.alt = true
    } else if (lower === 'space' || part === ' ') {
      result.key = ' '
    } else {
      result.key = trimmed.length === 1 ? trimmed.toLowerCase() : trimmed
    }
  }

  return result
}

/**
 * Check if a keyboard event matches a keybind string.
 * Treats Meta (Cmd) as equivalent to Ctrl for cross-platform support.
 */
export function matchesKeybind(event: KeyboardEvent, combo: string): boolean {
  const parsed = parseKeybind(combo)

  const ctrlOrMeta = event.ctrlKey || event.metaKey
  if (parsed.ctrl !== ctrlOrMeta) return false
  if (parsed.shift !== event.shiftKey) return false
  if (parsed.alt !== event.altKey) return false

  // Compare key — handle special cases
  const eventKey = event.key.length === 1 ? event.key.toLowerCase() : event.key
  return eventKey === parsed.key
}
