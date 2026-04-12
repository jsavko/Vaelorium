import { test, expect } from '@playwright/test'

test.describe('Settings', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('settings opens from sidebar gear icon', async ({ page }) => {
    await page.locator('button[aria-label="Settings"]').click()
    await expect(page.locator('[data-testid="settings"]')).toBeVisible()
    await expect(page.locator('.settings-panel')).toContainText('Settings')
  })

  test('settings shows keybinds tab by default', async ({ page }) => {
    await page.locator('button[aria-label="Settings"]').click()
    await expect(page.locator('.settings-panel')).toContainText('Keyboard Shortcuts')
    await expect(page.locator('.keybind-row')).toHaveCount(6)
  })

  test('can switch to appearance tab', async ({ page }) => {
    await page.locator('button[aria-label="Settings"]').click()
    await page.locator('.settings-nav-item:has-text("Appearance")').click()
    await expect(page.locator('.settings-panel')).toContainText('Theme')
    await expect(page.locator('.settings-panel')).toContainText('Font Size')
  })

  test('settings can be closed', async ({ page }) => {
    await page.locator('button[aria-label="Settings"]').click()
    await expect(page.locator('[data-testid="settings"]')).toBeVisible()
    await page.locator('.settings-close').click()
    await expect(page.locator('[data-testid="settings"]')).toBeHidden()
  })

  test('shows reset to defaults button for keybinds', async ({ page }) => {
    await page.locator('button[aria-label="Settings"]').click()
    await expect(page.locator('.reset-btn')).toBeVisible()
    await expect(page.locator('.reset-btn')).toContainText('Reset to defaults')
  })

  test('appearance tab shows dark library theme', async ({ page }) => {
    await page.locator('button[aria-label="Settings"]').click()
    await page.locator('.settings-nav-item:has-text("Appearance")').click()
    await expect(page.locator('.theme-card').first()).toContainText('Dark Library')
    await expect(page.locator('.theme-card.active')).toBeVisible()
  })

  test('account tab shows version and update controls', async ({ page }) => {
    await page.locator('button[aria-label="Settings"]').click()
    await page.locator('.settings-nav-item:has-text("Account")').click()
    await expect(page.locator('.settings-panel')).toContainText('About')
    await expect(page.locator('.settings-panel')).toContainText('Version')
    await expect(page.locator('.settings-panel')).toContainText('Vaelorium')
    // In browser (non-Tauri) build, updates are gated with a notice
    await expect(page.locator('.settings-panel')).toContainText('Updates are only available in the desktop app.')
  })
})
