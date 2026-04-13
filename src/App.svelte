<script lang="ts">
  import Sidebar from './lib/components/Sidebar.svelte'
  import MainContent from './lib/components/MainContent.svelte'
  import EntityListView from './lib/components/EntityListView.svelte'
  import GraphView from './lib/components/GraphView.svelte'
  import MapList from './lib/components/MapList.svelte'
  import MapViewer from './lib/components/MapViewer.svelte'
  import ChronicleView from './lib/components/ChronicleView.svelte'
  import BoardList from './lib/components/BoardList.svelte'
  import BoardView from './lib/components/BoardView.svelte'
  import DetailsPanel from './lib/components/DetailsPanel.svelte'
  import SearchOverlay from './lib/components/SearchOverlay.svelte'
  import SlashMenu from './lib/components/SlashMenu.svelte'
  import MentionSuggestion from './lib/components/MentionSuggestion.svelte'
  import ToastContainer from './lib/components/ToastContainer.svelte'
  import NewPageModal from './lib/components/NewPageModal.svelte'
  import TomePicker from './lib/components/TomePicker.svelte'
  import CreateTomeModal from './lib/components/CreateTomeModal.svelte'
  import Settings from './lib/components/Settings.svelte'
  import UpdateNotification from './lib/components/UpdateNotification.svelte'
  import ConflictResolver from './lib/components/ConflictResolver.svelte'
  import SyncUnlockModal from './lib/components/SyncUnlockModal.svelte'
  import { initSyncStore, syncStatus } from './lib/stores/syncStore'
  import { onMount } from 'svelte'
  import { createPage, currentPageId } from './lib/stores/pageStore'
  import { settings } from './lib/stores/settingsStore'
  import { loadEntityTypes } from './lib/stores/entityTypeStore'
  import { loadMaps } from './lib/stores/mapStore'
  import { loadTimelines } from './lib/stores/timelineStore'
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
      // Initialize sync store + event subscription (no-op in browser mock).
      initSyncStore().catch(() => {})
    } else {
      // In browser mock, a Tome is always "open"
      isTomeOpen.set(true)
      await loadEntityTypes()
      await loadMaps()
      await loadTimelines()
    }
  })

  // Load entity types when a Tome is opened
  $effect(() => {
    if ($isTomeOpen) {
      loadEntityTypes()
      loadMaps()
      loadTimelines()
    }
  })

  let searchOpen = $state(false)
  let detailsOpen = $state(false)
  let settingsOpen = $state(false)
  let settingsInitialTab = $state<string | undefined>(undefined)
  let syncUnlockOpen = $state(false)

  // Don't auto-open the unlock modal on launch — it covered the locked
  // pill (its z-index is higher than the sidebar's), defeating the
  // visibility the pill was added to provide. The pill stays visible;
  // user clicks it to open the unlock dialog.
  let newPageModalOpen = $state(false)
  let newPageInitialTypeId = $state<string | null>(null)
  let createTomeModalOpen = $state(false)
  // Active module — switching preserves each module's sub-state
  let activeModule = $state<'wiki' | 'atlas' | 'chronicle' | 'relations' | 'entity-list' | 'boards'>('wiki')
  let activeTypeListId = $state<string | null>(null)
  let activeMapId = $state<string | null>(null)
  let activeBoardId = $state<string | null>(null)

  function switchToWiki() {
    activeModule = 'wiki'
  }

  // Listen for cross-module navigation events
  $effect(() => {
    function handleOpenMap(e: Event) {
      const { mapId } = (e as CustomEvent).detail
      activeMapId = mapId
      activeModule = 'atlas'
    }
    function handleOpenTimeline(e: Event) {
      activeModule = 'chronicle'
    }
    window.addEventListener('vaelorium:page-selected', switchToWiki)
    window.addEventListener('vaelorium:open-map', handleOpenMap)
    window.addEventListener('vaelorium:open-timeline', handleOpenTimeline)
    return () => {
      window.removeEventListener('vaelorium:page-selected', switchToWiki)
      window.removeEventListener('vaelorium:open-map', handleOpenMap)
      window.removeEventListener('vaelorium:open-timeline', handleOpenTimeline)
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
      onOpenSettings={(tab?: string) => { settingsInitialTab = tab; settingsOpen = true }}
      onOpenUnlock={() => syncUnlockOpen = true}
      onNewPage={() => { newPageInitialTypeId = null; newPageModalOpen = true }}
      onSelectType={(typeId) => { activeTypeListId = typeId; activeModule = 'entity-list' }}
      activeTypeId={activeModule === 'entity-list' ? activeTypeListId : null}
      onOpenGraph={() => { activeModule = 'relations' }}
      onOpenAtlas={() => { activeModule = 'atlas' }}
      onOpenChronicle={() => { activeModule = 'chronicle' }}
      chronicleActive={activeModule === 'chronicle'}
      onOpenBoards={() => { activeModule = 'boards' }}
      boardsActive={activeModule === 'boards'}
      graphActive={activeModule === 'relations'}
      atlasActive={activeModule === 'atlas'}
      onOpenWiki={switchToWiki}
      wikiActive={activeModule === 'wiki'}
      onCloseTome={async () => { await closeTome(); await loadRecentTomes() }}
    />
    {#if activeModule === 'boards' && activeBoardId}
      <BoardView boardId={activeBoardId} onClose={() => activeBoardId = null} />
    {:else if activeModule === 'boards'}
      <BoardList onOpenBoard={(id) => activeBoardId = id} onClose={() => activeModule = 'wiki'} />
    {:else if activeModule === 'chronicle'}
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
      <div class="main-with-resolver">
        <ConflictResolver />
        <MainContent onToggleDetails={() => detailsOpen = !detailsOpen} {detailsOpen} />
      </div>
      <DetailsPanel open={detailsOpen} onClose={() => detailsOpen = false} />
    {/if}
  </div>

  <SearchOverlay open={searchOpen} onClose={() => searchOpen = false} />
  <SlashMenu />
  <MentionSuggestion />
  <Settings open={settingsOpen} initialTab={settingsInitialTab} onClose={() => settingsOpen = false} />
  <SyncUnlockModal open={syncUnlockOpen} onClose={() => syncUnlockOpen = false} />
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
{#if $isTomeOpen}
  <UpdateNotification />
{/if}

<style>
  .app-layout {
    display: flex;
    width: 100%;
    height: 100%;
  }

  /* Wrapper so ConflictResolver can sit above MainContent. Takes over
     MainContent's former role as the flex-1 child of app-layout; the
     flex-column + min-width:0 lets MainContent's own flex: 1 continue
     to fill vertical space and prevents content from blowing out width. */
  .main-with-resolver {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }
</style>
