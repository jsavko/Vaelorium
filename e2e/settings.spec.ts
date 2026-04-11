import { test, expect } from '@playwright/test'

test.describe('Settings', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('settings opens from sidebar gear icon', async ({ page }) => {
    await page.locator('.settings-btn').click()
    await expect(page.locator('[data-testid="settings"]')).toBeVisible()
    await expect(page.locator('.settings-panel')).toContainText('Settings')
  })

  test('settings shows keybinds tab by default', async ({ page }) => {
    await page.locator('.settings-btn').click()
    await expect(page.locator('.settings-panel')).toContainText('Keyboard Shortcuts')
    await expect(page.locator('.keybind-row')).toHaveCount(6)
  })

  test('can switch to appearance tab', async ({ page }) => {
    await page.locator('.settings-btn').click()
    await page.locator('.settings-nav-item:has-text("Appearance")').click()
    await expect(page.locator('.settings-panel')).toContainText('Theme')
    await expect(page.locator('.settings-panel')).toContainText('Font Size')
  })

  test('settings can be closed', async ({ page }) => {
    await page.locator('.settings-btn').click()
    await expect(page.locator('[data-testid="settings"]')).toBeVisible()
    await page.locator('.settings-close').click()
    await expect(page.locator('[data-testid="settings"]')).toBeHidden()
  })

  test('shows reset to defaults button for keybinds', async ({ page }) => {
    await page.locator('.settings-btn').click()
    await expect(page.locator('.reset-btn')).toBeVisible()
    await expect(page.locator('.reset-btn')).toContainText('Reset to defaults')
  })

  test('appearance tab shows dark library theme', async ({ page }) => {
    await page.locator('.settings-btn').click()
    await page.locator('.settings-nav-item:has-text("Appearance")').click()
    await expect(page.locator('.theme-card')).toContainText('Dark Library')
    await expect(page.locator('.theme-card.active')).toBeVisible()
  })
})
