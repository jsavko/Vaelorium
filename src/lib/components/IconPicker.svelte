<script lang="ts">
  interface Props {
    currentIcon: string | null
    onSelect: (icon: string | null) => void
  }

  let { currentIcon, onSelect }: Props = $props()

  let open = $state(false)

  const commonEmojis = [
    '⚔️', '🏰', '📜', '🗺️', '👤', '🐉', '💎', '🔮',
    '⭐', '🌙', '🔥', '❄️', '⚡', '🌿', '💀', '👑',
    '🛡️', '🗡️', '🏹', '🧪', '📖', '🎭', '🏛️', '⛪',
    '🌊', '🏔️', '🌲', '🏜️', '🕯️', '🔔', '⚓', '🧭',
    '🦅', '🐺', '🦁', '🐍', '🕷️', '🦇', '🐎', '🦌',
    '🍺', '🗝️', '💰', '📿', '🎪', '⚒️', '🏴', '🎵',
  ]

  function select(emoji: string) {
    onSelect(emoji)
    open = false
  }

  function removeIcon() {
    onSelect(null)
    open = false
  }
</script>

<div class="icon-picker-wrapper">
  <button
    class="icon-trigger"
    onclick={(e) => { e.stopPropagation(); open = !open; }}
    title="Set page icon"
  >
    {#if currentIcon}
      <span class="current-icon">{currentIcon}</span>
    {:else}
      <span class="add-icon">+</span>
    {/if}
  </button>

  {#if open}
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="icon-dropdown" data-testid="icon-picker" onclick={(e) => e.stopPropagation()}>
      <div class="icon-header">
        <span class="icon-title">Choose icon</span>
        {#if currentIcon}
          <button class="remove-btn" onclick={removeIcon}>Remove</button>
        {/if}
      </div>
      <div class="icon-grid">
        {#each commonEmojis as emoji}
          <button
            class="icon-option"
            class:selected={emoji === currentIcon}
            onclick={() => select(emoji)}
          >
            {emoji}
          </button>
        {/each}
      </div>
    </div>
  {/if}
</div>

<svelte:window onclick={() => open = false} />

<style>
  .icon-picker-wrapper {
    position: relative;
  }

  .icon-trigger {
    background: none;
    border: 1px dashed var(--color-border-default);
    border-radius: var(--radius-sm);
    cursor: pointer;
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 20px;
  }

  .icon-trigger:hover {
    border-color: var(--color-accent-gold);
  }

  .current-icon {
    font-size: 20px;
  }

  .add-icon {
    font-size: 16px;
    color: var(--color-fg-tertiary);
  }

  .icon-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    margin-top: 4px;
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-md);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
    padding: 8px;
    width: 280px;
    z-index: 50;
  }

  .icon-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 4px 8px;
  }

  .icon-title {
    font-family: var(--font-ui);
    font-size: 12px;
    font-weight: 600;
    color: var(--color-fg-tertiary);
  }

  .remove-btn {
    background: none;
    border: none;
    font-family: var(--font-ui);
    font-size: 11px;
    color: var(--color-status-error);
    cursor: pointer;
  }

  .icon-grid {
    display: grid;
    grid-template-columns: repeat(8, 1fr);
    gap: 2px;
  }

  .icon-option {
    background: none;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: 18px;
    padding: 4px;
    text-align: center;
  }

  .icon-option:hover {
    background: var(--color-surface-tertiary);
  }

  .icon-option.selected {
    background: var(--color-accent-gold-subtle);
  }
</style>
