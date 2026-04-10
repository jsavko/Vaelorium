import { describe, it, expect, beforeEach } from 'vitest'
import { callCommand } from './bridge'

describe('bridge mock backend', () => {
  // Each test gets a fresh state since the mock is module-level
  // We'll create pages fresh in each test

  it('creates a page and returns it', async () => {
    const page = await callCommand('create_page', {
      input: { title: 'Elara Nightwhisper' },
    })
    expect(page).toHaveProperty('id')
    expect(page).toHaveProperty('title', 'Elara Nightwhisper')
    expect(page).toHaveProperty('visibility', 'private')
    expect(page).toHaveProperty('parent_id', null)
  })

  it('gets a page by id', async () => {
    const created: any = await callCommand('create_page', {
      input: { title: 'Moonwell Sanctum' },
    })
    const fetched: any = await callCommand('get_page', { id: created.id })
    expect(fetched.title).toBe('Moonwell Sanctum')
    expect(fetched.id).toBe(created.id)
  })

  it('updates a page title', async () => {
    const created: any = await callCommand('create_page', {
      input: { title: 'Draft' },
    })
    const updated: any = await callCommand('update_page', {
      id: created.id,
      input: { title: 'The Silver Flame' },
    })
    expect(updated.title).toBe('The Silver Flame')
  })

  it('deletes a page', async () => {
    const created: any = await callCommand('create_page', {
      input: { title: 'To Delete' },
    })
    await callCommand('delete_page', { id: created.id })
    await expect(callCommand('get_page', { id: created.id })).rejects.toThrow()
  })

  it('lists all pages', async () => {
    await callCommand('create_page', { input: { title: 'Page A' } })
    await callCommand('create_page', { input: { title: 'Page B' } })
    const pages: any[] = await callCommand('list_pages')
    const titles = pages.map((p) => p.title)
    expect(titles).toContain('Page A')
    expect(titles).toContain('Page B')
  })

  it('returns page tree with children count', async () => {
    const parent: any = await callCommand('create_page', {
      input: { title: 'Characters' },
    })
    await callCommand('create_page', {
      input: { title: 'Elara', parent_id: parent.id },
    })
    await callCommand('create_page', {
      input: { title: 'Theron', parent_id: parent.id },
    })
    const tree: any[] = await callCommand('get_page_tree')
    const parentNode = tree.find((n) => n.id === parent.id)
    expect(parentNode.children_count).toBe(2)
  })

  it('saves and retrieves page content', async () => {
    const page: any = await callCommand('create_page', {
      input: { title: 'Test Content' },
    })
    const content = [1, 2, 3, 4, 5]
    await callCommand('save_page_content', {
      pageId: page.id,
      yjsState: content,
    })
    const retrieved: any = await callCommand('get_page_content', {
      pageId: page.id,
    })
    expect(retrieved).toEqual(content)
  })

  it('searches pages by title', async () => {
    await callCommand('create_page', { input: { title: 'UniqueSearchTarget_XYZ' } })
    await callCommand('create_page', { input: { title: 'Moonwell Sanctum' } })
    const results: any[] = await callCommand('search_pages', { query: 'UniqueSearchTarget' })
    expect(results.length).toBe(1)
    expect(results[0].title).toBe('UniqueSearchTarget_XYZ')
  })

  it('returns empty results for non-matching search', async () => {
    const results: any[] = await callCommand('search_pages', { query: 'zzzzz' })
    expect(results.length).toBe(0)
  })

  it('creates and lists tags', async () => {
    const tag: any = await callCommand('create_tag', { name: 'NPC', color: '#B85C5C' })
    expect(tag.name).toBe('NPC')
    const tags: any[] = await callCommand('list_tags')
    expect(tags.some((t) => t.name === 'NPC')).toBe(true)
  })

  it('adds and retrieves tags for a page', async () => {
    const page: any = await callCommand('create_page', { input: { title: 'Tagged Page' } })
    const tag: any = await callCommand('create_tag', { name: 'Important' })
    await callCommand('add_tag_to_page', { page_id: page.id, tag_id: tag.id })
    const pageTags: any[] = await callCommand('get_page_tags', { page_id: page.id })
    expect(pageTags.length).toBe(1)
    expect(pageTags[0].name).toBe('Important')
  })

  it('tracks backlinks from wiki links', async () => {
    const pageA: any = await callCommand('create_page', { input: { title: 'Page A' } })
    const pageB: any = await callCommand('create_page', { input: { title: 'Page B' } })
    await callCommand('save_wiki_links', {
      source_page_id: pageA.id,
      links: [{ target_page_id: pageB.id, link_text: 'Page B' }],
    })
    const backlinks: any[] = await callCommand('get_backlinks', { pageId: pageB.id })
    expect(backlinks.length).toBe(1)
    expect(backlinks[0].title).toBe('Page A')
  })

  it('creates and lists version history', async () => {
    const page: any = await callCommand('create_page', { input: { title: 'Versioned' } })
    await callCommand('create_version', {
      pageId: page.id,
      yjsSnapshot: [1, 2, 3],
      summary: 'Initial save',
    })
    const versions: any[] = await callCommand('list_versions', { pageId: page.id })
    expect(versions.length).toBe(1)
    expect(versions[0].summary).toBe('Initial save')
    expect(versions[0].version_number).toBe(1)
  })
})
