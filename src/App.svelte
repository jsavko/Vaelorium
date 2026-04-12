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
  // Active module — switching preserves each module's sub-state
  let activeModule = $state<'wiki' | 'atlas' | 'chronicle' | 'relations' | 'entity-list'>('wiki')
  let activeTypeListId = $state<string | null>(null)
  let activeMapId = $state<string | null>(null)

  // When a page is selected (from sidebar tree, pin click, etc.), switch to wiki
  let prevPageId: string | null = null
  $effect(() => {
    const pageId = $currentPageId
    if (pageId && pageId !== prevPageId) {
      activeModule = 'wiki'
    }
    prevPageId = pageId
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
      onSelectType={(typeId) => { activeTypeListId = typeId; activeModule = 'entity-list' }}
      activeTypeId={activeModule === 'entity-list' ? activeTypeListId : null}
      onOpenGraph={() => { activeModule = 'relations' }}
      onOpenAtlas={() => { activeModule = 'atlas' }}
      onOpenChronicle={() => { activeModule = 'chronicle' }}
      chronicleActive={activeModule === 'chronicle'}
      graphActive={activeModule === 'relations'}
      atlasActive={activeModule === 'atlas'}
      onCloseTome={async () => { await closeTome(); await loadRecentTomes() }}
    />
    {#if activeModule === 'chronicle'}
      <ChronicleView onClose={() => activeModule = 'wiki'} />
    {:else if activeModule === 'atlas' && activeMapId}
      <MapViewer mapId={activeMapId} onClose={() => activeMapId = null} />
    {:else if activeModule === 'atlas'}
      <MapList
        onOpenMap={(id) => activeMapId = id}
        onClose={() => activeModule = 'wiki'}
      />
    {:else if activeModule === 'relations'}
      <GraphView onClose={() => activeModule = 'wiki'} />
    {:else if activeModule === 'entity-list' && activeTypeListId}
      <EntityListView
        entityTypeId={activeTypeListId}
        onOpenNewPage={() => { newPageInitialTypeId = activeTypeListId; newPageModalOpen = true }}
        onClose={() => { activeTypeListId = null; activeModule = 'wiki' }}
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
