import type { JSONContent } from '@tiptap/core'

export function extractPlainText(doc: JSONContent): string {
  const parts: string[] = []

  function walk(node: JSONContent) {
    if (node.text) {
      parts.push(node.text)
    }
    if (node.content) {
      for (const child of node.content) {
        walk(child)
      }
      // Add newline after block-level elements
      if (['paragraph', 'heading', 'blockquote', 'listItem'].includes(node.type ?? '')) {
        parts.push('\n')
      }
    }
  }

  walk(doc)
  return parts.join('').trim()
}
