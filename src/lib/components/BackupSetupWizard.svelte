<script lang="ts">
  import { isTauri } from '../api/bridge'
  import { configureBackup } from '../api/backup'
  import { cloudSignin } from '../api/cloud'
  import { refreshBackupStatus, refreshSyncStatus } from '../stores/syncStore'
  import { showToast } from '../stores/toastStore'

  interface Props {
    open: boolean
    onClose: () => void
    onComplete?: () => void
  }
  let { open, onClose, onComplete }: Props = $props()

  type Step = 1 | 2 | 3 | 4 | 5
  let step = $state<Step>(1)
  let busy = $state(false)
  let error = $state<string | null>(null)
  // Tracks whether the most recent mousedown started on the scrim itself
  // — guards against drag-from-input-overshooting-modal-edge closing the
  // modal. Cleared when the click resolves either way.
  let scrimMouseDown = $state(false)

  // Form state — same fields as the inline Settings → Backup form.
  let backendKind = $state<'filesystem' | 's3' | 'hosted'>('hosted')
  let backendPath = $state('')
  let s3Endpoint = $state('')
  let s3Region = $state('us-east-1')
  let s3Bucket = $state('')
  let s3AccessKey = $state('')
  let s3SecretKey = $state('')
  let s3Prefix = $state('vaelorium')
  // Hosted (Vaelorium Cloud) signin fields. Distinct from `passphrase`
  // — cloud password authenticates to the account; passphrase encrypts
  // your data before upload.
  let cloudEmail = $state('')
  let cloudPassword = $state('')
  let cloudAccountInfo = $state<{ email: string; tier: string | null } | null>(null)
  let passphrase = $state('')
  let passphraseConfirm = $state('')
  let deviceName = $state('')

  // Reset wizard each time it opens so a re-launch starts fresh.
  $effect(() => {
    if (open) {
      step = 1
      error = null
      cloudAccountInfo = null
    }
  })

  async function openExternal(url: string) {
    // In web preview mode we can just use window.open. In the Tauri
    // desktop window, WebKitGTK will intercept target=_blank links
    // and hand them off to the OS default browser.
    window.open(url, '_blank', 'noopener,noreferrer')
  }

  async function pickFolder() {
    if (!isTauri) { error = 'Folder picker only available in the desktop app'; return }
    const { open } = await import('@tauri-apps/plugin-dialog')
    const path = await open({ directory: true })
    if (path) backendPath = path as string
  }

  function validateStep(s: Step): string | null {
    if (s === 3) {
      if (backendKind === 'filesystem' && !backendPath) return 'Folder path is required'
      if (backendKind === 's3') {
        if (!s3Bucket) return 'Bucket name is required'
        if (!s3Region) return 'Region is required'
        if (!s3AccessKey || !s3SecretKey) return 'Access key and secret key are required'
      }
      if (backendKind === 'hosted') {
        if (!cloudAccountInfo) return 'Sign in to Vaelorium Cloud to continue'
      }
    }
    if (s === 4) {
      if (passphrase.length < 8) return 'Passphrase must be at least 8 characters'
      if (passphrase !== passphraseConfirm) return 'Passphrases do not match'
    }
    return null
  }

  async function next() {
    const v = validateStep(step)
    if (v) { error = v; return }
    error = null
    if (step < 5) step = (step + 1) as Step
  }

  async function handleCloudSignin() {
    error = null
    if (!cloudEmail || !cloudPassword) {
      error = 'Email and password are required'
      return
    }
    busy = true
    try {
      const info = await cloudSignin({
        email: cloudEmail.trim(),
        password: cloudPassword,
        deviceName: deviceName || undefined,
      })
      cloudAccountInfo = { email: info.email, tier: info.tier }
      // Drop the account password from memory as soon as signin succeeds
      // — we don't need it again; the device token is in the keychain.
      cloudPassword = ''
      showToast(`Signed in as ${info.email}`, 'success')
    } catch (e: any) {
      error = e?.message || String(e)
    } finally {
      busy = false
    }
  }

  function back() {
    error = null
    if (step > 1) step = (step - 1) as Step
  }

  async function connect() {
    error = null
    busy = true
    try {
      let backendConfig: Record<string, unknown>
      if (backendKind === 'filesystem') {
        backendConfig = { path: backendPath }
      } else if (backendKind === 's3') {
        backendConfig = {
          endpoint: s3Endpoint || null,
          region: s3Region,
          bucket: s3Bucket,
          access_key: s3AccessKey,
          secret_key: s3SecretKey,
          prefix: s3Prefix || null,
        }
      } else {
        // hosted: no user-supplied backend_config; the Rust side fills
        // in email + tier from keychain.
        backendConfig = {}
      }
      await configureBackup({ backendKind, backendConfig, passphrase, deviceName: deviceName || undefined })
      await refreshBackupStatus()
      await refreshSyncStatus()
      showToast('Backup connected', 'success')
      // Clear sensitive fields before closing.
      passphrase = ''
      passphraseConfirm = ''
      s3AccessKey = ''
      s3SecretKey = ''
      cloudPassword = ''
      onComplete?.()
      onClose()
    } catch (e: any) {
      error = e?.message || String(e)
    } finally {
      busy = false
    }
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    class="scrim"
    onmousedown={(e) => { scrimMouseDown = e.target === e.currentTarget }}
    onclick={(e) => { if (scrimMouseDown && e.target === e.currentTarget) onClose() }}
    role="presentation"
  >
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_interactive_supports_focus -->
    <div class="modal" role="dialog" aria-modal="true">
      <header class="head">
        <div class="head-text">
          <h2>Set up backup</h2>
          <p class="sub">Step {step} of 5</p>
        </div>
        <button class="close" onclick={onClose} aria-label="Close">×</button>
      </header>

      <div class="body">
        {#if step === 1}
          <h3>Welcome</h3>
          <p>
            Vaelorium can back up your Tomes and sync them across devices.
            Everything is end-to-end encrypted — even on Vaelorium Cloud,
            your data is encrypted before upload with a passphrase only you know.
          </p>
          <ul class="bullets">
            <li><strong>Vaelorium Cloud</strong> — hosted, paid, zero-config. Sign in and sync across any number of devices.</li>
            <li><strong>A folder</strong> on your disk or mounted NAS share — works with Syncthing / Dropbox / iCloud for cross-device sync.</li>
            <li><strong>S3-compatible bucket</strong> — AWS, Cloudflare R2, Backblaze B2, Minio…</li>
            <li>One backup destination is shared across every Tome you sync.</li>
          </ul>
        {:else if step === 2}
          <h3>Where should backups go?</h3>
          <label class="kind-card" class:selected={backendKind === 'hosted'}>
            <input type="radio" name="kind" value="hosted" checked={backendKind === 'hosted'} onchange={() => backendKind = 'hosted'} />
            <div class="kind-body">
              <div class="kind-title">Vaelorium Cloud <span class="pill">recommended</span></div>
              <p class="kind-desc">
                Hosted, encrypted, paid. Sign in with your Vaelorium account — no bucket config, no credentials to manage.
                Zero-knowledge: your data is encrypted on-device before upload.
              </p>
            </div>
          </label>
          <label class="kind-card" class:selected={backendKind === 'filesystem'}>
            <input type="radio" name="kind" value="filesystem" checked={backendKind === 'filesystem'} onchange={() => backendKind = 'filesystem'} />
            <div class="kind-body">
              <div class="kind-title">A folder (local, synced, or network share)</div>
              <p class="kind-desc">
                Any path the OS can write — local disk, Syncthing/Dropbox/iCloud folder, or a mounted NAS share.
                Windows UNC paths (<code>\\SERVER\share\…</code>), macOS <code>/Volumes/&lt;share&gt;</code>, and Linux <code>/mnt/…</code>
                all work. Only encrypted ops/snapshots land here — your live Tome stays local.
              </p>
            </div>
          </label>
          <label class="kind-card" class:selected={backendKind === 's3'}>
            <input type="radio" name="kind" value="s3" checked={backendKind === 's3'} onchange={() => backendKind = 's3'} />
            <div class="kind-body">
              <div class="kind-title">S3-compatible bucket</div>
              <p class="kind-desc">AWS, Cloudflare R2, Backblaze B2, Wasabi, Minio, Garage. Best for BYO-cloud setups.</p>
            </div>
          </label>
        {:else if step === 3}
          {#if backendKind === 'hosted'}
            <h3>Sign in to Vaelorium Cloud</h3>
            {#if cloudAccountInfo}
              <div class="review">
                <dl>
                  <dt>Signed in as</dt>
                  <dd>{cloudAccountInfo.email}</dd>
                  {#if cloudAccountInfo.tier}
                    <dt>Plan</dt>
                    <dd>{cloudAccountInfo.tier}</dd>
                  {/if}
                </dl>
                <button class="ghost" type="button" onclick={() => { cloudAccountInfo = null; cloudPassword = '' }}>Sign in as a different account</button>
              </div>
            {:else}
              <p class="sub">
                Use your Vaelorium account password. Need an account?
                <button class="link" type="button" onclick={() => openExternal('https://cloud.vaelorium.com/signup')}>Create one →</button>
              </p>
              <form onsubmit={(e) => { e.preventDefault(); handleCloudSignin() }}>
                <label>Email
                  <input class="text" type="email" autocomplete="username" autofocus bind:value={cloudEmail} />
                </label>
                <label>Password
                  <input class="text" type="password" autocomplete="current-password" bind:value={cloudPassword} />
                </label>
                <div class="row">
                  <button class="primary" type="submit" disabled={busy}>
                    {busy ? 'Signing in…' : 'Sign in'}
                  </button>
                </div>
              </form>
            {/if}
          {:else if backendKind === 'filesystem'}
            <h3>Pick a folder</h3>
            <p class="sub">
              Any local directory, a Syncthing/Dropbox folder, or a mounted network share (SMB/UNC, NFS, AFP)
              works — the OS just needs to be able to write there. Your live <code>.tome</code> database stays local;
              this folder only receives immutable op/snapshot files.
            </p>
            <div class="row">
              <input class="text" type="text" placeholder="/path/to/backup/folder" bind:value={backendPath} />
              <button class="ghost" type="button" onclick={pickFolder}>Browse…</button>
            </div>
          {:else}
            <h3>S3 bucket details</h3>
            <div class="grid">
              <label>Endpoint <span class="opt">optional — leave blank for AWS</span>
                <input class="text" type="text" placeholder="https://s3.us-west-000.backblazeb2.com" bind:value={s3Endpoint} />
              </label>
              <label>Region <input class="text" type="text" bind:value={s3Region} /></label>
              <label>Bucket <input class="text" type="text" bind:value={s3Bucket} /></label>
              <label>Access key <input class="text" type="text" bind:value={s3AccessKey} /></label>
              <label>Secret key <input class="text" type="password" bind:value={s3SecretKey} /></label>
              <label>Prefix <span class="opt">optional</span>
                <input class="text" type="text" bind:value={s3Prefix} />
              </label>
            </div>
          {/if}
        {:else if step === 4}
          <h3>Encryption passphrase</h3>
          {#if backendKind === 'hosted'}
            <p class="sub">
              This is <strong>separate</strong> from your Vaelorium account password. It encrypts your
              data on-device before upload, so even Vaelorium Cloud can't read it.
            </p>
          {/if}
          <div class="warning">
            <strong>This passphrase encrypts everything</strong>. There is no recovery if you lose it.
            Write it down somewhere safe before continuing.
          </div>
          <label>New passphrase <input class="text" type="password" bind:value={passphrase} autocomplete="new-password" /></label>
          <label>Confirm passphrase <input class="text" type="password" bind:value={passphraseConfirm} autocomplete="new-password" /></label>
        {:else if step === 5}
          <h3>Review and connect</h3>
          <label>Device name <span class="opt">how this machine will appear in conflict logs</span>
            <input class="text" type="text" placeholder="My Laptop" bind:value={deviceName} />
          </label>
          <dl class="review">
            <dt>Destination</dt>
            <dd>
              {#if backendKind === 'hosted'}
                Vaelorium Cloud{cloudAccountInfo ? ` — ${cloudAccountInfo.email}` : ''}{cloudAccountInfo?.tier ? ` (${cloudAccountInfo.tier})` : ''}
              {:else if backendKind === 'filesystem'}
                Folder: {backendPath}
              {:else}
                S3: {s3Bucket}{s3Endpoint ? ' on ' + s3Endpoint : ''}
              {/if}
            </dd>
            <dt>Encryption</dt>
            <dd>End-to-end (passphrase-derived key, ChaCha20-Poly1305)</dd>
          </dl>
          <p class="sub">
            {#if backendKind === 'hosted'}
              Ready to connect. We'll persist the encryption passphrase in your OS keychain
              so you won't need to re-enter it every launch.
            {:else}
              We'll talk to the backend and verify your passphrase against any existing data.
              On a fresh destination this is instantaneous.
            {/if}
          </p>
        {/if}

        {#if error}
          <p class="err">{error}</p>
        {/if}
      </div>

      <footer class="foot">
        {#if step > 1}
          <button class="ghost" onclick={back} disabled={busy}>Back</button>
        {/if}
        <span class="spacer"></span>
        <button class="ghost" onclick={onClose} disabled={busy}>Cancel</button>
        {#if step < 5}
          <button class="primary" onclick={next} disabled={busy}>Next</button>
        {:else}
          <button class="primary" onclick={connect} disabled={busy}>{busy ? 'Connecting…' : 'Connect'}</button>
        {/if}
      </footer>
    </div>
  </div>
{/if}

<style>
  .scrim {
    position: fixed; inset: 0;
    background: rgba(0, 0, 0, 0.55);
    display: flex; align-items: center; justify-content: center;
    z-index: 2050; padding: 40px;
  }
  .modal {
    background: var(--color-surface-primary);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-md);
    width: 100%; max-width: 560px; max-height: 100%;
    display: flex; flex-direction: column; overflow: hidden;
  }
  .head {
    display: flex; align-items: flex-start; justify-content: space-between;
    padding: 16px 20px; border-bottom: 1px solid var(--color-border-default);
  }
  .head-text h2 { font-family: var(--font-heading); font-size: 18px; margin: 0; color: var(--color-fg-primary); }
  .sub { font-family: var(--font-ui); font-size: 12px; color: var(--color-fg-tertiary); margin: 2px 0 0; }
  .close { background: none; border: none; font-size: 24px; line-height: 1; color: var(--color-fg-tertiary); cursor: pointer; }
  .close:hover { color: var(--color-fg-primary); }

  .body {
    flex: 1; overflow-y: auto; padding: 18px 20px;
    display: flex; flex-direction: column; gap: 12px;
    font-family: var(--font-body); color: var(--color-fg-primary);
  }
  .body h3 { font-family: var(--font-heading); font-size: 16px; margin: 0; color: var(--color-fg-primary); }
  .body p { margin: 0; font-size: 13px; color: var(--color-fg-secondary); }
  .body label {
    display: flex; flex-direction: column; gap: 4px;
    font-family: var(--font-ui); font-size: 12px; font-weight: 600;
    color: var(--color-fg-secondary);
  }
  .opt { font-weight: 400; color: var(--color-fg-tertiary); margin-left: 6px; }
  .text {
    padding: 8px 10px; font-family: var(--font-ui); font-size: 13px;
    background: var(--color-surface-card); border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm); color: var(--color-fg-primary);
  }
  .text:focus { outline: none; border-color: var(--color-accent-gold); }
  .row { display: flex; gap: 8px; align-items: center; }
  .row .text { flex: 1; }
  .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
  .grid label:first-child, .grid label:nth-child(6) { grid-column: 1 / -1; }

  .bullets { margin: 4px 0 0; padding-left: 18px; font-size: 13px; color: var(--color-fg-secondary); }
  .bullets li { margin-bottom: 4px; }

  .kind-card {
    display: flex; gap: 10px; align-items: flex-start;
    padding: 12px; cursor: pointer;
    border: 1px solid var(--color-border-default); border-radius: var(--radius-sm);
    background: var(--color-surface-card);
  }
  .kind-card.selected { border-color: var(--color-accent-gold); }
  .kind-card input[type="radio"] { margin-top: 3px; accent-color: var(--color-accent-gold); }
  .kind-body { flex: 1; }
  .kind-title { font-family: var(--font-heading); font-size: 14px; font-weight: 600; color: var(--color-fg-primary); }
  .kind-desc { font-size: 12px; color: var(--color-fg-tertiary); margin: 2px 0 0; }

  .warning {
    background: rgba(217, 116, 116, 0.1);
    border: 1px solid var(--color-status-error, #d97474);
    color: var(--color-fg-primary); font-size: 12px;
    padding: 8px 10px; border-radius: var(--radius-sm);
  }
  .warning strong { color: var(--color-status-error, #d97474); }

  .review {
    margin: 4px 0 0; padding: 10px 12px;
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui); font-size: 12px;
    display: grid; grid-template-columns: 100px 1fr; gap: 4px 12px;
  }
  .review dt { color: var(--color-fg-tertiary); font-weight: 600; }
  .review dd { color: var(--color-fg-primary); margin: 0; word-break: break-all; }

  .err {
    color: var(--color-status-error, #d97474);
    font-family: var(--font-ui); font-size: 12px; margin: 0;
  }

  .foot {
    display: flex; align-items: center; gap: 8px;
    padding: 12px 20px; border-top: 1px solid var(--color-border-default);
  }
  .spacer { flex: 1; }
  .foot .ghost {
    padding: 7px 14px; background: var(--color-surface-tertiary);
    border: 1px solid var(--color-border-default); border-radius: var(--radius-sm);
    font-family: var(--font-ui); font-size: 12px; font-weight: 600;
    color: var(--color-fg-secondary); cursor: pointer;
  }
  .foot .ghost:hover { color: var(--color-fg-primary); border-color: var(--color-accent-gold); }
  .foot .primary {
    padding: 7px 14px; background: var(--color-accent-gold); border: none;
    border-radius: var(--radius-sm);
    font-family: var(--font-ui); font-size: 12px; font-weight: 600;
    color: var(--color-fg-inverse); cursor: pointer;
  }
  .foot button:disabled { opacity: 0.5; cursor: not-allowed; }

  .body .primary {
    padding: 7px 14px; background: var(--color-accent-gold); border: none;
    border-radius: var(--radius-sm);
    font-family: var(--font-ui); font-size: 12px; font-weight: 600;
    color: var(--color-fg-inverse); cursor: pointer;
  }
  .body .primary:disabled { opacity: 0.5; cursor: not-allowed; }
  .body .ghost {
    padding: 6px 10px; background: transparent;
    border: 1px solid var(--color-border-default); border-radius: var(--radius-sm);
    font-family: var(--font-ui); font-size: 12px; color: var(--color-fg-secondary);
    cursor: pointer;
  }
  .body .ghost:hover { color: var(--color-fg-primary); border-color: var(--color-accent-gold); }

  .pill {
    display: inline-block; margin-left: 6px;
    padding: 1px 8px; background: var(--color-accent-gold);
    color: var(--color-fg-inverse);
    border-radius: 9999px;
    font-size: 10px; font-weight: 600; letter-spacing: 0.3px;
    vertical-align: middle;
  }

  .link {
    background: none; border: none; padding: 0;
    color: var(--color-accent-gold);
    font-family: inherit; font-size: inherit;
    cursor: pointer; text-decoration: underline;
  }
  .link:hover { text-decoration: none; }
</style>
