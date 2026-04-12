<script lang="ts">
  import { onMount } from 'svelte'
  import { isTauri } from '../api/bridge'
  import ConfirmDialog from './ConfirmDialog.svelte'

  let updateAvailable = $state<{ version: string; notes: string } | null>(null)
  let downloading = $state(false)
  let downloadProgress = $state(0)
  let installPromptOpen = $state(false)
  let update: any = null

  onMount(async () => {
    if (!isTauri) return

    // Only check once per 6 hours
    const lastCheck = localStorage.getItem('vaelorium-last-update-check')
    if (lastCheck) {
      const hoursSince = (Date.now() - parseInt(lastCheck)) / (1000 * 60 * 60)
      if (hoursSince < 6) return
    }

    try {
      const { check } = await import('@tauri-apps/plugin-updater')
      const result = await check()
      localStorage.setItem('vaelorium-last-update-check', String(Date.now()))

      if (result) {
        // Check if user already dismissed this version
        const dismissed = localStorage.getItem('vaelorium-dismissed-version')
        if (dismissed === result.version) return

        update = result
        updateAvailable = { version: result.version, notes: result.body || '' }
      }
    } catch (e) {
      console.warn('Update check failed:', e)
    }
  })

  function handleDismiss() {
    if (updateAvailable) {
      localStorage.setItem('vaelorium-dismissed-version', updateAvailable.version)
    }
    updateAvailable = null
  }

  async function handleInstall() {
    if (!update) return
    installPromptOpen = true
  }

  async function confirmInstall() {
    installPromptOpen = false
    if (!update) return

    downloading = true
    try {
      let downloaded = 0
      let contentLength = 0

      await update.downloadAndInstall((event: any) => {
        if (event.event === 'Started') {
          contentLength = event.data.contentLength || 0
        } else if (event.event === 'Progress') {
          downloaded += event.data.chunkLength
          downloadProgress = contentLength > 0 ? (downloaded / contentLength) * 100 : 0
        } else if (event.event === 'Finished') {
          downloadProgress = 100
        }
      })

      // Relaunch the app
      const { relaunch } = await import('@tauri-apps/plugin-process')
      await relaunch()
    } catch (e) {
      console.error('Update failed:', e)
      downloading = false
      downloadProgress = 0
    }
  }
</script>

{#if updateAvailable}
  <div class="update-banner">
    <div class="banner-content">
      <div class="banner-icon">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 2v20M2 12h20"></path>
        </svg>
      </div>
      <div class="banner-text">
        {#if downloading}
          <span class="banner-title">Downloading update... {Math.round(downloadProgress)}%</span>
        {:else}
          <span class="banner-title">Vaelorium {updateAvailable.version} available</span>
          <span class="banner-desc">A new version is ready to install.</span>
        {/if}
      </div>
    </div>
    <div class="banner-actions">
      {#if downloading}
        <div class="progress-bar">
          <div class="progress-fill" style:width="{downloadProgress}%"></div>
        </div>
      {:else}
        <button class="banner-dismiss" onclick={handleDismiss}>Later</button>
        <button class="banner-install" onclick={handleInstall}>Install</button>
      {/if}
    </div>
  </div>
{/if}

<ConfirmDialog
  open={installPromptOpen}
  title="Install Update"
  message={`Vaelorium ${updateAvailable?.version || ''} will be downloaded and installed. The app will restart. Continue?`}
  confirmLabel="Install & Restart"
  onConfirm={confirmInstall}
  onCancel={() => installPromptOpen = false}
/>

<style>
  .update-banner {
    position: fixed;
    bottom: 24px;
    right: 24px;
    background: var(--color-surface-card);
    border: 1px solid var(--color-accent-gold);
    border-radius: var(--radius-md);
    padding: 12px 16px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    gap: 16px;
    z-index: 400;
    max-width: 420px;
  }
  .banner-content { display: flex; align-items: center; gap: 10px; flex: 1; }
  .banner-icon {
    color: var(--color-accent-gold);
    display: flex;
    align-items: center;
  }
  .banner-text { display: flex; flex-direction: column; gap: 2px; }
  .banner-title {
    font-family: var(--font-ui); font-size: 13px; font-weight: 600; color: var(--color-fg-primary);
  }
  .banner-desc {
    font-family: var(--font-ui); font-size: 11px; color: var(--color-fg-tertiary);
  }
  .banner-actions { display: flex; gap: 6px; align-items: center; }
  .banner-dismiss {
    padding: 5px 10px; background: none; border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm); font-family: var(--font-ui); font-size: 12px;
    color: var(--color-fg-secondary); cursor: pointer;
  }
  .banner-install {
    padding: 5px 12px; background: var(--color-accent-gold); border: none;
    border-radius: var(--radius-sm); font-family: var(--font-ui); font-size: 12px;
    font-weight: 600; color: var(--color-fg-inverse); cursor: pointer;
  }
  .banner-install:hover { background: var(--color-accent-gold-hover); }

  .progress-bar {
    width: 120px; height: 6px; background: var(--color-surface-tertiary);
    border-radius: 3px; overflow: hidden;
  }
  .progress-fill {
    height: 100%; background: var(--color-accent-gold);
    transition: width 0.1s;
  }
</style>
