import { test, expect } from '@playwright/test'
import { createPageViaModal, createAnotherPage } from './helpers'

test.describe('[[Wiki Link]] Syntax', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await createPageViaModal(page, 'Untitled Page')
    await createAnotherPage(page, 'Untitled Page')
    await expect(page.locator('.tree-row')).toHaveCount(2)
  })

  test('typing [[ opens the mention dropdown', async ({ page }) => {
    const editor = page.locator('.editor-content')
    await editor.click()
    await page.keyboard.type('[[')
    await expect(page.locator('[data-testid="mention-menu"]')).toBeVisible()
    await expect(page.locator('[data-testid="mention-menu"]')).toContainText('LINK TO')
  })

  test('selecting from [[ dropdown inserts a link', async ({ page }) => {
    const editor = page.locator('.editor-content')
    await editor.click()
    await page.keyboard.type('See [[')
    const menu = page.locator('[data-testid="mention-menu"]')
    await expect(menu).toBeVisible()
    await menu.locator('.mention-item').first().click()
    await expect(editor.locator('a')).toContainText('Untitled Page')
  })

  test('Escape closes the [[ dropdown', async ({ page }) => {
    const editor = page.locator('.editor-content')
    await editor.click()
    await page.keyboard.type('[[')
    await expect(page.locator('[data-testid="mention-menu"]')).toBeVisible()
    await page.keyboard.press('Escape')
    await expect(page.locator('[data-testid="mention-menu"]')).toBeHidden()
  })
})
