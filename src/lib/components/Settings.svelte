<script lang="ts">
  import { settings, updateKeybind, resetKeybinds, updateAppearance } from '../stores/settingsStore'
  import { exportTomeJson, exportTomeMarkdown, importJson, importMarkdownFolder } from '../api/export'
  import { isTauri } from '../api/bridge'
  import { showToast } from '../stores/toastStore'
  import { loadPageTree } from '../stores/pageStore'

  interface Props {
    open: boolean
    onClose: () => void
  }

  let { open, onClose }: Props = $props()

  let activeTab = $state('keybinds')
  let editingKeybind = $state<string | null>(null)
  let listeningForKey = $state(false)

  const tabs = [
    { id: 'keybinds', label: 'Keybinds' },
    { id: 'appearance', label: 'Appearance' },
    { id: 'data', label: 'Data' },
  ]

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

  const themes = [
    { id: 'dark-library', label: 'Dark Library', description: 'Candlelit scriptorium (default)' },
  ]

  function startEditKeybind(id: string) {
    editingKeybind = id
    listeningForKey = true
  }

  function handleKeybindKeydown(e: KeyboardEvent) {
    if (!listeningForKey || !editingKeybind) return
    e.preventDefault()
    const parts: string[] = []
    if (e.ctrlKey || e.metaKey) parts.push('Ctrl')
    if (e.shiftKey) parts.push('Shift')
    if (e.altKey) parts.push('Alt')
    if (!['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) {
      if (e.key === ' ') {
        parts.push('Space')
      } else {
        parts.push(e.key.length === 1 ? e.key.toUpperCase() : e.key)
      }
    }
    if (parts.length > 1 || (!e.ctrlKey && !e.metaKey && !e.shiftKey && !e.altKey)) {
      const combo = parts.join('+')
      updateKeybind(editingKeybind, combo)
      editingKeybind = null
      listeningForKey = false
    }
  }
</script>

{#if open}
  <div class="settings-overlay" data-testid="settings">
    <div class="settings-panel">
      <div class="settings-sidebar">
        <h2 class="settings-title">Settings</h2>
        <nav class="settings-nav">
          {#each tabs as tab}
            <button
              class="settings-nav-item"
              class:active={activeTab === tab.id}
              onclick={() => activeTab = tab.id}
            >
              {tab.label}
            </button>
          {/each}
        </nav>
        <div class="settings-sidebar-spacer"></div>
        <button class="settings-close" onclick={onClose}>Close</button>
      </div>

      <div class="settings-content">
        {#if activeTab === 'keybinds'}
          <div class="settings-section">
            <div class="section-header-row">
              <h3 class="settings-section-title">Keyboard Shortcuts</h3>
              <button class="reset-btn" onclick={resetKeybinds}>Reset to defaults</button>
            </div>
            <div class="keybind-list">
              {#each $settings.keybinds as kb (kb.id)}
                <div class="keybind-row">
                  <span class="keybind-label">{kb.label}</span>
                  {#if editingKeybind === kb.id}
                    <!-- svelte-ignore a11y_autofocus -->
                    <input
                      class="keybind-input listening"
                      value="Press keys..."
                      readonly
                      autofocus
                      onkeydown={handleKeybindKeydown}
                      onblur={() => { editingKeybind = null; listeningForKey = false; }}
                    />
                  {:else}
                    <button class="keybind-value" onclick={() => startEditKeybind(kb.id)}>
                      {kb.keys}
                    </button>
                  {/if}
                </div>
              {/each}
            </div>
          </div>
        {:else if activeTab === 'appearance'}
          <div class="settings-section">
            <h3 class="settings-section-title">Theme</h3>
            <div class="theme-list">
              {#each themes as theme}
                <button
                  class="theme-card"
                  class:active={$settings.appearance.theme === theme.id}
                  onclick={() => updateAppearance({ theme: theme.id })}
                >
                  <span class="theme-name">{theme.label}</span>
                  <span class="theme-desc">{theme.description}</span>
                </button>
              {/each}
            </div>

            <h3 class="settings-section-title" style="margin-top: 24px">Font Size</h3>
            <div class="font-size-row">
              <input
                type="range"
                min="12"
                max="22"
                value={$settings.appearance.fontSize}
                oninput={(e) => updateAppearance({ fontSize: parseInt(e.currentTarget.value) })}
                class="font-size-slider"
              />
              <span class="font-size-value">{$settings.appearance.fontSize}px</span>
            </div>
          </div>
        {:else if activeTab === 'data'}
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
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .settings-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }

  .settings-panel {
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-lg);
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.4);
    width: 700px;
    height: 500px;
    display: flex;
    overflow: hidden;
  }

  .settings-sidebar {
    width: 200px;
    background: var(--color-surface-secondary);
    padding: 24px 16px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .settings-title {
    font-family: var(--font-heading);
    font-size: 20px;
    font-weight: 600;
    color: var(--color-fg-primary);
    margin: 0 0 16px 8px;
  }

  .settings-nav-item {
    display: block;
    width: 100%;
    padding: 8px 12px;
    border: none;
    background: transparent;
    font-family: var(--font-ui);
    font-size: 14px;
    color: var(--color-fg-secondary);
    text-align: left;
    cursor: pointer;
    border-radius: var(--radius-sm);
  }

  .settings-nav-item:hover {
    background: var(--color-surface-tertiary);
  }

  .settings-nav-item.active {
    background: var(--color-accent-gold-subtle);
    color: var(--color-fg-primary);
  }

  .settings-sidebar-spacer {
    flex: 1;
  }

  .settings-close {
    padding: 8px 12px;
    border: none;
    background: transparent;
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-tertiary);
    cursor: pointer;
    border-radius: var(--radius-sm);
    text-align: left;
  }

  .settings-close:hover {
    background: var(--color-surface-tertiary);
    color: var(--color-fg-primary);
  }

  .settings-content {
    flex: 1;
    padding: 24px 32px;
    overflow-y: auto;
  }

  .settings-section {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .section-header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .settings-section-title {
    font-family: var(--font-ui);
    font-size: 16px;
    font-weight: 600;
    color: var(--color-fg-primary);
    margin: 0;
  }

  .reset-btn {
    background: none;
    border: none;
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-accent-gold);
    cursor: pointer;
  }

  .keybind-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .keybind-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    border-radius: var(--radius-sm);
  }

  .keybind-row:hover {
    background: var(--color-surface-tertiary);
  }

  .keybind-label {
    font-family: var(--font-ui);
    font-size: 14px;
    color: var(--color-fg-primary);
  }

  .keybind-value {
    background: var(--color-surface-tertiary);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    padding: 4px 12px;
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-fg-secondary);
    cursor: pointer;
    min-width: 80px;
    text-align: center;
  }

  .keybind-value:hover {
    border-color: var(--color-accent-gold);
  }

  .keybind-input {
    background: var(--color-accent-gold-subtle);
    border: 1px solid var(--color-accent-gold);
    border-radius: var(--radius-sm);
    padding: 4px 12px;
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-accent-gold);
    min-width: 80px;
    text-align: center;
    outline: none;
  }

  .theme-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .theme-card {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 14px 16px;
    background: var(--color-surface-tertiary);
    border: 2px solid transparent;
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
  }

  .theme-card.active {
    border-color: var(--color-accent-gold);
  }

  .theme-name {
    font-family: var(--font-ui);
    font-size: 14px;
    font-weight: 600;
    color: var(--color-fg-primary);
  }

  .theme-desc {
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-fg-tertiary);
  }

  .font-size-row {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .font-size-slider {
    flex: 1;
    accent-color: var(--color-accent-gold);
  }

  .font-size-value {
    font-family: var(--font-ui);
    font-size: 14px;
    color: var(--color-fg-primary);
    min-width: 40px;
  }

  .data-actions { display: flex; gap: 8px; margin-bottom: 8px; }
  .data-btn {
    padding: 8px 16px; background: var(--color-surface-tertiary);
    border: 1px solid var(--color-border-default); border-radius: var(--radius-sm);
    font-family: var(--font-ui); font-size: 13px; color: var(--color-fg-primary); cursor: pointer;
  }
  .data-btn:hover { border-color: var(--color-accent-gold); }
  .data-desc { font-family: var(--font-ui); font-size: 12px; color: var(--color-fg-tertiary); margin: 0; }
</style>
