<script lang="ts">
  import { settings, updateKeybind, resetKeybinds } from '../stores/settingsStore'

  let editingKeybind = $state<string | null>(null)
  let listeningForKey = $state(false)

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

<style>
  .reset-btn {
    background: none; border: none;
    font-family: var(--font-ui); font-size: 12px;
    color: var(--color-accent-gold); cursor: pointer;
  }
  .keybind-list { display: flex; flex-direction: column; gap: 4px; }
  .keybind-row {
    display: flex; align-items: center; justify-content: space-between;
    padding: 8px 12px; border-radius: var(--radius-sm);
  }
  .keybind-row:hover { background: var(--color-surface-tertiary); }
  .keybind-label { font-family: var(--font-ui); font-size: 14px; color: var(--color-fg-primary); }
  .keybind-value {
    background: var(--color-surface-tertiary);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    padding: 4px 12px;
    font-family: var(--font-ui); font-size: 12px;
    color: var(--color-fg-secondary); cursor: pointer;
    min-width: 80px; text-align: center;
  }
  .keybind-value:hover { border-color: var(--color-accent-gold); }
  .keybind-input {
    background: var(--color-accent-gold-subtle);
    border: 1px solid var(--color-accent-gold);
    border-radius: var(--radius-sm);
    padding: 4px 12px;
    font-family: var(--font-ui); font-size: 12px;
    color: var(--color-accent-gold);
    min-width: 80px; text-align: center; outline: none;
  }
</style>
