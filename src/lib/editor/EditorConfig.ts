import StarterKit from '@tiptap/starter-kit'
import { Table } from '@tiptap/extension-table'
import { TableRow } from '@tiptap/extension-table-row'
import { TableHeader } from '@tiptap/extension-table-header'
import { TableCell } from '@tiptap/extension-table-cell'
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
      history: false,
    }),
    Table.configure({ resizable: true }),
    TableRow,
    TableHeader,
    TableCell,
    Image.configure({ inline: false }),
    Link.configure({
      openOnClick: false,
    }),
    Placeholder.configure({
      placeholder: 'Start writing...',
    }),
    Typography,
    CharacterCount,
    Collaboration.configure({
      document: ydoc,
      field: 'content',
    }),
  ]
}
