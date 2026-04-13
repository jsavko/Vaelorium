import { writable } from 'svelte/store'
import type { Board, BoardCard, BoardConnector } from '../api/boards'
import * as api from '../api/boards'

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
  await api.deleteCard(id)
  currentCards.update((c) => c.filter((card) => card.id !== id))
  currentConnectors.update((c) => c.filter((conn) => conn.source_card_id !== id && conn.target_card_id !== id))
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
