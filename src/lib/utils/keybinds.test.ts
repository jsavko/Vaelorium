import { describe, it, expect } from 'vitest'
import { parseKeybind, matchesKeybind } from './keybinds'

describe('parseKeybind', () => {
  it('parses Ctrl+K', () => {
    const result = parseKeybind('Ctrl+K')
    expect(result).toEqual({ ctrl: true, shift: false, alt: false, key: 'k' })
  })

  it('parses Ctrl+Shift+N', () => {
    const result = parseKeybind('Ctrl+Shift+N')
    expect(result).toEqual({ ctrl: true, shift: true, alt: false, key: 'n' })
  })

  it('parses Alt+P', () => {
    const result = parseKeybind('Alt+P')
    expect(result).toEqual({ ctrl: false, shift: false, alt: true, key: 'p' })
  })

  it('parses Ctrl+\\', () => {
    const result = parseKeybind('Ctrl+\\')
    expect(result).toEqual({ ctrl: true, shift: false, alt: false, key: '\\' })
  })

  it('parses Cmd as Ctrl', () => {
    const result = parseKeybind('Cmd+K')
    expect(result).toEqual({ ctrl: true, shift: false, alt: false, key: 'k' })
  })

  it('handles single key', () => {
    const result = parseKeybind('Escape')
    expect(result).toEqual({ ctrl: false, shift: false, alt: false, key: 'Escape' })
  })
})

describe('matchesKeybind', () => {
  function makeEvent(overrides: Partial<KeyboardEvent>): KeyboardEvent {
    return {
      key: '',
      ctrlKey: false,
      metaKey: false,
      shiftKey: false,
      altKey: false,
      ...overrides,
    } as KeyboardEvent
  }

  it('matches Ctrl+K', () => {
    expect(matchesKeybind(makeEvent({ key: 'k', ctrlKey: true }), 'Ctrl+K')).toBe(true)
  })

  it('matches Meta+K (Mac Cmd)', () => {
    expect(matchesKeybind(makeEvent({ key: 'k', metaKey: true }), 'Ctrl+K')).toBe(true)
  })

  it('does not match without modifier', () => {
    expect(matchesKeybind(makeEvent({ key: 'k' }), 'Ctrl+K')).toBe(false)
  })

  it('does not match wrong key', () => {
    expect(matchesKeybind(makeEvent({ key: 'j', ctrlKey: true }), 'Ctrl+K')).toBe(false)
  })

  it('matches Ctrl+Shift+N', () => {
    expect(matchesKeybind(makeEvent({ key: 'N', ctrlKey: true, shiftKey: true }), 'Ctrl+Shift+N')).toBe(true)
  })

  it('does not match Ctrl+N when expecting Ctrl+Shift+N', () => {
    expect(matchesKeybind(makeEvent({ key: 'n', ctrlKey: true }), 'Ctrl+Shift+N')).toBe(false)
  })

  it('matches Ctrl+\\', () => {
    expect(matchesKeybind(makeEvent({ key: '\\', ctrlKey: true }), 'Ctrl+\\')).toBe(true)
  })

  it('matches case-insensitively for single chars', () => {
    expect(matchesKeybind(makeEvent({ key: 'K', ctrlKey: true }), 'Ctrl+K')).toBe(true)
    expect(matchesKeybind(makeEvent({ key: 'k', ctrlKey: true }), 'Ctrl+K')).toBe(true)
  })
})
