import { test, expect } from '@playwright/test'

test.describe('Tags', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await page.getByRole('button', { name: 'Create your first page' }).click()
    await expect(page.locator('input.page-title')).toBeVisible()
    // Open details panel
    await page.locator('.details-toggle').click()
    await expect(page.locator('.details-panel')).toBeVisible()
  })

  test('tag input is visible in details panel', async ({ page }) => {
    await expect(page.locator('[data-testid="tag-input"]')).toBeVisible()
    await expect(page.locator('[data-testid="tag-input"]')).toContainText('TAGS')
  })

  test('can create and add a new tag', async ({ page }) => {
    const input = page.locator('.tag-input')
    await input.fill('NPC')
    await input.press('Enter')
    await expect(page.locator('.tag-pill')).toContainText('NPC')
  })

  test('can remove a tag', async ({ page }) => {
    const input = page.locator('.tag-input')
    await input.fill('NPC')
    await input.press('Enter')
    await expect(page.locator('.tag-pill')).toContainText('NPC')

    await page.locator('.tag-remove').click()
    await expect(page.locator('.tag-pill')).toHaveCount(0)
  })

  test('can add multiple tags', async ({ page }) => {
    const input = page.locator('.tag-input')
    await input.fill('NPC')
    await input.press('Enter')
    await input.fill('Silvergrove')
    await input.press('Enter')
    await expect(page.locator('.tag-pill')).toHaveCount(2)
  })
})
