import { test, expect } from '@playwright/test'

test.describe('Wiki Engine', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('shows welcome screen on first load', async ({ page }) => {
    await expect(page.locator('h1:has-text("Welcome to Vaelorium")')).toBeVisible()
    await expect(page.getByRole('button', { name: 'Create your first page' })).toBeVisible()
  })

  test('creates a new page from empty state', async ({ page }) => {
    await page.getByRole('button', { name: 'Create your first page' }).click()
    const titleInput = page.locator('input.page-title')
    await expect(titleInput).toBeVisible({ timeout: 10000 })
    await expect(titleInput).toHaveValue('Untitled Page')
    await expect(page.locator('.tree-row').first()).toContainText('Untitled Page')
  })

  test('creates a new page from + button', async ({ page }) => {
    await page.locator('button[title="New page"]').click()
    await expect(page.locator('input.page-title')).toHaveValue('Untitled Page')
  })

  test('shows TipTap editor with placeholder', async ({ page }) => {
    await page.getByRole('button', { name: 'Create your first page' }).click()
    await expect(page.locator('.editor-content')).toBeVisible()
    await expect(page.locator('.editor-content .is-empty')).toBeVisible()
  })

  test('can type content in the editor', async ({ page }) => {
    await page.getByRole('button', { name: 'Create your first page' }).click()
    const editor = page.locator('.editor-content')
    await editor.click()
    await page.keyboard.type('Hello from the Vaelorium!')
    await expect(editor).toContainText('Hello from the Vaelorium!')
  })

  test('can format text as H2 heading', async ({ page }) => {
    await page.getByRole('button', { name: 'Create your first page' }).click()
    const editor = page.locator('.editor-content')
    await editor.click()
    await page.locator('.toolbar-btn:has-text("H2")').click()
    await page.keyboard.type('Background')
    await expect(editor.locator('h2')).toContainText('Background')
  })

  test('content persists across page switches', async ({ page }) => {
    // Create first page with content
    await page.getByRole('button', { name: 'Create your first page' }).click()
    const editor = page.locator('.editor-content')
    await editor.click()
    await page.keyboard.type('First page content')

    // Create second page
    await page.locator('button[title="New page"]').click()
    // Wait for new page to load
    await expect(page.locator('.tree-row')).toHaveCount(2)
    await editor.click()
    await page.keyboard.type('Second page content')

    // Switch back to first page (first tree item)
    await page.locator('.tree-row').first().click()
    await expect(editor).toContainText('First page content')
  })

  test('opens search overlay with Cmd+K', async ({ page }) => {
    await page.getByRole('button', { name: 'Create your first page' }).click()
    await page.keyboard.press('Meta+k')
    await expect(page.locator('.search-modal')).toBeVisible()
    await expect(page.locator('.search-input')).toBeFocused()
  })

  test('search finds pages by title', async ({ page }) => {
    await page.getByRole('button', { name: 'Create your first page' }).click()
    await page.keyboard.press('Meta+k')
    await page.locator('.search-input').fill('Untitled')
    await expect(page.locator('.result-item').first()).toContainText('Untitled Page')
  })

  test('closes search with Escape', async ({ page }) => {
    await page.getByRole('button', { name: 'Create your first page' }).click()
    await page.keyboard.press('Meta+k')
    await expect(page.locator('.search-modal')).toBeVisible()
    // Focus the search input before pressing Escape
    await page.locator('.search-input').focus()
    await page.keyboard.press('Escape')
    await expect(page.locator('.search-modal')).toBeHidden({ timeout: 5000 })
  })

  test('toggles details panel', async ({ page }) => {
    await page.getByRole('button', { name: 'Create your first page' }).click()
    await page.locator('.details-toggle').click()
    await expect(page.locator('.details-panel')).toBeVisible()
    await expect(page.locator('.details-panel')).toContainText('PAGE INFO')
    await expect(page.locator('.details-panel')).toContainText('BACKLINKS')
  })

  test('shows toast notification on page creation', async ({ page }) => {
    await page.getByRole('button', { name: 'Create your first page' }).click()
    await expect(page.locator('.toast')).toBeVisible()
    await expect(page.locator('.toast')).toContainText('Page created')
  })

  test('toolbar has formatting buttons', async ({ page }) => {
    await page.getByRole('button', { name: 'Create your first page' }).click()
    const toolbar = page.locator('.editor-toolbar')
    await expect(toolbar).toBeVisible()
    // Check toolbar exists with expected button count
    const buttons = toolbar.locator('.toolbar-btn')
    await expect(buttons).toHaveCount(10) // B, I, U, H1, H2, H3, List, Link, Img, Table
  })

  test('sidebar shows navigation items', async ({ page }) => {
    await expect(page.getByRole('button', { name: 'Wiki' })).toBeVisible()
    await expect(page.getByRole('button', { name: 'Atlas' })).toBeVisible()
    await expect(page.getByRole('button', { name: 'Chronicle' })).toBeVisible()
    await expect(page.getByRole('button', { name: 'Boards' })).toBeVisible()
    await expect(page.getByRole('button', { name: 'Relations' })).toBeVisible()
  })
})
