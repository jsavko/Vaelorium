<script lang="ts">
  import { onMount } from 'svelte'
  import { isTauri } from '../api/bridge'
  import ConfirmDialog from './ConfirmDialog.svelte'

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
</script>

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

<ConfirmDialog
  open={installPromptOpen}
  title="Install Update"
  message={`Vaelorium ${availableUpdate?.version || ''} will be downloaded and installed. The app will restart. Continue?`}
  confirmLabel="Install & Restart"
  onConfirm={confirmInstall}
  onCancel={() => installPromptOpen = false}
/>

<style>
  .account-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    border-radius: var(--radius-sm);
  }
  .account-row:hover { background: var(--color-surface-tertiary); }
  .account-label { font-family: var(--font-ui); font-size: 14px; color: var(--color-fg-secondary); }
  .account-value { font-family: var(--font-ui); font-size: 14px; color: var(--color-fg-primary); font-weight: 500; }
  .update-actions { display: flex; gap: 8px; margin-bottom: 8px; }
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
