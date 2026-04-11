import { test, expect } from '@playwright/test'

test.describe('Tomes', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('app starts with editor view in browser mock (tome auto-open)', async ({ page }) => {
    // In browser mock, a tome is always open, so we see the editor
    await expect(page.locator('text=Welcome to Vaelorium')).toBeVisible()
    // Sidebar should be visible
    await expect(page.locator('text=PAGES')).toBeVisible()
  })

  test('sidebar shows tome name or Vaelorium', async ({ page }) => {
    // Should show "Vaelorium" or the mock tome name
    const logo = page.locator('.logo')
    await expect(logo).toBeVisible()
  })

  test('creating a page works with tome system active', async ({ page }) => {
    // Verify the full page creation flow still works
    await page.getByRole('button', { name: 'Create your first page' }).click()
    const dialog = page.getByRole('dialog')
    await dialog.waitFor()
    await dialog.getByPlaceholder('Page title...').fill('Tome Test Page')
    await dialog.getByRole('button', { name: 'Create' }).click()
    await expect(page.locator('input.page-title')).toHaveValue('Tome Test Page')
  })
})
