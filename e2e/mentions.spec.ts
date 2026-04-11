import { test, expect } from '@playwright/test'
import { createPageViaModal, createAnotherPage } from './helpers'

test.describe('@Mention Linking', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    // Create two pages so we have something to mention
    await createPageViaModal(page, 'Untitled Page')
    await createAnotherPage(page, 'Untitled Page')
    await expect(page.locator('.tree-row')).toHaveCount(2)
  })

  test('typing @ opens the mention dropdown', async ({ page }) => {
    const editor = page.locator('.editor-content')
    await editor.click()
    await page.keyboard.type('@')
    await expect(page.locator('[data-testid="mention-menu"]')).toBeVisible()
    await expect(page.locator('[data-testid="mention-menu"]')).toContainText('LINK TO PAGE')
  })

  test('mention dropdown shows available pages', async ({ page }) => {
    const editor = page.locator('.editor-content')
    await editor.click()
    await page.keyboard.type('@')
    const menu = page.locator('[data-testid="mention-menu"]')
    await expect(menu).toBeVisible()
    await expect(menu.locator('.mention-item')).toHaveCount(2)
  })

  test('clicking a mention inserts a link', async ({ page }) => {
    const editor = page.locator('.editor-content')
    await editor.click()
    await page.keyboard.type('See also @')
    const menu = page.locator('[data-testid="mention-menu"]')
    await expect(menu).toBeVisible()
    await menu.locator('.mention-item').first().click()
    await expect(editor).toContainText('See also Untitled Page')
    // The mention text should be rendered as a link
    await expect(editor.locator('a')).toContainText('Untitled Page')
  })

  test('Escape closes the mention dropdown', async ({ page }) => {
    const editor = page.locator('.editor-content')
    await editor.click()
    await page.keyboard.type('@')
    await expect(page.locator('[data-testid="mention-menu"]')).toBeVisible()
    await page.keyboard.press('Escape')
    await expect(page.locator('[data-testid="mention-menu"]')).toBeHidden()
  })
})
