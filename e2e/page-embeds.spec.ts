import { test, expect } from '@playwright/test'
import { createPageViaModal, createAnotherPage } from './helpers'

test.describe('Page Embeds', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('slash menu shows Page Embed option', async ({ page }) => {
    await createPageViaModal(page, 'Test Page')
    await page.locator('.editor-content').click()
    await page.keyboard.type('/')
    const menu = page.locator('[data-testid="slash-menu"]')
    await expect(menu).toBeVisible()
    await expect(menu).toContainText('Page Embed')
  })

  test('selecting Page Embed opens the embed picker', async ({ page }) => {
    await createPageViaModal(page, 'Host Page')
    await createAnotherPage(page, 'Target Page')

    // Go back to Host Page
    await page.locator('.tree-row:has-text("Host Page")').click()
    await expect(page.locator('input.page-title')).toHaveValue('Host Page')

    // Type /embed
    await page.locator('.editor-content').click()
    await page.keyboard.type('/embed')
    const menu = page.locator('[data-testid="slash-menu"]')
    await expect(menu).toBeVisible()

    // Click "Page Embed"
    await menu.locator('.slash-item:has-text("Page Embed")').click()

    // Embed picker should open
    await expect(page.locator('.embed-picker')).toBeVisible()
    await expect(page.locator('.embed-picker')).toContainText('Target Page')
  })

  test('selecting a page from embed picker inserts embed block', async ({ page }) => {
    await createPageViaModal(page, 'Host Page')
    await createAnotherPage(page, 'Embedded Content')

    // Go back to Host Page
    await page.locator('.tree-row:has-text("Host Page")').click()
    await expect(page.locator('input.page-title')).toHaveValue('Host Page')

    // Insert embed via slash command
    await page.locator('.editor-content').click()
    await page.keyboard.type('/embed')
    await page.locator('.slash-item:has-text("Page Embed")').click()
    await page.locator('.embed-page-item:has-text("Embedded Content")').click()

    // Embed block should be visible in editor
    await expect(page.locator('.page-embed-wrapper')).toBeVisible()
    await expect(page.locator('.page-embed-title')).toContainText('Embedded Content')
  })

  test('embed has Open link that navigates to source page', async ({ page }) => {
    await createPageViaModal(page, 'Host')
    await createAnotherPage(page, 'Source')

    await page.locator('.tree-row:has-text("Host")').click()
    await page.locator('.editor-content').click()
    await page.keyboard.type('/embed')
    await page.locator('.slash-item:has-text("Page Embed")').click()
    await page.locator('.embed-page-item:has-text("Source")').click()

    await expect(page.locator('.page-embed-open')).toBeVisible()
  })

  test('embed picker does not show current page', async ({ page }) => {
    await createPageViaModal(page, 'Only Page')

    await page.locator('.editor-content').click()
    await page.keyboard.type('/embed')
    await page.locator('.slash-item:has-text("Page Embed")').click()

    // Should show "No other pages" since this is the only page
    await expect(page.locator('.embed-empty')).toContainText('No other pages')
  })
})
