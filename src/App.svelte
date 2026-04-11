<script lang="ts">
  import Sidebar from './lib/components/Sidebar.svelte'
  import MainContent from './lib/components/MainContent.svelte'
  import DetailsPanel from './lib/components/DetailsPanel.svelte'
  import SearchOverlay from './lib/components/SearchOverlay.svelte'
  import SlashMenu from './lib/components/SlashMenu.svelte'
  import MentionSuggestion from './lib/components/MentionSuggestion.svelte'
  import ToastContainer from './lib/components/ToastContainer.svelte'
  import Settings from './lib/components/Settings.svelte'
  import { createPage } from './lib/stores/pageStore'
  import { settings } from './lib/stores/settingsStore'
  import { matchesKeybind } from './lib/utils/keybinds'

  let searchOpen = $state(false)
  let detailsOpen = $state(false)
  let settingsOpen = $state(false)

  function getKeybindCombo(id: string): string {
    return $settings.keybinds.find((k) => k.id === id)?.keys || ''
  }

  function handleKeydown(e: KeyboardEvent) {
    // Don't handle keybinds when settings is open and listening for key input
    if (settingsOpen) return

    const searchCombo = getKeybindCombo('search')
    const newPageCombo = getKeybindCombo('new-page')
    const detailsCombo = getKeybindCombo('toggle-details')

    if (searchCombo && matchesKeybind(e, searchCombo)) {
      e.preventDefault()
      searchOpen = !searchOpen
    }
    if (newPageCombo && matchesKeybind(e, newPageCombo)) {
      e.preventDefault()
      createPage('Untitled Page')
    }
    if (detailsCombo && matchesKeybind(e, detailsCombo)) {
      e.preventDefault()
      detailsOpen = !detailsOpen
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="app-layout">
  <Sidebar onOpenSettings={() => settingsOpen = true} />
  <MainContent onToggleDetails={() => detailsOpen = !detailsOpen} {detailsOpen} />
  <DetailsPanel open={detailsOpen} onClose={() => detailsOpen = false} />
</div>

<style>
  .app-layout {
    display: flex;
    width: 100%;
    height: 100%;
  }
</style>

<SearchOverlay open={searchOpen} onClose={() => searchOpen = false} />
<SlashMenu />
<MentionSuggestion />
<Settings open={settingsOpen} onClose={() => settingsOpen = false} />
<ToastContainer />
