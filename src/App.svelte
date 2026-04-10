<script lang="ts">
  import Sidebar from './lib/components/Sidebar.svelte'
  import MainContent from './lib/components/MainContent.svelte'
  import DetailsPanel from './lib/components/DetailsPanel.svelte'
  import SearchOverlay from './lib/components/SearchOverlay.svelte'
  import SlashMenu from './lib/components/SlashMenu.svelte'
  import MentionSuggestion from './lib/components/MentionSuggestion.svelte'
  import ToastContainer from './lib/components/ToastContainer.svelte'
  import { createPage } from './lib/stores/pageStore'

  let searchOpen = $state(false)
  let detailsOpen = $state(false)

  function handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
      e.preventDefault()
      searchOpen = !searchOpen
    }
    if ((e.metaKey || e.ctrlKey) && e.key === '\\') {
      e.preventDefault()
      detailsOpen = !detailsOpen
    }
    if ((e.metaKey || e.ctrlKey) && e.key === 'n') {
      e.preventDefault()
      createPage('Untitled Page')
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div id="app">
  <Sidebar />
  <MainContent onToggleDetails={() => detailsOpen = !detailsOpen} {detailsOpen} />
  <DetailsPanel open={detailsOpen} onClose={() => detailsOpen = false} />
</div>

<SearchOverlay open={searchOpen} onClose={() => searchOpen = false} />
<SlashMenu />
<MentionSuggestion />
<ToastContainer />
