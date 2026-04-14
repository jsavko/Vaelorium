<script lang="ts">
  import { isTauri } from '../api/bridge'
  import { showToast } from '../stores/toastStore'
  import { backupStatus, refreshBackupStatus, refreshSyncStatus } from '../stores/syncStore'
  import { configureBackup, disconnectBackup, unlockBackup } from '../api/backup'
  import { cloudSignout } from '../api/cloud'
  import { cloudAccount, refreshCloudAccount } from '../stores/cloudStore'

  interface Props {
    open: boolean
    onClose: () => void
    onOpenWizard?: () => void
  }
  let { open, onClose, onOpenWizard }: Props = $props()

  let syncSetupOpen = $state(false)
  let syncUnlockPassphrase = $state('')
  let syncBackendKind = $state<'filesystem' | 's3'>('filesystem')
  let syncBackendPath = $state('')
  let syncS3Endpoint = $state('')
  let syncS3Region = $state('us-east-1')
  let syncS3Bucket = $state('')
  let syncS3AccessKey = $state('')
  let syncS3SecretKey = $state('')
  let syncS3Prefix = $state('vaelorium')
  let syncPassphrase = $state('')
  let syncPassphraseConfirm = $state('')
  let syncDeviceName = $state('')
  let syncBusy = $state(false)
  let syncSetupError = $state<string | null>(null)
  let cloudBusy = $state(false)

  // Refresh cloud account info when this tab becomes active so plan /
  // usage changes from out-of-band actions (Stripe webhook, admin tool)
  // are visible without waiting for the next sync.
  $effect(() => {
    if (open) refreshCloudAccount()
  })

  async function pickBackendDir() {
    if (!isTauri) { syncSetupError = 'Sync requires the desktop app'; return }
    const { open } = await import('@tauri-apps/plugin-dialog')
    const path = await open({ directory: true })
    if (path) syncBackendPath = path as string
  }

  async function submitConfigureBackup() {
    syncSetupError = null
    if (syncPassphrase.length < 8) { syncSetupError = 'Passphrase must be at least 8 characters'; return }
    if (syncPassphrase !== syncPassphraseConfirm) { syncSetupError = 'Passphrases do not match'; return }

    let backendConfig: Record<string, unknown>
    if (syncBackendKind === 'filesystem') {
      if (!syncBackendPath) { syncSetupError = 'Folder path is required'; return }
      backendConfig = { path: syncBackendPath }
    } else {
      if (!syncS3Bucket) { syncSetupError = 'Bucket name is required'; return }
      if (!syncS3AccessKey || !syncS3SecretKey) { syncSetupError = 'Access key and secret key are required'; return }
      backendConfig = {
        endpoint: syncS3Endpoint || null,
        region: syncS3Region,
        bucket: syncS3Bucket,
        access_key: syncS3AccessKey,
        secret_key: syncS3SecretKey,
        prefix: syncS3Prefix || null,
      }
    }

    syncBusy = true
    try {
      await configureBackup({
        backendKind: syncBackendKind,
        backendConfig,
        passphrase: syncPassphrase,
        deviceName: syncDeviceName || undefined,
      })
      syncSetupOpen = false
      syncPassphrase = ''
      syncPassphraseConfirm = ''
      syncS3AccessKey = ''
      syncS3SecretKey = ''
      await refreshBackupStatus()
      await refreshSyncStatus()
      showToast('Backup connected', 'success')
    } catch (e: any) {
      syncSetupError = e.message || String(e)
    } finally {
      syncBusy = false
    }
  }

  // Cloud reports quotas as SI decimal bytes (1 GB = 1,000,000,000)
  // matching the "1 GB / 10 GB" marketing copy on plan cards.
  function formatBytes(n: number): string {
    if (n < 1000) return `${n} B`
    if (n < 1_000_000) return `${(n / 1000).toFixed(1)} KB`
    if (n < 1_000_000_000) return `${(n / 1_000_000).toFixed(1)} MB`
    return `${(n / 1_000_000_000).toFixed(2)} GB`
  }

  async function handleCloudSignoutOnly() {
    cloudBusy = true
    try {
      await cloudSignout()
      await refreshCloudAccount()
      await refreshBackupStatus()
      await refreshSyncStatus()
      showToast('Signed out of Vaelorium Cloud', 'success')
    } catch (e: any) {
      showToast(`Sign out failed: ${e.message || e}`, 'error')
    } finally {
      cloudBusy = false
    }
  }

  async function handleDisconnectBackup() {
    syncBusy = true
    try {
      if ($backupStatus.backendKind === 'hosted') {
        try { await cloudSignout() } catch (e) { console.warn('[cloud] signout failed:', e) }
      }
      await disconnectBackup()
      await refreshBackupStatus()
      await refreshSyncStatus()
      showToast('Backup disconnected', 'success')
    } catch (e: any) {
      showToast(`Disconnect failed: ${e.message || e}`, 'error')
    } finally {
      syncBusy = false
    }
  }

  async function handleUnlock() {
    if (!syncUnlockPassphrase) return
    syncBusy = true
    try {
      await unlockBackup(syncUnlockPassphrase)
      syncUnlockPassphrase = ''
      await refreshBackupStatus()
      await refreshSyncStatus()
      showToast('Backup unlocked', 'success')
    } catch (e: any) {
      showToast(`Unlock failed: ${e.message || e}`, 'error')
    } finally { syncBusy = false }
  }
</script>

<div class="settings-section">
  <h3 class="settings-section-title">Backup destination</h3>
  <p class="data-desc">
    Configure one backend (filesystem or S3-compatible bucket) and one passphrase for the whole app.
    Each Tome can opt into syncing on the Sync tab. Your data is end-to-end encrypted before it leaves your device.
  </p>
  {#if !isTauri}
    <p class="data-desc">Backup is only available in the desktop app.</p>
  {:else if $backupStatus.configured && $backupStatus.locked}
    <p class="data-desc">Backup is locked. Enter your passphrase to unlock.</p>
    <div class="sync-form">
      <label class="sync-field">
        <span class="sync-label">Passphrase</span>
        <input type="password" bind:value={syncUnlockPassphrase} class="sync-input" autocomplete="current-password"
          onkeydown={(e) => { if (e.key === 'Enter') handleUnlock() }} />
      </label>
      <div class="sync-actions">
        <button class="data-btn primary" onclick={handleUnlock} disabled={syncBusy || !syncUnlockPassphrase}>
          {syncBusy ? 'Unlocking…' : 'Unlock'}
        </button>
      </div>
    </div>
  {:else if $backupStatus.configured}
    <div class="sync-status-card">
      <div class="sync-status-row">
        <span class="sync-status-label">Status</span>
        <span class="sync-status-value">Connected</span>
      </div>
      <div class="sync-status-row">
        <span class="sync-status-label">Backend</span>
        <span class="sync-status-value">{$backupStatus.backendKind} — {$backupStatus.backendSummary}</span>
      </div>
      <div class="sync-status-row">
        <span class="sync-status-label">Device name</span>
        <span class="sync-status-value">{$backupStatus.deviceName}</span>
      </div>
    </div>

    {#if $backupStatus.backendKind === 'hosted'}
      <h3 class="settings-section-title" style="margin-top: 20px">Vaelorium Cloud account</h3>
      {#if $cloudAccount}
        <div class="sync-status-card">
          <div class="sync-status-row">
            <span class="sync-status-label">Email</span>
            <span class="sync-status-value">{$cloudAccount.email}</span>
          </div>
          {#if $cloudAccount.tier}
            <div class="sync-status-row">
              <span class="sync-status-label">Plan</span>
              <span class="sync-status-value">
                {$cloudAccount.tier}
                {#if $cloudAccount.usage?.subscriptionStatus && $cloudAccount.usage.subscriptionStatus !== 'active'}
                  <span class="sync-status-value warn">· {$cloudAccount.usage.subscriptionStatus}</span>
                {/if}
              </span>
            </div>
          {/if}
          {#if $cloudAccount.usage}
            <div class="sync-status-row">
              <span class="sync-status-label">Storage</span>
              <span class="sync-status-value">
                {formatBytes($cloudAccount.usage.bytesUsed)} of {formatBytes($cloudAccount.usage.quotaBytes)}
              </span>
            </div>
            <div class="sync-status-row">
              <span class="sync-status-label">Tomes</span>
              <span class="sync-status-value">
                {$cloudAccount.usage.tomeCount}{$cloudAccount.usage.tomeLimit !== null ? ` of ${$cloudAccount.usage.tomeLimit}` : ' (unlimited)'}
              </span>
            </div>
          {:else}
            <div class="sync-status-row">
              <span class="sync-status-label">Usage</span>
              <span class="sync-status-value" style="opacity: 0.7">Unavailable</span>
            </div>
          {/if}
        </div>
        <div class="sync-actions" style="margin-top: 8px">
          <button class="data-btn" onclick={handleCloudSignoutOnly} disabled={cloudBusy}>
            {cloudBusy ? 'Signing out…' : 'Sign out'}
          </button>
        </div>
        <p class="data-desc" style="margin-top: 6px">
          Sign out clears your cloud auth only — your backup config + encryption passphrase are kept, so you can sign back in without re-running setup.
          Use <strong>Disconnect backup</strong> below to tear down the whole backup configuration.
        </p>
      {:else}
        <p class="data-desc">Not signed in to Vaelorium Cloud.</p>
        {#if onOpenWizard}
          <div class="sync-actions" style="margin-top: 8px">
            <button class="data-btn primary" onclick={() => { onClose(); onOpenWizard?.() }}>Sign in…</button>
          </div>
        {/if}
      {/if}
    {/if}

    <div class="sync-actions" style="margin-top: 20px">
      <button class="data-btn danger" onclick={handleDisconnectBackup} disabled={syncBusy}>
        Disconnect backup
      </button>
    </div>
  {:else if syncSetupOpen}
    <div class="sync-form">
      <label class="sync-field">
        <span class="sync-label">Backend</span>
        <select bind:value={syncBackendKind} class="sync-input">
          <option value="filesystem">Filesystem (local folder, Syncthing-friendly)</option>
          <option value="s3">S3-compatible (AWS, R2, Minio, Backblaze, Wasabi, Garage…)</option>
        </select>
      </label>
      {#if syncBackendKind === 'filesystem'}
        <label class="sync-field">
          <span class="sync-label">Folder</span>
          <div class="sync-row">
            <input type="text" bind:value={syncBackendPath} placeholder="/path/to/sync/folder" class="sync-input" />
            <button type="button" class="data-btn" onclick={pickBackendDir}>Pick…</button>
          </div>
        </label>
      {:else}
        <label class="sync-field">
          <span class="sync-label">Endpoint URL (leave empty for AWS S3)</span>
          <input type="text" bind:value={syncS3Endpoint} placeholder="https://s3.us-east-1.amazonaws.com" class="sync-input" />
        </label>
        <div class="sync-row-split">
          <label class="sync-field">
            <span class="sync-label">Region</span>
            <input type="text" bind:value={syncS3Region} placeholder="us-east-1" class="sync-input" />
          </label>
          <label class="sync-field">
            <span class="sync-label">Bucket</span>
            <input type="text" bind:value={syncS3Bucket} placeholder="my-vaelorium-bucket" class="sync-input" />
          </label>
        </div>
        <label class="sync-field">
          <span class="sync-label">Access key ID</span>
          <input type="text" bind:value={syncS3AccessKey} class="sync-input" autocomplete="off" />
        </label>
        <label class="sync-field">
          <span class="sync-label">Secret access key</span>
          <input type="password" bind:value={syncS3SecretKey} class="sync-input" autocomplete="new-password" />
        </label>
        <label class="sync-field">
          <span class="sync-label">Prefix (optional)</span>
          <input type="text" bind:value={syncS3Prefix} placeholder="vaelorium" class="sync-input" />
        </label>
      {/if}
      <label class="sync-field">
        <span class="sync-label">Passphrase</span>
        <input type="password" bind:value={syncPassphrase} class="sync-input" autocomplete="new-password" />
      </label>
      <label class="sync-field">
        <span class="sync-label">Confirm passphrase</span>
        <input type="password" bind:value={syncPassphraseConfirm} class="sync-input" autocomplete="new-password" />
      </label>
      <p class="sync-warning">
        ⚠ Lose this passphrase and your data is unrecoverable. There is no recovery.
      </p>
      <label class="sync-field">
        <span class="sync-label">Device name (shown in op history across devices)</span>
        <input type="text" bind:value={syncDeviceName} placeholder="Laptop" class="sync-input" />
      </label>
      {#if syncSetupError}
        <p class="sync-error">{syncSetupError}</p>
      {/if}
      <div class="sync-actions">
        <button class="data-btn" onclick={() => syncSetupOpen = false}>Cancel</button>
        <button class="data-btn primary" onclick={submitConfigureBackup} disabled={syncBusy}>
          {syncBusy ? 'Connecting…' : 'Connect backup'}
        </button>
      </div>
    </div>
  {:else}
    <div class="setup-actions">
      {#if onOpenWizard}
        <button class="data-btn primary" onclick={() => { onClose(); onOpenWizard?.() }}>Set up backup…</button>
      {/if}
      <button class="data-btn" onclick={() => syncSetupOpen = true}>Manual setup</button>
    </div>
  {/if}
</div>

<style>
  .sync-form { display: flex; flex-direction: column; gap: 14px; margin-top: 12px; }
  .sync-field { display: flex; flex-direction: column; gap: 6px; }
  .sync-label {
    font-family: var(--font-ui); font-size: 12px; font-weight: 600;
    text-transform: uppercase; letter-spacing: 0.06em;
    color: var(--color-fg-tertiary);
  }
  .sync-input {
    padding: 8px 12px; background: var(--color-surface-tertiary);
    border: 1px solid var(--color-border-default); border-radius: var(--radius-sm);
    color: var(--color-fg-primary); font-family: var(--font-ui); font-size: 14px;
  }
  .sync-input:focus { outline: 1px solid var(--color-accent-gold); }
  .sync-row { display: flex; gap: 8px; }
  .sync-row .sync-input { flex: 1; }
  .sync-row-split { display: flex; gap: 12px; }
  .sync-row-split > .sync-field { flex: 1; }
  .sync-warning {
    margin: 4px 0 0; padding: 10px 12px;
    background: rgba(184, 92, 92, 0.12);
    border: 1px solid var(--color-status-error);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui); font-size: 13px;
    color: var(--color-fg-primary);
  }
  .sync-error {
    color: var(--color-status-error);
    font-family: var(--font-ui); font-size: 13px; margin: 0;
  }
  .sync-actions { display: flex; gap: 8px; flex-wrap: wrap; margin-top: 8px; }
  .sync-status-card {
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-md);
    padding: 14px 16px; margin: 12px 0;
    display: flex; flex-direction: column; gap: 6px;
  }
  .sync-status-row {
    display: flex; justify-content: space-between; align-items: baseline;
    font-family: var(--font-ui); font-size: 13px;
  }
  .sync-status-label { color: var(--color-fg-tertiary); }
  .sync-status-value { color: var(--color-fg-primary); font-weight: 500; }
  .sync-status-value.warn { color: var(--color-status-warning); }
  .setup-actions { display: flex; gap: 8px; }
</style>
