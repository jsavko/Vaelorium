<script lang="ts">
  import { isTauri } from '../api/bridge'
  import { exportTomeJson, exportTomeMarkdown, importJson, importMarkdownFolder } from '../api/export'
  import { showToast } from '../stores/toastStore'
  import { loadPageTree } from '../stores/pageStore'

  async function handleExportJson() {
    try {
      const json = await exportTomeJson()
      if (isTauri) {
        const { save } = await import('@tauri-apps/plugin-dialog')
        const path = await save({ defaultPath: 'tome-export.json', filters: [{ name: 'JSON', extensions: ['json'] }] })
        if (path) {
          const { writeTextFile } = await import('@tauri-apps/plugin-fs')
          await writeTextFile(path as string, json)
          showToast('Exported as JSON', 'success')
        }
      } else {
        const blob = new Blob([json], { type: 'application/json' })
        const url = URL.createObjectURL(blob)
        const a = document.createElement('a')
        a.href = url; a.download = 'tome-export.json'; a.click()
        URL.revokeObjectURL(url)
        showToast('Exported as JSON', 'success')
      }
    } catch (e: any) { showToast('Export failed: ' + e.message, 'error') }
  }

  async function handleExportMarkdown() {
    if (isTauri) {
      try {
        const { open } = await import('@tauri-apps/plugin-dialog')
        const path = await open({ directory: true })
        if (path) {
          await exportTomeMarkdown(path as string)
          showToast('Exported as Markdown', 'success')
        }
      } catch (e: any) { showToast('Export failed: ' + e.message, 'error') }
    }
  }

  async function handleImportJson() {
    try {
      if (isTauri) {
        const { open } = await import('@tauri-apps/plugin-dialog')
        const path = await open({ filters: [{ name: 'JSON', extensions: ['json'] }] })
        if (path) {
          const { readTextFile } = await import('@tauri-apps/plugin-fs')
          const json = await readTextFile(path as string)
          const result = await importJson(json)
          await loadPageTree()
          showToast(`Imported ${result.pages_imported} pages`, 'success')
        }
      } else {
        const input = document.createElement('input')
        input.type = 'file'; input.accept = '.json'
        input.onchange = async () => {
          const file = input.files?.[0]
          if (file) {
            const json = await file.text()
            const result = await importJson(json)
            await loadPageTree()
            showToast(`Imported ${result.pages_imported} pages`, 'success')
          }
        }
        input.click()
      }
    } catch (e: any) { showToast('Import failed: ' + e.message, 'error') }
  }

  async function handleImportMarkdown() {
    if (isTauri) {
      try {
        const { open } = await import('@tauri-apps/plugin-dialog')
        const path = await open({ directory: true })
        if (path) {
          const result = await importMarkdownFolder(path as string)
          await loadPageTree()
          showToast(`Imported ${result.pages_imported} pages`, 'success')
        }
      } catch (e: any) { showToast('Import failed: ' + e.message, 'error') }
    }
  }
</script>

<div class="tab-content">
  <h3 class="settings-section-title">Export</h3>
  <div class="data-actions">
    <button class="data-btn" onclick={handleExportJson}>Export as JSON</button>
    <button class="data-btn" onclick={handleExportMarkdown}>Export as Markdown</button>
  </div>
  <p class="data-desc">JSON exports everything (pages, types, relations, maps, etc.). Markdown exports pages as .md files with frontmatter.</p>

  <h3 class="settings-section-title" style="margin-top: 24px">Import</h3>
  <div class="data-actions">
    <button class="data-btn" onclick={handleImportJson}>Import JSON</button>
    <button class="data-btn" onclick={handleImportMarkdown}>Import Markdown Folder</button>
  </div>
  <p class="data-desc">Import adds pages to the current Tome without replacing existing data.</p>
</div>
