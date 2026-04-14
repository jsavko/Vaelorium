import { writable, get } from 'svelte/store'
import type { Board, BoardCard, BoardConnector } from '../api/boards'
import * as api from '../api/boards'
import { showToast } from './toastStore'

export const boards = writable<Board[]>([])
export const currentBoard = writable<Board | null>(null)
export const currentCards = writable<BoardCard[]>([])
export const currentConnectors = writable<BoardConnector[]>([])

export async function loadBoards() {
  const list = await api.listBoards()
  boards.set(list)
}

export async function loadBoard(id: string) {
  const list = await api.listBoards()
  const board = list.find((b) => b.id === id)
  currentBoard.set(board || null)
  const [cards, connectors] = await Promise.all([
    api.getBoardCards(id),
    api.getBoardConnectors(id),
  ])
  currentCards.set(cards)
  currentConnectors.set(connectors)
}

export async function createBoard(name: string) {
  const board = await api.createBoard(name)
  await loadBoards()
  return board
}

export async function renameBoard(id: string, name: string) {
  const updated = await api.updateBoard(id, name)
  boards.update((list) => list.map((b) => (b.id === id ? updated : b)))
  currentBoard.update((cur) => (cur && cur.id === id ? updated : cur))
}

export async function deleteBoard(id: string) {
  await api.deleteBoard(id)
  boards.update((list) => list.filter((b) => b.id !== id))
  currentBoard.update((cur) => (cur && cur.id === id ? null : cur))
}

export async function addCard(boardId: string, x: number, y: number, content?: string | null, pageId?: string | null, color?: string | null) {
  const card = await api.createCard(boardId, x, y, content, pageId, color)
  currentCards.update((c) => [...c, card])
  return card
}

export async function moveCard(id: string, x: number, y: number) {
  await api.updateCard(id, { x, y })
  currentCards.update((cards) => cards.map((c) => c.id === id ? { ...c, x, y } : c))
}

export async function updateCardContent(id: string, content: string) {
  await api.updateCard(id, { content })
  currentCards.update((cards) => cards.map((c) => c.id === id ? { ...c, content } : c))
}

export async function resizeCard(id: string, width: number, height: number) {
  await api.updateCard(id, { width, height })
  currentCards.update((cards) => cards.map((c) => c.id === id ? { ...c, width, height } : c))
}

export async function removeCard(id: string) {
  // Snapshot the card + any connectors touching it before deleting, so the
  // user can undo. Connectors cascade-delete on the backend; we track them
  // here and reconstruct on undo.
  const card = get(currentCards).find((c) => c.id === id)
  const relatedConnectors = get(currentConnectors).filter(
    (conn) => conn.source_card_id === id || conn.target_card_id === id,
  )

  await api.deleteCard(id)
  currentCards.update((c) => c.filter((c2) => c2.id !== id))
  currentConnectors.update((c) => c.filter((conn) => conn.source_card_id !== id && conn.target_card_id !== id))

  if (!card) return
  showToast('Card deleted', 'info', {
    action: {
      label: 'Undo',
      onClick: async () => { await restoreCard(card, relatedConnectors) },
    },
  })
}

/**
 * Re-create a previously-deleted card + its connectors.
 *
 * A new card id is generated server-side; the snapshot's id is used only
 * to remap connector endpoints. Connectors whose OTHER endpoint was also
 * deleted (and not yet restored) are skipped — can't reconstruct a
 * dangling connector.
 */
async function restoreCard(card: BoardCard, relatedConnectors: BoardConnector[]) {
  const newCard = await api.createCard(card.board_id, card.x, card.y, card.content, card.page_id, card.color)
  // Restore size (createCard uses defaults).
  if (card.width !== newCard.width || card.height !== newCard.height) {
    await api.updateCard(newCard.id, { width: card.width, height: card.height })
    newCard.width = card.width
    newCard.height = card.height
  }
  currentCards.update((c) => [...c, newCard])

  const surviving = new Set(get(currentCards).map((c) => c.id))
  for (const conn of relatedConnectors) {
    const sourceId = conn.source_card_id === card.id ? newCard.id : conn.source_card_id
    const targetId = conn.target_card_id === card.id ? newCard.id : conn.target_card_id
    if (!surviving.has(sourceId) || !surviving.has(targetId)) continue
    const restored = await api.createConnector(card.board_id, sourceId, targetId, conn.label, conn.color)
    currentConnectors.update((c) => [...c, restored])
  }
}

export async function addConnector(boardId: string, sourceId: string, targetId: string, label?: string | null) {
  const conn = await api.createConnector(boardId, sourceId, targetId, label)
  currentConnectors.update((c) => [...c, conn])
  return conn
}

export async function removeConnector(id: string) {
  await api.deleteConnector(id)
  currentConnectors.update((c) => c.filter((conn) => conn.id !== id))
}
