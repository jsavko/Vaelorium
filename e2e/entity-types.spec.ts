import { test, expect } from '@playwright/test'
import { createPageViaModal, createAnotherPage } from './helpers'

test.describe('Entity Type System', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('new page modal shows all 8 entity types', async ({ page }) => {
    await page.getByRole('button', { name: 'Create your first page' }).click()
    const dialog = page.getByRole('dialog')
    await dialog.waitFor()

    await expect(dialog.getByRole('button', { name: 'Character' })).toBeVisible()
    await expect(dialog.getByRole('button', { name: 'Location' })).toBeVisible()
    await expect(dialog.getByRole('button', { name: 'Quest' })).toBeVisible()
    await expect(dialog.getByRole('button', { name: 'Organisation' })).toBeVisible()
    await expect(dialog.getByRole('button', { name: 'Item' })).toBeVisible()
    await expect(dialog.getByRole('button', { name: 'Creature' })).toBeVisible()
    await expect(dialog.getByRole('button', { name: 'Event' })).toBeVisible()
    await expect(dialog.getByRole('button', { name: 'Journal' })).toBeVisible()
  })

  test('creates a blank page (no type)', async ({ page }) => {
    await createPageViaModal(page, 'My Blank Page')
    await expect(page.locator('input.page-title')).toHaveValue('My Blank Page')
    // No entity badge should appear
    await expect(page.locator('.entity-badge')).not.toBeVisible()
  })

  test('creates a typed page (Character)', async ({ page }) => {
    await createPageViaModal(page, 'Elara Nightwhisper', { entityType: 'Character' })
    await expect(page.locator('input.page-title')).toHaveValue('Elara Nightwhisper')
    // Entity badge should show "Character"
    await expect(page.locator('.entity-badge')).toContainText('Character')
  })

  test('typed page shows colored dot in sidebar', async ({ page }) => {
    await createPageViaModal(page, 'Moonwell Sanctum', { entityType: 'Location' })
    const dot = page.locator('.tree-row.active .entity-dot')
    await expect(dot).toBeVisible()
    // Location color is #4A8C6A
    const bg = await dot.evaluate((el) => getComputedStyle(el).backgroundColor)
    expect(bg).not.toBe('') // has a color set
  })

  test('change page type via details panel', async ({ page }) => {
    await createPageViaModal(page, 'Retyped Page')
    // No badge initially
    await expect(page.locator('.entity-badge')).not.toBeVisible()

    // Open details panel
    await page.getByRole('button', { name: 'Details' }).click()

    // Change type to Quest
    await page.locator('.type-select').selectOption({ label: 'Quest' })

    // Badge should appear
    await expect(page.locator('.entity-badge')).toContainText('Quest')
  })

  test('remove page type via details panel', async ({ page }) => {
    await createPageViaModal(page, 'Typed Then Untyped', { entityType: 'Item' })
    await expect(page.locator('.entity-badge')).toContainText('Item')

    // Open details and remove type
    await page.getByRole('button', { name: 'Details' }).click()
    await page.locator('.type-select').selectOption({ label: 'None (blank page)' })

    // Badge should disappear
    await expect(page.locator('.entity-badge')).not.toBeVisible()
  })

  test('modal closes on Escape', async ({ page }) => {
    await page.getByRole('button', { name: 'Create your first page' }).click()
    await page.getByRole('dialog').waitFor()
    await page.keyboard.press('Escape')
    await expect(page.getByRole('dialog')).not.toBeVisible()
  })

  test('create button is disabled without title', async ({ page }) => {
    await page.getByRole('button', { name: 'Create your first page' }).click()
    const dialog = page.getByRole('dialog')
    await dialog.waitFor()
    const createBtn = dialog.getByRole('button', { name: 'Create' })
    await expect(createBtn).toBeDisabled()
  })
})
