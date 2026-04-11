import { test, expect } from '@playwright/test'
import { createPageViaModal } from './helpers'

test.describe('Keybinds Actually Work', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    // Clear localStorage to reset settings
    await page.evaluate(() => localStorage.removeItem('vaelorium-settings'))
    await page.reload()
  })

  test('default Ctrl+K opens search', async ({ page }) => {
    await createPageViaModal(page, 'Untitled Page', { via: 'keyboard' })
    await page.keyboard.press('Meta+k')
    await expect(page.locator('.search-modal')).toBeVisible()
  })

  test('changing search keybind to Ctrl+J makes Ctrl+J open search', async ({ page }) => {
    await createPageViaModal(page, 'Untitled Page')

    // Open settings
    await page.locator('button[aria-label="Settings"]').click()
    await expect(page.locator('[data-testid="settings"]')).toBeVisible()

    // Click on the Search keybind value to edit it
    const searchRow = page.locator('.keybind-row').filter({ hasText: 'Search' })
    await searchRow.locator('.keybind-value').click()

    // Press Ctrl+J to set new keybind
    await page.keyboard.press('Control+j')

    // Close settings
    await page.locator('.settings-close').click()

    // Verify Ctrl+K no longer opens search
    await page.keyboard.press('Meta+k')
    await expect(page.locator('.search-modal')).not.toBeVisible()

    // Verify Ctrl+J opens search
    await page.keyboard.press('Control+j')
    await expect(page.locator('.search-modal')).toBeVisible()
  })

  test('reset to defaults restores original keybinds', async ({ page }) => {
    await createPageViaModal(page, 'Untitled Page')

    // Open settings and change search keybind
    await page.locator('button[aria-label="Settings"]').click()
    const searchRow = page.locator('.keybind-row').filter({ hasText: 'Search' })
    await searchRow.locator('.keybind-value').click()
    await page.keyboard.press('Control+j')

    // Reset to defaults
    await page.locator('.reset-btn').click()

    // Verify keybind value shows Ctrl+K again
    await expect(searchRow.locator('.keybind-value')).toContainText('Ctrl+K')

    // Close settings and verify Ctrl+K works
    await page.locator('.settings-close').click()
    await page.keyboard.press('Meta+k')
    await expect(page.locator('.search-modal')).toBeVisible()
  })
})
