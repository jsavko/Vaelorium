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

export interface EditorExtensionOptions {
  /** Read-only preview mode (e.g. VersionHistory's version preview).
   *  Skips the interactive extensions that register global suggestion
   *  handlers or reference singleton UI elements owned by the live
   *  editor — SlashCommands, MentionExtension, WikiLinkSyntax. Preview
   *  only needs to RENDER existing content, not support typing/commands. */
  forPreview?: boolean
}

export function createEditorExtensions(ydoc: YDoc, opts: EditorExtensionOptions = {}) {
  const base = [
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
    PageEmbedNode,
    CalloutBlock,
  ]
  if (opts.forPreview) return base
  return [
    ...base,
    SlashCommands,
    MentionExtension,
    WikiLinkSyntax,
  ]
}
