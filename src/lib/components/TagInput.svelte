<script lang="ts">
  import { callCommand } from '../api/bridge'
  import { currentPageId } from '../stores/pageStore'

  interface Tag {
    id: string
    name: string
    color: string | null
  }

  let pageTags = $state<Tag[]>([])
  let allTags = $state<Tag[]>([])
  let inputValue = $state('')
  let showSuggestions = $state(false)

  let filteredSuggestions = $derived(
    inputValue.trim()
      ? allTags
          .filter((t) => t.name.toLowerCase().includes(inputValue.toLowerCase()))
          .filter((t) => !pageTags.some((pt) => pt.id === t.id))
          .slice(0, 5)
      : []
  )

  $effect(() => {
    const pageId = $currentPageId
    if (pageId) {
      loadTags(pageId)
    } else {
      pageTags = []
    }
  })

  async function loadTags(pageId: string) {
    try {
      pageTags = await callCommand('get_page_tags', { page_id: pageId, pageId })
      allTags = await callCommand('list_tags')
    } catch {
      pageTags = []
    }
  }

  async function addTag() {
    const name = inputValue.trim()
    if (!name || !$currentPageId) return

    let tag = allTags.find((t) => t.name.toLowerCase() === name.toLowerCase())
    if (!tag) {
      tag = await callCommand<typeof allTags[number]>('create_tag', { name, color: null })
      if (!tag) return
      allTags = [...allTags, tag]
    }

    if (!pageTags.some((t) => t.id === tag.id)) {
      await callCommand('add_tag_to_page', { page_id: $currentPageId, tag_id: tag.id, pageId: $currentPageId, tagId: tag.id })
      pageTags = [...pageTags, tag]
    }

    inputValue = ''
    showSuggestions = false
  }

  async function removeTag(tagId: string) {
    if (!$currentPageId) return
    await callCommand('remove_tag_from_page', { page_id: $currentPageId, tag_id: tagId, pageId: $currentPageId, tagId })
    pageTags = pageTags.filter((t) => t.id !== tagId)
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault()
      if (filteredSuggestions.length > 0) {
        selectSuggestion(filteredSuggestions[0])
      } else {
        addTag()
      }
    }
  }

  async function selectSuggestion(tag: Tag) {
    if (!$currentPageId) return
    if (!pageTags.some((t) => t.id === tag.id)) {
      await callCommand('add_tag_to_page', { page_id: $currentPageId, tag_id: tag.id, pageId: $currentPageId, tagId: tag.id })
      pageTags = [...pageTags, tag]
    }
    inputValue = ''
    showSuggestions = false
  }
</script>

<div class="tag-input-section" data-testid="tag-input">
  <h3 class="section-label">TAGS</h3>

  {#if pageTags.length > 0}
    <div class="tag-list">
      {#each pageTags as tag (tag.id)}
        <span class="tag-pill">
          {tag.name}
          <button class="tag-remove" onclick={() => removeTag(tag.id)}>&times;</button>
        </span>
      {/each}
    </div>
  {/if}

  <div class="tag-input-wrapper">
    <input
      class="tag-input"
      placeholder="Add tag..."
      bind:value={inputValue}
      onkeydown={handleKeydown}
      onfocus={() => showSuggestions = true}
      onblur={() => setTimeout(() => showSuggestions = false, 200)}
    />
    {#if showSuggestions && filteredSuggestions.length > 0}
      <div class="tag-suggestions">
        {#each filteredSuggestions as suggestion (suggestion.id)}
          <button class="tag-suggestion-item" onmousedown={(e) => e.preventDefault()} onclick={() => selectSuggestion(suggestion)}>
            {suggestion.name}
          </button>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .tag-input-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .section-label {
    font-family: var(--font-ui);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 2px;
    color: var(--color-fg-tertiary);
    margin: 0;
  }

  .tag-list {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .tag-pill {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    background: var(--color-surface-tertiary);
    border-radius: 4px;
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-fg-secondary);
  }

  .tag-remove {
    background: none;
    border: none;
    color: var(--color-fg-tertiary);
    cursor: pointer;
    font-size: 14px;
    padding: 0 2px;
    line-height: 1;
  }

  .tag-remove:hover {
    color: var(--color-status-error);
  }

  .tag-input-wrapper {
    position: relative;
  }

  .tag-input {
    width: 100%;
    padding: 6px 10px;
    background: var(--color-surface-primary);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-primary);
    outline: none;
  }

  .tag-input::placeholder {
    color: var(--color-fg-tertiary);
  }

  .tag-input:focus {
    border-color: var(--color-accent-gold);
  }

  .tag-suggestions {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    margin-top: 4px;
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
    z-index: 10;
  }

  .tag-suggestion-item {
    display: block;
    width: 100%;
    padding: 6px 10px;
    background: none;
    border: none;
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-primary);
    text-align: left;
    cursor: pointer;
  }

  .tag-suggestion-item:hover {
    background: var(--color-accent-gold-subtle);
  }
</style>
