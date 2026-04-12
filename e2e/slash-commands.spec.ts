import { test, expect } from '@playwright/test'
import { createPageViaModal } from './helpers'

test.describe('Slash Commands', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await createPageViaModal(page, 'Untitled Page')
    await expect(page.locator('.editor-content')).toBeVisible()
  })

  test('typing / opens the slash menu', async ({ page }) => {
    await page.locator('.editor-content').click()
    await page.keyboard.type('/')
    await expect(page.locator('[data-testid="slash-menu"]')).toBeVisible()
  })

  test('slash menu shows command options', async ({ page }) => {
    await page.locator('.editor-content').click()
    await page.keyboard.type('/')
    const menu = page.locator('[data-testid="slash-menu"]')
    await expect(menu).toBeVisible()
    await expect(menu.locator('.slash-item')).toHaveCount(13)
    await expect(menu).toContainText('Heading 1')
    await expect(menu).toContainText('Bullet List')
    await expect(menu).toContainText('Table')
  })

  test('slash menu filters as you type', async ({ page }) => {
    await page.locator('.editor-content').click()
    await page.keyboard.type('/head')
    const menu = page.locator('[data-testid="slash-menu"]')
    await expect(menu).toBeVisible()
    // Should only show heading options
    const items = menu.locator('.slash-item')
    await expect(items).toHaveCount(3) // Heading 1, 2, 3
  })

  test('clicking a command applies formatting', async ({ page }) => {
    await page.locator('.editor-content').click()
    await page.keyboard.type('/')
    const menu = page.locator('[data-testid="slash-menu"]')
    await expect(menu).toBeVisible()
    // Click Heading 2
    await menu.locator('.slash-item:has-text("Heading 2")').click()
    await page.keyboard.type('My Section Title')
    await expect(page.locator('.editor-content h2')).toContainText('My Section Title')
  })

  test('Escape closes the slash menu', async ({ page }) => {
    await page.locator('.editor-content').click()
    await page.keyboard.type('/')
    await expect(page.locator('[data-testid="slash-menu"]')).toBeVisible()
    await page.keyboard.press('Escape')
    await expect(page.locator('[data-testid="slash-menu"]')).toBeHidden()
  })
})
