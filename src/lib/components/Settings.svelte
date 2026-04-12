<script lang="ts">
  import { onMount } from 'svelte'
  import { settings, updateKeybind, resetKeybinds, updateAppearance } from '../stores/settingsStore'
  import { exportTomeJson, exportTomeMarkdown, importJson, importMarkdownFolder } from '../api/export'
  import { isTauri } from '../api/bridge'
  import { showToast } from '../stores/toastStore'
  import { loadPageTree } from '../stores/pageStore'
  import ConfirmDialog from './ConfirmDialog.svelte'

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
    { id: 'account', label: 'Account' },
  ]

  let appVersion = $state<string>('')
  let checkingUpdate = $state(false)
  let updateStatus = $state<'idle' | 'up-to-date' | 'available' | 'error'>('idle')
  let updateError = $state<string>('')
  let availableUpdate = $state<{ version: string; notes: string } | null>(null)
  let updateHandle: any = null
  let installPromptOpen = $state(false)
  let installing = $state(false)
  let installProgress = $state(0)

  onMount(async () => {
    if (isTauri) {
      try {
        const { getVersion } = await import('@tauri-apps/api/app')
        appVersion = await getVersion()
      } catch {
        appVersion = 'unknown'
      }
    } else {
      appVersion = 'web'
    }
  })

  async function handleCheckForUpdates() {
    if (!isTauri || checkingUpdate) return
    checkingUpdate = true
    updateStatus = 'idle'
    updateError = ''
    availableUpdate = null
    updateHandle = null
    try {
      const { check } = await import('@tauri-apps/plugin-updater')
      const result = await check()
      localStorage.setItem('vaelorium-last-update-check', String(Date.now()))
      // Clear prior dismissal so user can re-engage with updates from the banner
      localStorage.removeItem('vaelorium-dismissed-version')
      if (result) {
        updateHandle = result
        availableUpdate = { version: result.version, notes: result.body || '' }
        updateStatus = 'available'
      } else {
        updateStatus = 'up-to-date'
      }
    } catch (e: any) {
      updateStatus = 'error'
      updateError = e?.message || String(e)
    } finally {
      checkingUpdate = false
    }
  }

  function handleInstallClick() {
    if (!updateHandle) return
    installPromptOpen = true
  }

  async function confirmInstall() {
    installPromptOpen = false
    if (!updateHandle) return
    installing = true
    installProgress = 0
    try {
      let downloaded = 0
      let contentLength = 0
      await updateHandle.downloadAndInstall((event: any) => {
        if (event.event === 'Started') {
          contentLength = event.data.contentLength || 0
        } else if (event.event === 'Progress') {
          downloaded += event.data.chunkLength
          installProgress = contentLength > 0 ? (downloaded / contentLength) * 100 : 0
        } else if (event.event === 'Finished') {
          installProgress = 100
        }
      })
      const { relaunch } = await import('@tauri-apps/plugin-process')
      await relaunch()
    } catch (e: any) {
      updateStatus = 'error'
      updateError = e?.message || String(e)
      installing = false
    }
  }

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
    { id: 'dark-library', label: 'Dark Library', description: 'Candlelit scriptorium', mode: 'dark', swatches: ['#2C2520', '#332B25', '#C8A55C', '#E8DFD0'] },
    { id: 'midnight-ink', label: 'Midnight Ink', description: 'Starlit study, ice-blue', mode: 'dark', swatches: ['#0E1420', '#17202F', '#7EB3E0', '#E0E6F0'] },
    { id: 'obsidian', label: 'Obsidian', description: 'Cyberpunk neon green', mode: 'dark', swatches: ['#0A0A0A', '#161616', '#39FF88', '#E8E8E8'] },
    { id: 'ember-hearth', label: 'Ember Hearth', description: 'Fireside warmth, orange', mode: 'dark', swatches: ['#1A1513', '#25191A', '#E87840', '#F0E6DC'] },
    { id: 'dusk', label: 'Dusk', description: 'Twilight, lavender accents', mode: 'dark', swatches: ['#1F1A26', '#2A2332', '#C39EF5', '#E6DEF0'] },
    { id: 'forest', label: 'Forest', description: 'Deep woods, emerald', mode: 'dark', swatches: ['#0F1814', '#18241F', '#5FB370', '#E0E8DE'] },
    { id: 'storm', label: 'Storm', description: 'Moody modern, electric blue', mode: 'dark', swatches: ['#1A1D24', '#262B34', '#4FC3F7', '#E8ECF0'] },
    { id: 'moonstone', label: 'Moonstone', description: 'Clean & airy, slate blue', mode: 'light', swatches: ['#F5F2ED', '#FFFFFF', '#4A6B9E', '#1C2230'] },
    { id: 'parchment', label: 'Parchment', description: 'Old manuscript, sepia', mode: 'light', swatches: ['#F5EDDE', '#FCF5E5', '#8B4513', '#3A2820'] },
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
                  <div class="theme-swatches">
                    {#each theme.swatches as color}
                      <span class="theme-swatch" style:background-color={color}></span>
                    {/each}
                  </div>
                  <div class="theme-info">
                    <span class="theme-name">{theme.label} <span class="theme-mode">{theme.mode}</span></span>
                    <span class="theme-desc">{theme.description}</span>
                  </div>
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
        {:else if activeTab === 'account'}
          <div class="settings-section">
            <h3 class="settings-section-title">About</h3>
            <div class="account-row">
              <span class="account-label">App</span>
              <span class="account-value">Vaelorium</span>
            </div>
            <div class="account-row">
              <span class="account-label">Version</span>
              <span class="account-value">{appVersion || '—'}</span>
            </div>

            <h3 class="settings-section-title" style="margin-top: 24px">Updates</h3>
            {#if !isTauri}
              <p class="data-desc">Updates are only available in the desktop app.</p>
            {:else}
              <div class="update-actions">
                <button
                  class="data-btn"
                  onclick={handleCheckForUpdates}
                  disabled={checkingUpdate || installing}
                >
                  {checkingUpdate ? 'Checking…' : 'Check for Updates'}
                </button>
                {#if updateStatus === 'available' && availableUpdate}
                  <button class="data-btn install-btn" onclick={handleInstallClick} disabled={installing}>
                    {installing ? `Installing… ${Math.round(installProgress)}%` : `Install ${availableUpdate.version}`}
                  </button>
                {/if}
              </div>
              {#if updateStatus === 'up-to-date'}
                <p class="update-status ok">You're on the latest version.</p>
              {:else if updateStatus === 'available' && availableUpdate}
                <p class="update-status info">Version {availableUpdate.version} is available.</p>
                {#if availableUpdate.notes}
                  <pre class="update-notes">{availableUpdate.notes}</pre>
                {/if}
              {:else if updateStatus === 'error'}
                <p class="update-status error">Update check failed: {updateError}</p>
              {/if}
            {/if}
          </div>
        {/if}
      </div>
    </div>
  </div>

  <ConfirmDialog
    open={installPromptOpen}
    title="Install Update"
    message={`Vaelorium ${availableUpdate?.version || ''} will be downloaded and installed. The app will restart. Continue?`}
    confirmLabel="Install & Restart"
    onConfirm={confirmInstall}
    onCancel={() => installPromptOpen = false}
  />
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
    align-items: center;
    gap: 14px;
    padding: 12px 14px;
    background: var(--color-surface-tertiary);
    border: 2px solid transparent;
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
  }

  .theme-card:hover {
    border-color: var(--color-border-strong);
  }

  .theme-card.active {
    border-color: var(--color-accent-gold);
  }

  .theme-swatches {
    display: flex;
    gap: 0;
    border-radius: var(--radius-sm);
    overflow: hidden;
    border: 1px solid var(--color-border-default);
    flex-shrink: 0;
  }

  .theme-swatch {
    width: 14px;
    height: 36px;
  }

  .theme-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    min-width: 0;
  }

  .theme-name {
    font-family: var(--font-ui);
    font-size: 14px;
    font-weight: 600;
    color: var(--color-fg-primary);
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .theme-mode {
    font-size: 10px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--color-fg-tertiary);
    background: var(--color-surface-primary);
    padding: 2px 6px;
    border-radius: 3px;
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

  .account-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    border-radius: var(--radius-sm);
  }
  .account-row:hover { background: var(--color-surface-tertiary); }
  .account-label {
    font-family: var(--font-ui); font-size: 14px; color: var(--color-fg-secondary);
  }
  .account-value {
    font-family: var(--font-ui); font-size: 14px; color: var(--color-fg-primary); font-weight: 500;
  }
  .update-actions { display: flex; gap: 8px; margin-bottom: 8px; }
  .data-btn:disabled { opacity: 0.6; cursor: not-allowed; }
  .install-btn {
    background: var(--color-accent-gold);
    border-color: var(--color-accent-gold);
    color: var(--color-fg-inverse);
    font-weight: 600;
  }
  .update-status {
    font-family: var(--font-ui); font-size: 13px; margin: 4px 0 0; padding: 8px 12px;
    border-radius: var(--radius-sm);
  }
  .update-status.ok { color: var(--color-fg-secondary); background: var(--color-surface-tertiary); }
  .update-status.info { color: var(--color-fg-primary); background: var(--color-accent-gold-subtle); }
  .update-status.error { color: var(--color-fg-primary); background: var(--color-surface-tertiary); border: 1px solid var(--color-border-default); }
  .update-notes {
    font-family: var(--font-ui); font-size: 12px; color: var(--color-fg-secondary);
    background: var(--color-surface-tertiary); padding: 10px 12px; border-radius: var(--radius-sm);
    white-space: pre-wrap; margin: 6px 0 0; max-height: 160px; overflow-y: auto;
  }
</style>
