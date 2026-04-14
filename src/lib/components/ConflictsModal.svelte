<script lang="ts">
  import { syncConflicts, refreshSyncStatus } from '../stores/syncStore'
  import { resolveConflict } from '../api/sync'
  import type { SyncConflict } from '../api/sync'

  interface Props {
    open: boolean
    onClose: () => void
  }
  let { open, onClose }: Props = $props()

  let choices = $state<Record<string, 'local' | 'remote'>>({})
  let busy = $state(false)
  // Modal-scrim close-only-on-genuine-scrim-clicks pattern (see
  // feedback_modal_scrim_close).
  let scrimMouseDown = $state(false)

  function parse(json: string | null) {
    if (json === null) return null
    try { return JSON.parse(json) } catch { return json }
  }

  function display(v: unknown): string {
    if (v === null || v === undefined) return '(empty)'
    if (typeof v === 'string') return v
    return JSON.stringify(v)
  }

  function humanizeField(name: string): string {
    const cleaned = name.replace(/_/g, ' ')
    return cleaned.charAt(0).toUpperCase() + cleaned.slice(1)
  }

  // Group by table+row so a user resolving 3 fields on one row sees them together.
  let groups = $derived.by(() => {
    const by: Record<string, SyncConflict[]> = {}
    for (const c of $syncConflicts) {
      const key = `${c.tableName}::${c.rowId}`
      ;(by[key] ||= []).push(c)
    }
    return Object.entries(by).map(([key, items]) => ({
      key,
      tableLabel: items[0].tableLabel,
      rowLabel: items[0].rowLabel,
      items,
    }))
  })

  async function applyAll() {
    busy = true
    try {
      for (const c of $syncConflicts) {
        const choice = choices[c.conflictId] ?? 'local'
        await resolveConflict(c.conflictId, choice === 'local')
      }
      choices = {}
      await refreshSyncStatus()
      onClose()
    } finally {
      busy = false
    }
  }

  function chooseAll(side: 'local' | 'remote') {
    const next: Record<string, 'local' | 'remote'> = {}
    for (const c of $syncConflicts) next[c.conflictId] = side
    choices = next
  }
</script>

{#if open}
  <div
    class="scrim"
    onmousedown={(e) => { scrimMouseDown = e.target === e.currentTarget }}
    onclick={(e) => { if (scrimMouseDown && e.target === e.currentTarget) onClose() }}
    onkeydown={(e) => e.key === 'Escape' && onClose()}
    role="presentation"
  >
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_interactive_supports_focus -->
    <div class="modal" role="dialog" aria-modal="true">
      <header class="modal-head">
        <h2>Resolve sync conflicts</h2>
        <button class="close-btn" onclick={onClose} aria-label="Close">×</button>
      </header>

      {#if $syncConflicts.length === 0}
        <p class="empty">No unresolved conflicts.</p>
      {:else}
        <p class="lede">
          These fields were edited on more than one device. Pick which side to keep per field, then Apply.
        </p>

        <div class="bulk">
          <button class="bulk-btn" onclick={() => chooseAll('local')} disabled={busy}>Keep all local</button>
          <button class="bulk-btn" onclick={() => chooseAll('remote')} disabled={busy}>Keep all remote</button>
        </div>

        <div class="groups">
          {#each groups as g (g.key)}
            <div class="group">
              <div class="group-head">
                <span class="group-table">{g.tableLabel}</span>
                <span class="group-row">{g.rowLabel}</span>
              </div>
              {#each g.items as c (c.conflictId)}
                <div class="conflict-row">
                  <div class="field">{humanizeField(c.fieldName)}</div>
                  <label class="choice" class:selected={(choices[c.conflictId] ?? 'local') === 'local'}>
                    <input
                      type="radio"
                      name={c.conflictId}
                      value="local"
                      checked={(choices[c.conflictId] ?? 'local') === 'local'}
                      onchange={() => choices = { ...choices, [c.conflictId]: 'local' }}
                    />
                    <span class="side">This device</span>
                    <span class="value">{display(parse(c.localValue))}</span>
                  </label>
                  <label class="choice" class:selected={choices[c.conflictId] === 'remote'}>
                    <input
                      type="radio"
                      name={c.conflictId}
                      value="remote"
                      checked={choices[c.conflictId] === 'remote'}
                      onchange={() => choices = { ...choices, [c.conflictId]: 'remote' }}
                    />
                    <span class="side">Other device</span>
                    <span class="value">{display(parse(c.remoteValue))}</span>
                  </label>
                </div>
              {/each}
            </div>
          {/each}
        </div>

        <footer class="modal-foot">
          <button class="foot-btn cancel" onclick={onClose} disabled={busy}>Cancel</button>
          <button class="foot-btn apply" onclick={applyAll} disabled={busy}>
            {busy ? 'Applying…' : `Apply (${$syncConflicts.length})`}
          </button>
        </footer>
      {/if}
    </div>
  </div>
{/if}

<style>
  .scrim {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2000;
    padding: 40px;
  }

  .modal {
    background: var(--color-surface-primary);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-md);
    max-width: 720px;
    width: 100%;
    max-height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .modal-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid var(--color-border-default);
  }

  .modal-head h2 {
    font-family: var(--font-heading);
    font-size: 18px;
    margin: 0;
    color: var(--color-fg-primary);
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 24px;
    color: var(--color-fg-tertiary);
    cursor: pointer;
    line-height: 1;
  }

  .close-btn:hover {
    color: var(--color-fg-primary);
  }

  .empty,
  .lede {
    font-family: var(--font-body);
    font-size: 13px;
    color: var(--color-fg-secondary);
    margin: 0;
    padding: 16px 20px;
  }

  .bulk {
    display: flex;
    gap: 8px;
    padding: 0 20px 12px;
  }

  .bulk-btn {
    padding: 6px 12px;
    background: var(--color-surface-tertiary);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-fg-secondary);
    cursor: pointer;
  }

  .bulk-btn:hover {
    border-color: var(--color-accent-gold);
    color: var(--color-fg-primary);
  }

  .groups {
    overflow-y: auto;
    padding: 0 20px;
    flex: 1;
  }

  .group {
    margin-bottom: 16px;
    padding: 12px;
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
  }

  .group-head {
    display: flex;
    gap: 10px;
    align-items: baseline;
    margin-bottom: 8px;
    padding-bottom: 8px;
    border-bottom: 1px dashed var(--color-border-default);
  }

  .group-table {
    font-family: var(--font-ui);
    font-size: 13px;
    font-weight: 600;
    color: var(--color-accent-gold);
  }

  .group-row {
    font-family: var(--font-ui);
    font-size: 11px;
    color: var(--color-fg-tertiary);
  }

  .conflict-row {
    display: grid;
    grid-template-columns: 140px 1fr 1fr;
    gap: 10px;
    align-items: start;
    padding: 8px 0;
  }

  .field {
    font-family: var(--font-ui);
    font-size: 12px;
    font-weight: 600;
    color: var(--color-fg-secondary);
    padding-top: 4px;
  }

  .choice {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 6px 8px;
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    cursor: pointer;
    background: var(--color-surface-primary);
  }

  .choice.selected {
    border-color: var(--color-accent-gold);
    background: var(--color-surface-tertiary);
  }

  .choice input[type="radio"] {
    display: none;
  }

  .side {
    font-family: var(--font-ui);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.5px;
    color: var(--color-fg-tertiary);
  }

  .value {
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-fg-primary);
    word-break: break-word;
  }

  .modal-foot {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 20px;
    border-top: 1px solid var(--color-border-default);
  }

  .foot-btn {
    padding: 8px 16px;
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
  }

  .foot-btn.cancel {
    background: var(--color-surface-tertiary);
    color: var(--color-fg-secondary);
  }

  .foot-btn.apply {
    background: var(--color-accent-gold);
    border-color: var(--color-accent-gold);
    color: var(--color-fg-inverse);
  }

  .foot-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
