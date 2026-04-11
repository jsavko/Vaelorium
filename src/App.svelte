<script lang="ts">
  import Sidebar from './lib/components/Sidebar.svelte'
  import MainContent from './lib/components/MainContent.svelte'
  import EntityListView from './lib/components/EntityListView.svelte'
  import GraphView from './lib/components/GraphView.svelte'
  import MapList from './lib/components/MapList.svelte'
  import MapViewer from './lib/components/MapViewer.svelte'
  import ChronicleView from './lib/components/ChronicleView.svelte'
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
  import { isTomeOpen, currentTome, currentTomeMetadata, loadRecentTomes, closeTome } from './lib/stores/tomeStore'
  import { getTomeMetadata } from './lib/api/tomes'
  import { isTauri } from './lib/api/bridge'
  import { matchesKeybind } from './lib/utils/keybinds'

  onMount(async () => {
    if (isTauri) {
      await loadRecentTomes()
      // Check if a Tome was auto-opened (legacy migration)
      try {
        const meta = await getTomeMetadata()
        if (meta) {
          isTomeOpen.set(true)
          currentTomeMetadata.set(meta)
          currentTome.set({ path: '', name: meta.name, description: meta.description })
        }
      } catch {
        // No Tome open — stay on Tome Picker
      }
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
  let newPageInitialTypeId = $state<string | null>(null)
  let createTomeModalOpen = $state(false)
  let activeTypeListId = $state<string | null>(null)
  let graphViewOpen = $state(false)
  let atlasOpen = $state(false)
  let activeMapId = $state<string | null>(null)
  let chronicleOpen = $state(false)

  // Clear entity list view when a page is selected
  $effect(() => {
    if ($currentPageId) {
      activeTypeListId = null
      graphViewOpen = false
      atlasOpen = false
      activeMapId = null
      chronicleOpen = false
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
      newPageInitialTypeId = null
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
      onNewPage={() => { newPageInitialTypeId = null; newPageModalOpen = true }}
      onSelectType={(typeId) => { activeTypeListId = typeId; graphViewOpen = false; detailsOpen = false }}
      activeTypeId={activeTypeListId}
      onOpenGraph={() => { graphViewOpen = true; activeTypeListId = null; atlasOpen = false; activeMapId = null; detailsOpen = false }}
      onOpenAtlas={() => { atlasOpen = true; activeMapId = null; graphViewOpen = false; chronicleOpen = false; activeTypeListId = null; detailsOpen = false }}
      onOpenChronicle={() => { chronicleOpen = true; graphViewOpen = false; atlasOpen = false; activeMapId = null; activeTypeListId = null; detailsOpen = false }}
      chronicleActive={chronicleOpen}
      graphActive={graphViewOpen}
      atlasActive={atlasOpen || !!activeMapId}
      onCloseTome={async () => { await closeTome(); await loadRecentTomes() }}
    />
    {#if chronicleOpen}
      <ChronicleView onClose={() => chronicleOpen = false} />
    {:else if activeMapId}
      <MapViewer mapId={activeMapId} onClose={() => { activeMapId = null; atlasOpen = true }} />
    {:else if atlasOpen}
      <MapList
        onOpenMap={(id) => { activeMapId = id; atlasOpen = false }}
        onClose={() => atlasOpen = false}
      />
    {:else if graphViewOpen}
      <GraphView onClose={() => graphViewOpen = false} />
    {:else if activeTypeListId}
      <EntityListView
        entityTypeId={activeTypeListId}
        onOpenNewPage={() => { newPageInitialTypeId = activeTypeListId; newPageModalOpen = true }}
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
    onClose={() => { newPageModalOpen = false; newPageInitialTypeId = null }}
    initialTypeId={newPageInitialTypeId}
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
