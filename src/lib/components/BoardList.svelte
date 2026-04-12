<script lang="ts">
  import { Layout, Plus } from 'lucide-svelte'
  import InputModal from './InputModal.svelte'
  import { boards, loadBoards, createBoard } from '../stores/boardStore'
  import { onMount } from 'svelte'

  interface Props {
    onOpenBoard: (boardId: string) => void
    onClose: () => void
  }

  let { onOpenBoard, onClose }: Props = $props()
  let nameModalOpen = $state(false)

  onMount(() => loadBoards())

  async function handleCreate(name: string) {
    nameModalOpen = false
    const board = await createBoard(name)
    onOpenBoard(board.id)
  }
</script>

<div class="board-list-view">
  <header class="list-header">
    <div class="header-left">
      <button class="back-btn" onclick={onClose} aria-label="Back">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"></polyline></svg>
      </button>
      <Layout size={20} />
      <h2 class="header-title">Boards</h2>
      <span class="header-count">{$boards.length}</span>
    </div>
    <button class="new-btn" onclick={() => nameModalOpen = true}>+ New Board</button>
  </header>

  {#if $boards.length === 0}
    <div class="empty-state">
      <Layout size={48} />
      <p>No boards yet</p>
      <button class="empty-create" onclick={() => nameModalOpen = true}>Create your first board</button>
    </div>
  {:else}
    <div class="board-grid">
      {#each $boards as board (board.id)}
        <button class="board-card" onclick={() => onOpenBoard(board.id)}>
          <div class="card-cover"><Layout size={28} /></div>
          <div class="card-body"><h3 class="card-title">{board.name}</h3></div>
        </button>
      {/each}
      <button class="board-card new-card" onclick={() => nameModalOpen = true}>
        <div class="new-content"><Plus size={24} /><span>New Board</span></div>
      </button>
    </div>
  {/if}
</div>

<InputModal open={nameModalOpen} title="New Board" placeholder="Board name..." confirmLabel="Create" onConfirm={handleCreate} onCancel={() => nameModalOpen = false} />

<style>
  .board-list-view { flex: 1; display: flex; flex-direction: column; height: 100%; overflow: hidden; }
  .list-header { display: flex; align-items: center; justify-content: space-between; padding: 12px 24px; background: var(--color-surface-secondary); border-bottom: 1px solid var(--color-border-subtle); flex-shrink: 0; }
  .header-left { display: flex; align-items: center; gap: 10px; color: var(--color-fg-tertiary); }
  .back-btn { background: none; border: none; color: var(--color-fg-tertiary); cursor: pointer; padding: 4px; border-radius: var(--radius-sm); }
  .back-btn:hover { background: var(--color-surface-tertiary); color: var(--color-fg-primary); }
  .header-title { font-family: var(--font-heading); font-size: 20px; font-weight: 600; color: var(--color-fg-primary); margin: 0; }
  .header-count { font-family: var(--font-ui); font-size: 12px; color: var(--color-fg-tertiary); background: var(--color-surface-tertiary); padding: 2px 8px; border-radius: 10px; }
  .new-btn { padding: 6px 16px; background: var(--color-accent-gold); border: none; border-radius: var(--radius-sm); font-family: var(--font-ui); font-size: 13px; font-weight: 600; color: var(--color-fg-inverse); cursor: pointer; }
  .empty-state { display: flex; flex-direction: column; align-items: center; justify-content: center; flex: 1; gap: 12px; color: var(--color-fg-tertiary); }
  .empty-create { padding: 8px 20px; background: var(--color-accent-gold); border: none; border-radius: var(--radius-sm); font-family: var(--font-ui); font-size: 13px; font-weight: 600; color: var(--color-fg-inverse); cursor: pointer; }
  .board-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px; padding: 24px; overflow-y: auto; flex: 1; }
  .board-card { background: var(--color-surface-card); border: 1px solid var(--color-border-default); border-radius: var(--radius-md); overflow: hidden; cursor: pointer; text-align: left; display: flex; flex-direction: column; }
  .board-card:hover { border-color: var(--color-accent-gold); }
  .card-cover { height: 80px; display: flex; align-items: center; justify-content: center; background: var(--color-surface-tertiary); color: var(--color-fg-tertiary); opacity: 0.3; }
  .card-body { padding: 12px; }
  .card-title { font-family: var(--font-heading); font-size: 15px; font-weight: 600; color: var(--color-fg-primary); margin: 0; }
  .new-card { border-style: dashed; }
  .new-content { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; flex: 1; padding: 24px; color: var(--color-accent-gold); font-family: var(--font-ui); font-size: 13px; font-weight: 600; }
</style>
