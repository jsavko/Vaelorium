import { test, expect } from '@playwright/test'

test.describe('Read Mode', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await page.getByRole('button', { name: 'Create your first page' }).click()
    await expect(page.locator('input.page-title')).toBeVisible()
  })

  test('clicking Read toggles to reading view', async ({ page }) => {
    // Type some content first
    const editor = page.locator('.editor-content')
    await editor.click()
    await page.keyboard.type('Hello world')

    // Click Read button
    await page.locator('.mode-toggle').click()
    await expect(page.locator('.mode-toggle')).toContainText('Edit')

    // Reading view should show the title
    await expect(page.locator('.reading-wrapper .page-title, .reading-wrapper h1')).toBeVisible()
  })

  test('clicking Edit returns to editing view', async ({ page }) => {
    await page.locator('.mode-toggle').click()
    await expect(page.locator('.mode-toggle')).toContainText('Edit')

    await page.locator('.mode-toggle').click()
    await expect(page.locator('.mode-toggle')).toContainText('Read')
    await expect(page.locator('.editor-content')).toBeVisible()
  })
})
