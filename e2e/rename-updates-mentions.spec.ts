import { test, expect } from '@playwright/test'
import { createPageViaModal, createAnotherPage } from './helpers'

test.describe('Rename Updates Mentions', () => {
  test('renaming a page updates its @mention link text in other pages', async ({ page }) => {
    await page.goto('/')

    // Create Page A
    await createPageViaModal(page, 'Untitled Page')

    // Create Page B
    await createAnotherPage(page, 'Untitled Page')
    await expect(page.locator('.tree-row')).toHaveCount(2)

    // In Page B, type @ to open mention menu and click first item (Page A)
    const editor = page.locator('.editor-content')
    await editor.click()
    await page.keyboard.type('@')
    const menu = page.locator('[data-testid="mention-menu"]')
    await expect(menu).toBeVisible()
    await menu.locator('.mention-item').first().click()
    await expect(editor.locator('a')).toContainText('Untitled Page')

    // Wait for auto-save
    await page.waitForTimeout(2000)

    // Switch to Page A
    await page.locator('.tree-row').first().click()
    await page.waitForTimeout(500)

    // Rename Page A using keyboard (execCommand approach for proper Svelte binding)
    const titleInput = page.locator('input.page-title')
    await titleInput.focus()
    await titleInput.selectText()
    await page.keyboard.type('Elara Nightwhisper')
    await titleInput.blur()

    // Wait for rename + mention update propagation
    await page.waitForTimeout(3000)

    // Switch to Page B and verify link text updated
    await page.locator('.tree-row').nth(1).click()
    await page.waitForTimeout(1000)

    await expect(editor.locator('a')).toContainText('Elara Nightwhisper')
  })
})
