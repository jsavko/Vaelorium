import { test, expect } from '@playwright/test'
import { createPageViaModal, createAnotherPage } from './helpers'

test.describe('Relations', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('relations panel shows in details', async ({ page }) => {
    await createPageViaModal(page, 'Test Page')
    await page.getByRole('button', { name: 'Details' }).click()
    await expect(page.getByRole('heading', { name: 'RELATIONS' })).toBeVisible()
    await expect(page.locator('text=No relations yet')).toBeVisible()
  })

  test('can add a relation between two pages', async ({ page }) => {
    await createPageViaModal(page, 'Page A')
    await createAnotherPage(page, 'Page B')

    // Go to Page A
    await page.locator('.tree-row:has-text("Page A")').click()
    await page.getByRole('button', { name: 'Details' }).click()

    // Add relation
    await page.locator('.section-header:has-text("RELATIONS") .add-btn').click()
    await page.locator('.type-select').last().selectOption('rel-ally-of')
    await page.locator('.search-input').last().fill('Page B')
    await page.locator('.search-result:has-text("Page B")').click()

    // Should show the relation
    await expect(page.locator('.rel-link:has-text("Page B")')).toBeVisible()
  })

  test('relation appears on both pages', async ({ page }) => {
    await createPageViaModal(page, 'Alpha')
    await createAnotherPage(page, 'Beta')

    // Add relation on Alpha
    await page.locator('.tree-row:has-text("Alpha")').click()
    await page.getByRole('button', { name: 'Details' }).click()
    await page.locator('.section-header:has-text("RELATIONS") .add-btn').click()
    await page.locator('.type-select').last().selectOption('rel-enemy-of')
    await page.locator('.search-input').last().fill('Beta')
    await page.locator('.search-result:has-text("Beta")').click()

    // Check Beta's relations
    await page.locator('.tree-row:has-text("Beta")').click()
    await expect(page.locator('.rel-link:has-text("Alpha")')).toBeVisible()
  })

  test('can delete a relation', async ({ page }) => {
    await createPageViaModal(page, 'Del A')
    await createAnotherPage(page, 'Del B')

    await page.locator('.tree-row:has-text("Del A")').click()
    await page.getByRole('button', { name: 'Details' }).click()
    await page.locator('.section-header:has-text("RELATIONS") .add-btn').click()
    await page.locator('.type-select').last().selectOption('rel-ally-of')
    await page.locator('.search-input').last().fill('Del B')
    await page.locator('.search-result:has-text("Del B")').click()

    // Delete it
    await page.locator('.relation-row').hover()
    await page.locator('.rel-remove').click()
    await expect(page.locator('text=No relations yet')).toBeVisible()
  })

  test('graph view opens from Relations nav', async ({ page }) => {
    await page.getByRole('button', { name: 'Relations' }).click()
    await expect(page.locator('text=Relations Graph')).toBeVisible()
    await expect(page.locator('canvas')).toBeVisible()
  })
})
