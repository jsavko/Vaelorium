import StarterKit from '@tiptap/starter-kit'
import { Table } from '@tiptap/extension-table'
import { TableRow } from '@tiptap/extension-table-row'
import { TableHeader } from '@tiptap/extension-table-header'
import { TableCell } from '@tiptap/extension-table-cell'
import { FloatImage } from './FloatImage'
import { ImagePastePlugin } from './ImagePastePlugin'
import Link from '@tiptap/extension-link'
import Placeholder from '@tiptap/extension-placeholder'
import Typography from '@tiptap/extension-typography'
import CharacterCount from '@tiptap/extension-character-count'
import Collaboration from '@tiptap/extension-collaboration'
import { SlashCommands } from './SlashCommands'
import { MentionExtension } from './MentionExtension'
import { WikiLinkSyntax } from './WikiLinkSyntax'
import { PageEmbedNode } from './PageEmbedExtension'
import { CalloutBlock } from './CalloutExtension'
import type { Doc as YDoc } from 'yjs'

export function createEditorExtensions(ydoc: YDoc) {
  return [
    StarterKit.configure({
      heading: { levels: [1, 2, 3] },
      undoRedo: false, // Yjs Collaboration has its own undo
    }),
    Table.configure({ resizable: true }),
    TableRow,
    TableHeader,
    TableCell,
    FloatImage.configure({ inline: false }),
    ImagePastePlugin,
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
    SlashCommands,
    MentionExtension,
    WikiLinkSyntax,
    PageEmbedNode,
    CalloutBlock,
  ]
}
