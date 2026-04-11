import type { Page } from '@playwright/test'

/**
 * Creates a page via the New Page modal.
 * Handles both the "Create your first page" button and the "+" button,
 * which both open the modal.
 */
export async function createPageViaModal(
  page: Page,
  title: string = 'Untitled Page',
  options?: {
    entityType?: string // e.g., "Character", "Location"
    parent?: string // parent page title to select
    via?: 'first-page' | 'plus-button' | 'keyboard'
  },
) {
  const via = options?.via || 'first-page'

  if (via === 'first-page') {
    await page.getByRole('button', { name: 'Create your first page' }).click()
  } else if (via === 'plus-button') {
    await page.locator('button[title="New page"]').click()
  } else if (via === 'keyboard') {
    await page.keyboard.press('Control+n')
  }

  // Wait for modal
  await page.getByRole('dialog').waitFor()

  // Select entity type if specified
  if (options?.entityType) {
    await page.getByRole('dialog').getByRole('button', { name: options.entityType }).click()
  }

  // Type title
  const titleInput = page.getByRole('dialog').getByPlaceholder('Page title...')
  await titleInput.fill(title)

  // Select parent if specified
  if (options?.parent) {
    await page.getByRole('dialog').locator('select').selectOption({ label: options.parent })
  }

  // Click Create
  await page.getByRole('dialog').getByRole('button', { name: 'Create' }).click()

  // Wait for page to load
  await page.locator('input.page-title').waitFor({ timeout: 10000 })
}

/**
 * Creates a second page (when pages already exist, use + button).
 */
export async function createAnotherPage(
  page: Page,
  title: string = 'Untitled Page',
  options?: { entityType?: string; parent?: string },
) {
  return createPageViaModal(page, title, { ...options, via: 'plus-button' })
}
