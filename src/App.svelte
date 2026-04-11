<script lang="ts">
  import Sidebar from './lib/components/Sidebar.svelte'
  import MainContent from './lib/components/MainContent.svelte'
  import EntityListView from './lib/components/EntityListView.svelte'
  import DetailsPanel from './lib/components/DetailsPanel.svelte'
  import SearchOverlay from './lib/components/SearchOverlay.svelte'
  import SlashMenu from './lib/components/SlashMenu.svelte'
  import MentionSuggestion from './lib/components/MentionSuggestion.svelte'
  import ToastContainer from './lib/components/ToastContainer.svelte'
  import NewPageModal from './lib/components/NewPageModal.svelte'
  import TomePicker from './lib/components/TomePicker.svelte'
  import CreateTomeModal from './lib/components/CreateTomeModal.svelte'
  import Settings from './lib/components/Settings.svelte'
  import { onMount } from 'svelte'
  import { createPage, currentPageId } from './lib/stores/pageStore'
  import { settings } from './lib/stores/settingsStore'
  import { loadEntityTypes } from './lib/stores/entityTypeStore'
  import { isTomeOpen, loadRecentTomes } from './lib/stores/tomeStore'
  import { isTauri } from './lib/api/bridge'
  import { matchesKeybind } from './lib/utils/keybinds'

  onMount(async () => {
    if (isTauri) {
      // In Tauri, start on Tome Picker — no DB until a Tome is opened
      await loadRecentTomes()
    } else {
      // In browser mock, a Tome is always "open"
      isTomeOpen.set(true)
      await loadEntityTypes()
    }
  })

  // Load entity types when a Tome is opened
  $effect(() => {
    if ($isTomeOpen) {
      loadEntityTypes()
    }
  })

  let searchOpen = $state(false)
  let detailsOpen = $state(false)
  let settingsOpen = $state(false)
  let newPageModalOpen = $state(false)
  let createTomeModalOpen = $state(false)
  let activeTypeListId = $state<string | null>(null)

  // Clear entity list view when a page is selected
  $effect(() => {
    if ($currentPageId) {
      activeTypeListId = null
    }
  })

  function getKeybindCombo(id: string): string {
    return $settings.keybinds.find((k) => k.id === id)?.keys || ''
  }

  function handleKeydown(e: KeyboardEvent) {
    if (settingsOpen || !$isTomeOpen) return

    const searchCombo = getKeybindCombo('search')
    const newPageCombo = getKeybindCombo('new-page')
    const detailsCombo = getKeybindCombo('toggle-details')

    if (searchCombo && matchesKeybind(e, searchCombo)) {
      e.preventDefault()
      searchOpen = !searchOpen
    }
    if (newPageCombo && matchesKeybind(e, newPageCombo)) {
      e.preventDefault()
      newPageModalOpen = true
    }
    if (detailsCombo && matchesKeybind(e, detailsCombo)) {
      e.preventDefault()
      detailsOpen = !detailsOpen
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if $isTomeOpen}
  <div class="app-layout">
    <Sidebar
      onOpenSettings={() => settingsOpen = true}
      onNewPage={() => newPageModalOpen = true}
      onSelectType={(typeId) => { activeTypeListId = typeId; detailsOpen = false }}
      activeTypeId={activeTypeListId}
      onCloseTome={() => { isTomeOpen.set(false); loadRecentTomes() }}
    />
    {#if activeTypeListId}
      <EntityListView
        entityTypeId={activeTypeListId}
        onOpenNewPage={() => newPageModalOpen = true}
        onClose={() => activeTypeListId = null}
      />
    {:else}
      <MainContent onToggleDetails={() => detailsOpen = !detailsOpen} {detailsOpen} />
      <DetailsPanel open={detailsOpen} onClose={() => detailsOpen = false} />
    {/if}
  </div>

  <SearchOverlay open={searchOpen} onClose={() => searchOpen = false} />
  <SlashMenu />
  <MentionSuggestion />
  <Settings open={settingsOpen} onClose={() => settingsOpen = false} />
  <NewPageModal
    open={newPageModalOpen}
    onClose={() => newPageModalOpen = false}
    onCreate={async (title, parentId, entityTypeId) => {
      newPageModalOpen = false
      await createPage(title, parentId, entityTypeId)
    }}
  />
{:else}
  <TomePicker onCreateNew={() => createTomeModalOpen = true} />
  <CreateTomeModal
    open={createTomeModalOpen}
    onClose={() => createTomeModalOpen = false}
  />
{/if}

<ToastContainer />

<style>
  .app-layout {
    display: flex;
    width: 100%;
    height: 100%;
  }
</style>
