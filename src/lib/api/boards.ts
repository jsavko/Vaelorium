import { callCommand } from './bridge'

export interface Board { id: string; name: string; sort_order: number; created_at: string; updated_at: string }
export interface BoardCard { id: string; board_id: string; page_id: string | null; content: string | null; x: number; y: number; width: number; height: number; color: string | null; created_at: string }
export interface BoardConnector { id: string; board_id: string; source_card_id: string; target_card_id: string; label: string | null; color: string | null; created_at: string }

export async function createBoard(name: string): Promise<Board> { return callCommand('create_board', { name }) }
export async function listBoards(): Promise<Board[]> { return callCommand('list_boards') }
export async function deleteBoard(id: string): Promise<void> { return callCommand('delete_board', { id }) }
export async function createCard(boardId: string, x: number, y: number, content?: string | null, pageId?: string | null, color?: string | null): Promise<BoardCard> { return callCommand('create_card', { boardId, x, y, content, pageId, color }) }
export async function updateCard(id: string, updates: { x?: number; y?: number; content?: string; pageId?: string; color?: string; width?: number; height?: number }): Promise<BoardCard> { return callCommand('update_card', { id, ...updates }) }
export async function deleteCard(id: string): Promise<void> { return callCommand('delete_card', { id }) }
export async function getBoardCards(boardId: string): Promise<BoardCard[]> { return callCommand('get_board_cards', { boardId }) }
export async function createConnector(boardId: string, sourceCardId: string, targetCardId: string, label?: string | null, color?: string | null): Promise<BoardConnector> { return callCommand('create_connector', { boardId, sourceCardId, targetCardId, label, color }) }
export async function deleteConnector(id: string): Promise<void> { return callCommand('delete_connector', { id }) }
export async function getBoardConnectors(boardId: string): Promise<BoardConnector[]> { return callCommand('get_board_connectors', { boardId }) }
