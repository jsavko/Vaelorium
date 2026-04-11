import { writable, get } from 'svelte/store'

export interface KeyBinding {
  id: string
  label: string
  keys: string
  defaultKeys: string
}

export interface AppSettings {
  keybinds: KeyBinding[]
  appearance: {
    fontSize: number
    theme: string
  }
}

const defaultKeybinds: KeyBinding[] = [
  { id: 'search', label: 'Search', keys: 'Ctrl+K', defaultKeys: 'Ctrl+K' },
  { id: 'new-page', label: 'New Page', keys: 'Ctrl+N', defaultKeys: 'Ctrl+N' },
  { id: 'toggle-details', label: 'Toggle Details', keys: 'Ctrl+\\', defaultKeys: 'Ctrl+\\' },
  { id: 'save', label: 'Save', keys: 'Ctrl+S', defaultKeys: 'Ctrl+S' },
  { id: 'bold', label: 'Bold', keys: 'Ctrl+B', defaultKeys: 'Ctrl+B' },
  { id: 'italic', label: 'Italic', keys: 'Ctrl+I', defaultKeys: 'Ctrl+I' },
]

const defaultSettings: AppSettings = {
  keybinds: defaultKeybinds,
  appearance: {
    fontSize: 16,
    theme: 'dark-library',
  },
}

function loadSettings(): AppSettings {
  try {
    const stored = localStorage.getItem('vaelorium-settings')
    if (stored) {
      const parsed = JSON.parse(stored)
      return { ...defaultSettings, ...parsed }
    }
  } catch {}
  return { ...defaultSettings }
}

function saveSettings(settings: AppSettings) {
  try {
    localStorage.setItem('vaelorium-settings', JSON.stringify(settings))
  } catch {}
}

export const settings = writable<AppSettings>(loadSettings())

export function updateKeybind(id: string, newKeys: string) {
  settings.update((s) => {
    const kb = s.keybinds.find((k) => k.id === id)
    if (kb) kb.keys = newKeys
    saveSettings(s)
    return s
  })
}

export function resetKeybinds() {
  settings.update((s) => {
    s.keybinds = defaultKeybinds.map((k) => ({ ...k }))
    saveSettings(s)
    return s
  })
}

export function updateAppearance(updates: Partial<AppSettings['appearance']>) {
  settings.update((s) => {
    s.appearance = { ...s.appearance, ...updates }
    saveSettings(s)
    return s
  })
}

export function getKeybind(id: string): string {
  const s = get(settings)
  return s.keybinds.find((k) => k.id === id)?.keys || ''
}
