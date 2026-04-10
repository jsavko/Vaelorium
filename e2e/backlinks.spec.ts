import { test, expect } from '@playwright/test'

test.describe('Backlinks', () => {
  test('shows backlinks when a page is mentioned from another', async ({ page }) => {
    await page.goto('/')

    // Create page 1
    await page.getByRole('button', { name: 'Create your first page' }).click()
    await expect(page.locator('input.page-title')).toBeVisible()

    // Create page 2
    await page.locator('button[title="New page"]').click()
    await expect(page.locator('.tree-row')).toHaveCount(2)

    // Type @mention in page 2 linking to page 1
    const editor = page.locator('.editor-content')
    await editor.click()
    await page.keyboard.type('@')
    const menu = page.locator('[data-testid="mention-menu"]')
    await expect(menu).toBeVisible()
    await menu.locator('.mention-item').first().click()
    await expect(editor.locator('a')).toBeVisible()

    // Wait for auto-save (1s debounce + buffer)
    await page.waitForTimeout(2000)

    // Switch to page 1
    await page.locator('.tree-row').first().click()
    await page.waitForTimeout(500)

    // Open details panel
    await page.locator('.details-toggle').click()
    await expect(page.locator('.details-panel')).toBeVisible()

    // Backlinks should show page 2
    const backlinks = page.locator('.backlinks-section')
    await expect(backlinks.locator('.backlink-item')).toHaveCount(1, { timeout: 5000 })
    await expect(backlinks.locator('.backlink-item')).toContainText('Untitled Page')
  })
})
