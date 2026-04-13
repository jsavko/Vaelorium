<script lang="ts">
  import { syncConflicts, refreshSyncStatus } from '../stores/syncStore'
  import { resolveConflict } from '../api/sync'
  import { currentPageId } from '../stores/pageStore'

  // For pages-table conflicts only (Phase 3 scope). Filter by current page.
  let pageConflicts = $derived(
    $syncConflicts.filter(
      (c) => c.tableName === 'pages' && c.rowId === $currentPageId,
    ),
  )

  // User's per-conflict choice. Map: conflict_id → 'local' | 'remote'.
  let choices = $state<Record<string, 'local' | 'remote'>>({})
  let busy = $state(false)
  let collapsed = $state(false)

  function parse(json: string | null) {
    if (json === null) return null
    try { return JSON.parse(json) } catch { return json }
  }

  async function applyAll() {
    busy = true
    try {
      for (const c of pageConflicts) {
        const choice = choices[c.conflictId] ?? 'local'
        await resolveConflict(c.conflictId, choice === 'local')
      }
      choices = {}
      await refreshSyncStatus()
    } finally {
      busy = false
    }
  }
</script>

{#if pageConflicts.length > 0 && !collapsed}
  <div class="resolver" data-testid="conflict-resolver">
    <div class="resolver-head">
      <span class="resolver-title">
        ⚠ {pageConflicts.length} field conflict{pageConflicts.length === 1 ? '' : 's'} on this page
      </span>
      <button class="resolver-collapse" onclick={() => collapsed = true} title="Hide">×</button>
    </div>
    <p class="resolver-sub">
      This page was edited on another device. Pick which version to keep for each field.
    </p>
    {#each pageConflicts as c (c.conflictId)}
      <div class="conflict-row">
        <div class="conflict-field">{c.fieldName}</div>
        <label class="choice" class:selected={(choices[c.conflictId] ?? 'local') === 'local'}>
          <input type="radio" name={c.conflictId} value="local"
                 checked={(choices[c.conflictId] ?? 'local') === 'local'}
                 onchange={() => choices = { ...choices, [c.conflictId]: 'local' }}/>
          <span class="choice-side">This device</span>
          <span class="choice-value">{parse(c.localValue) ?? '(empty)'}</span>
        </label>
        <label class="choice" class:selected={choices[c.conflictId] === 'remote'}>
          <input type="radio" name={c.conflictId} value="remote"
                 checked={choices[c.conflictId] === 'remote'}
                 onchange={() => choices = { ...choices, [c.conflictId]: 'remote' }}/>
          <span class="choice-side">Other device</span>
          <span class="choice-value">{parse(c.remoteValue) ?? '(empty)'}</span>
        </label>
      </div>
    {/each}
    <div class="resolver-actions">
      <button class="apply-btn" onclick={applyAll} disabled={busy}>
        {busy ? 'Applying…' : 'Apply resolution'}
      </button>
    </div>
  </div>
{/if}

<style>
  .resolver {
    background: var(--color-surface-card);
    border: 2px solid var(--color-status-warning);
    border-radius: var(--radius-md);
    padding: 14px 16px;
    margin: 12px 16px;
  }
  .resolver-head {
    display: flex; align-items: center; justify-content: space-between;
    margin-bottom: 4px;
  }
  .resolver-title {
    font-family: var(--font-ui); font-weight: 600; font-size: 14px;
    color: var(--color-status-warning);
  }
  .resolver-collapse {
    background: none; border: none; color: var(--color-fg-tertiary);
    font-size: 18px; cursor: pointer; padding: 0 4px;
  }
  .resolver-sub {
    margin: 0 0 12px; font-family: var(--font-ui); font-size: 12px;
    color: var(--color-fg-tertiary);
  }
  .conflict-row {
    display: grid; grid-template-columns: 120px 1fr 1fr;
    gap: 10px; align-items: center;
    padding: 8px 0;
    border-top: 1px solid var(--color-border-subtle);
  }
  .conflict-field {
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 12px; color: var(--color-accent-gold); font-weight: 600;
  }
  .choice {
    display: flex; flex-direction: column; gap: 2px;
    padding: 8px 10px; border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm); cursor: pointer;
    background: var(--color-surface-tertiary);
  }
  .choice.selected { border-color: var(--color-accent-gold); background: var(--color-accent-gold-subtle); }
  .choice input { display: none; }
  .choice-side {
    font-family: var(--font-ui); font-size: 11px; font-weight: 600;
    text-transform: uppercase; color: var(--color-fg-tertiary);
  }
  .choice-value {
    font-family: var(--font-ui); font-size: 13px; color: var(--color-fg-primary);
    word-break: break-word;
  }
  .resolver-actions {
    display: flex; justify-content: flex-end; margin-top: 12px;
  }
  .apply-btn {
    padding: 8px 16px;
    background: var(--color-accent-gold); color: var(--color-fg-inverse);
    border: none; border-radius: var(--radius-sm);
    font-family: var(--font-ui); font-weight: 600; font-size: 13px;
    cursor: pointer;
  }
  .apply-btn:disabled { opacity: 0.6; cursor: not-allowed; }
</style>
