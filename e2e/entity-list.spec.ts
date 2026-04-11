import { test, expect } from '@playwright/test'
import { createPageViaModal } from './helpers'

test.describe('Entity List View', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('sidebar shows entity type navigation', async ({ page }) => {
    await expect(page.locator('text=TYPES')).toBeVisible()
    await expect(page.getByRole('button', { name: 'Characters' })).toBeVisible()
    await expect(page.getByRole('button', { name: 'Locations' })).toBeVisible()
  })

  test('clicking a type shows entity list view', async ({ page }) => {
    await page.getByRole('button', { name: 'Characters' }).click()
    await expect(page.locator('text=No characters yet')).toBeVisible()
    await expect(page.locator('button:has-text("Create your first character")')).toBeVisible()
  })

  test('entity list shows pages of that type', async ({ page }) => {
    await createPageViaModal(page, 'Aragorn', { entityType: 'Character' })
    await page.getByRole('button', { name: 'Characters' }).click()
    await expect(page.locator('.card-title:has-text("Aragorn")')).toBeVisible()
  })

  test('clicking a card opens the page', async ({ page }) => {
    await createPageViaModal(page, 'Gandalf', { entityType: 'Character' })
    await page.getByRole('button', { name: 'Characters' }).click()
    await page.locator('.card:has-text("Gandalf")').click()
    await expect(page.locator('input.page-title')).toHaveValue('Gandalf')
  })

  test('search filters the list', async ({ page }) => {
    await createPageViaModal(page, 'Frodo', { entityType: 'Character' })
    await createPageViaModal(page, 'Samwise', { entityType: 'Character', via: 'plus-button' })
    await page.getByRole('button', { name: 'Characters' }).click()

    const searchInput = page.locator('.search-input')
    await searchInput.fill('Fro')
    await expect(page.locator('.card-title:has-text("Frodo")')).toBeVisible()
    await expect(page.locator('.card-title:has-text("Samwise")')).not.toBeVisible()
  })

  test('back button returns to editor', async ({ page }) => {
    await page.getByRole('button', { name: 'Characters' }).click()
    await page.locator('button[aria-label="Back to editor"]').click()
    // Should be back to welcome screen
    await expect(page.locator('text=Welcome to Vaelorium')).toBeVisible()
  })
})

test.describe('Custom Type Builder', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('custom type card appears in new page modal', async ({ page }) => {
    await page.getByRole('button', { name: 'Create your first page' }).click()
    await expect(page.getByRole('dialog').getByText('Custom Type')).toBeVisible()
  })

  test('creates a custom type with fields', async ({ page }) => {
    await page.getByRole('button', { name: 'Create your first page' }).click()
    const dialog = page.getByRole('dialog')

    // Click Custom Type
    await dialog.getByText('Custom Type').click()

    // Fill in the builder
    const builder = page.locator('.builder')
    await builder.locator('.form-input').fill('Spell')

    // Add a field
    await builder.getByText('+ Add field').click()
    await builder.locator('.field-name-input').fill('School')
    await builder.locator('.field-type-select').selectOption('select')
    await builder.locator('.field-options-input').fill('Abjuration, Conjuration, Divination, Enchantment')

    // Save
    await builder.getByText('Create Type').click()

    // Builder should close, custom type should now appear in modal
    await expect(dialog.getByText('Spell')).toBeVisible()
  })
})
