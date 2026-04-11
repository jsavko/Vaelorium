import { test, expect } from '@playwright/test'
import { createPageViaModal } from './helpers'

test.describe('Page Icon Picker', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await createPageViaModal(page, 'Untitled Page')
  })

  test('icon picker trigger is visible next to title', async ({ page }) => {
    await expect(page.locator('.icon-trigger')).toBeVisible()
  })

  test('clicking icon trigger opens emoji grid', async ({ page }) => {
    await page.locator('.icon-trigger').click()
    await expect(page.locator('[data-testid="icon-picker"]')).toBeVisible()
    await expect(page.locator('.icon-option')).toHaveCount(48)
  })

  test('selecting an emoji sets the page icon', async ({ page }) => {
    await page.locator('.icon-trigger').click()
    await page.locator('.icon-option').first().click()
    // Icon should now show in the trigger button
    await expect(page.locator('.current-icon')).toBeVisible()
  })

  test('remove button clears the icon', async ({ page }) => {
    // Set an icon first
    await page.locator('.icon-trigger').click()
    await page.locator('.icon-option').first().click()
    await expect(page.locator('.current-icon')).toBeVisible()

    // Remove it
    await page.locator('.icon-trigger').click()
    await page.locator('.remove-btn').click()
    await expect(page.locator('.add-icon')).toBeVisible()
  })
})
