import { test, expect } from '@playwright/test'

test.describe('Version History', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await page.getByRole('button', { name: 'Create your first page' }).click()
    await expect(page.locator('input.page-title')).toBeVisible()
  })

  test('version history is accessible from more menu', async ({ page }) => {
    await page.locator('.more-btn').click()
    await expect(page.locator('.more-dropdown')).toBeVisible()
    await expect(page.locator('.more-dropdown')).toContainText('Version History')
  })

  test('clicking Version History opens the panel', async ({ page }) => {
    await page.locator('.more-btn').click()
    await page.locator('.more-item:has-text("Version History")').click()
    await expect(page.locator('.version-panel')).toBeVisible()
    await expect(page.locator('.version-panel')).toContainText('Version History')
  })

  test('version history panel can be closed', async ({ page }) => {
    await page.locator('.more-btn').click()
    await page.locator('.more-item:has-text("Version History")').click()
    await expect(page.locator('.version-panel')).toBeVisible()

    await page.locator('.version-panel .close-btn').click()
    await expect(page.locator('.version-panel')).toBeHidden()
  })
})
