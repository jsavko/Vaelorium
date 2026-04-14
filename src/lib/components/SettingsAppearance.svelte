<script lang="ts">
  import { settings, updateAppearance } from '../stores/settingsStore'

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
</script>

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

<style>
  .theme-list { display: flex; flex-direction: column; gap: 8px; }
  .theme-card {
    display: flex; align-items: center; gap: 14px;
    padding: 12px 14px;
    background: var(--color-surface-tertiary);
    border: 2px solid transparent;
    border-radius: var(--radius-md);
    cursor: pointer; text-align: left;
  }
  .theme-card:hover { border-color: var(--color-border-strong); }
  .theme-card.active { border-color: var(--color-accent-gold); }
  .theme-swatches {
    display: flex; gap: 0;
    border-radius: var(--radius-sm); overflow: hidden;
    border: 1px solid var(--color-border-default); flex-shrink: 0;
  }
  .theme-swatch { width: 14px; height: 36px; }
  .theme-info {
    display: flex; flex-direction: column; gap: 2px;
    flex: 1; min-width: 0;
  }
  .theme-name {
    font-family: var(--font-ui); font-size: 14px; font-weight: 600;
    color: var(--color-fg-primary);
    display: flex; align-items: center; gap: 6px;
  }
  .theme-mode {
    font-size: 10px; font-weight: 500; text-transform: uppercase;
    letter-spacing: 0.5px; color: var(--color-fg-tertiary);
    background: var(--color-surface-primary);
    padding: 2px 6px; border-radius: 3px;
  }
  .theme-desc {
    font-family: var(--font-ui); font-size: 12px;
    color: var(--color-fg-tertiary);
  }
  .font-size-row { display: flex; align-items: center; gap: 16px; }
  .font-size-slider { flex: 1; accent-color: var(--color-accent-gold); }
  .font-size-value {
    font-family: var(--font-ui); font-size: 14px;
    color: var(--color-fg-primary); min-width: 40px;
  }
</style>
