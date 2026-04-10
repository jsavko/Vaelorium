import StarterKit from '@tiptap/starter-kit'
import { Table } from '@tiptap/extension-table'
import Image from '@tiptap/extension-image'
import Link from '@tiptap/extension-link'
import Placeholder from '@tiptap/extension-placeholder'
import Typography from '@tiptap/extension-typography'
import CharacterCount from '@tiptap/extension-character-count'
import Collaboration from '@tiptap/extension-collaboration'
import type { Doc as YDoc } from 'yjs'

export function createEditorExtensions(ydoc: YDoc) {
  return [
    StarterKit.configure({
      heading: { levels: [1, 2, 3] },
      history: false, // Disabled — Yjs handles undo/redo
    }),
    Table.configure({ resizable: true }),
    Image.configure({ inline: false }),
    Link.configure({
      openOnClick: false,
      HTMLAttributes: { class: 'wiki-link' },
    }),
    Placeholder.configure({
      placeholder: 'Start writing...',
    }),
    Typography,
    CharacterCount,
    Collaboration.configure({
      document: ydoc,
    }),
  ]
}
