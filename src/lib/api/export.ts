import { callCommand } from './bridge'

export async function exportTomeJson(): Promise<string> {
  return callCommand('export_tome_json')
}

export async function exportTomeMarkdown(path: string): Promise<void> {
  return callCommand('export_tome_markdown', { path })
}

export async function importMarkdownFolder(path: string): Promise<{ pages_imported: number; errors: string[] }> {
  return callCommand('import_markdown_folder', { path })
}

export async function importJson(json: string): Promise<{ pages_imported: number; errors: string[] }> {
  return callCommand('import_json', { json })
}
