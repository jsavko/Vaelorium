import { test, expect } from '@playwright/test'
import { createPageViaModal } from './helpers'

test.describe('Entity Structured Fields', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('Character page shows all 7 fields in details panel', async ({ page }) => {
    await createPageViaModal(page, 'Test Character', { entityType: 'Character' })
    await page.getByRole('button', { name: 'Details' }).click()

    await expect(page.locator('text=CHARACTER FIELDS')).toBeVisible()
    await expect(page.locator('label:has-text("Race")')).toBeVisible()
    await expect(page.locator('label:has-text("Class")')).toBeVisible()
    await expect(page.locator('label:has-text("Alignment")')).toBeVisible()
    await expect(page.locator('label:has-text("Status")')).toBeVisible()
    await expect(page.locator('label:has-text("HP")')).toBeVisible()
    await expect(page.locator('label:has-text("Location")')).toBeVisible()
    await expect(page.locator('label:has-text("Organisation")')).toBeVisible()
  })

  test('editing a text field saves the value', async ({ page }) => {
    await createPageViaModal(page, 'Editable Character', { entityType: 'Character' })
    await page.getByRole('button', { name: 'Details' }).click()

    const raceInput = page.locator('#field-char-race')
    await raceInput.fill('Half-Elf')
    // Wait for debounced save
    await page.waitForTimeout(500)

    // Value should persist — close and reopen details
    await page.getByRole('button', { name: 'Details' }).click()
    await page.getByRole('button', { name: 'Details' }).click()
    await expect(page.locator('#field-char-race')).toHaveValue('Half-Elf')
  })

  test('editing a select field saves immediately', async ({ page }) => {
    await createPageViaModal(page, 'Status Character', { entityType: 'Character' })
    await page.getByRole('button', { name: 'Details' }).click()

    const statusSelect = page.locator('#field-char-status')
    await statusSelect.selectOption('Dead')

    // Close and reopen
    await page.getByRole('button', { name: 'Details' }).click()
    await page.getByRole('button', { name: 'Details' }).click()
    await expect(page.locator('#field-char-status')).toHaveValue('Dead')
  })

  test('editing a number field saves the value', async ({ page }) => {
    await createPageViaModal(page, 'HP Character', { entityType: 'Character' })
    await page.getByRole('button', { name: 'Details' }).click()

    const hpInput = page.locator('#field-char-hp')
    await hpInput.fill('42')
    await page.waitForTimeout(500)

    await page.getByRole('button', { name: 'Details' }).click()
    await page.getByRole('button', { name: 'Details' }).click()
    await expect(page.locator('#field-char-hp')).toHaveValue('42')
  })

  test('blank page shows no entity fields', async ({ page }) => {
    await createPageViaModal(page, 'Blank Page')
    await page.getByRole('button', { name: 'Details' }).click()

    await expect(page.locator('text=FIELDS')).not.toBeVisible()
    await expect(page.locator('text=PAGE INFO')).toBeVisible()
  })

  test('changing page type updates the fields', async ({ page }) => {
    await createPageViaModal(page, 'Retype Page', { entityType: 'Character' })
    await page.getByRole('button', { name: 'Details' }).click()
    await expect(page.locator('text=CHARACTER FIELDS')).toBeVisible()

    // Change to Location
    await page.locator('.type-select').selectOption({ label: 'Location' })
    await expect(page.locator('text=LOCATION FIELDS')).toBeVisible()
    await expect(page.locator('label:has-text("Climate")')).toBeVisible()
    // Character fields should be gone
    await expect(page.locator('label:has-text("Race")')).not.toBeVisible()
  })

  test('Location page shows correct fields', async ({ page }) => {
    await createPageViaModal(page, 'Test City', { entityType: 'Location' })
    await page.getByRole('button', { name: 'Details' }).click()

    await expect(page.locator('text=LOCATION FIELDS')).toBeVisible()
    await expect(page.locator('label:has-text("Type")')).toBeVisible()
    await expect(page.locator('label:has-text("Region")')).toBeVisible()
    await expect(page.locator('label:has-text("Population")')).toBeVisible()
    await expect(page.locator('label:has-text("Climate")')).toBeVisible()
  })
})
