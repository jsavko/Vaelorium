import { describe, it, expect } from 'vitest'
import { callCommand } from './bridge'

describe('entity types mock backend', () => {
  it('lists 8 built-in entity types', async () => {
    const types: any[] = await callCommand('list_entity_types')
    const builtins = types.filter((t) => t.is_builtin)
    expect(builtins.length).toBe(8)
    expect(builtins.map((t) => t.name)).toEqual([
      'Character', 'Location', 'Quest', 'Organisation',
      'Item', 'Creature', 'Event', 'Journal',
    ])
  })

  it('gets a built-in entity type by id', async () => {
    const type: any = await callCommand('get_entity_type', { id: 'builtin-character' })
    expect(type.name).toBe('Character')
    expect(type.icon).toBe('shield')
    expect(type.color).toBe('#B85C5C')
    expect(type.is_builtin).toBe(true)
  })

  it('creates a custom entity type', async () => {
    const type: any = await callCommand('create_entity_type', {
      name: 'Spell',
      icon: 'wand',
      color: '#AA55CC',
    })
    expect(type).toHaveProperty('id')
    expect(type.name).toBe('Spell')
    expect(type.icon).toBe('wand')
    expect(type.color).toBe('#AA55CC')
    expect(type.is_builtin).toBe(false)
  })

  it('updates a custom entity type', async () => {
    const type: any = await callCommand('create_entity_type', {
      name: 'Vehicle',
      icon: 'car',
      color: '#333333',
    })
    const updated: any = await callCommand('update_entity_type', {
      id: type.id,
      name: 'Vessel',
      color: '#444444',
    })
    expect(updated.name).toBe('Vessel')
    expect(updated.color).toBe('#444444')
  })

  it('prevents deleting built-in types', async () => {
    await expect(
      callCommand('delete_entity_type', { id: 'builtin-character' }),
    ).rejects.toThrow('Cannot delete built-in entity types')
  })

  it('deletes a custom entity type', async () => {
    const type: any = await callCommand('create_entity_type', {
      name: 'Deletable',
      icon: 'trash',
      color: '#FF0000',
    })
    await callCommand('delete_entity_type', { id: type.id })
    await expect(
      callCommand('get_entity_type', { id: type.id }),
    ).rejects.toThrow()
  })

  it('lists built-in fields for Character type', async () => {
    const fields: any[] = await callCommand('list_entity_type_fields', {
      entity_type_id: 'builtin-character',
    })
    expect(fields.length).toBe(7)
    expect(fields.map((f) => f.name)).toEqual([
      'Race', 'Class', 'Alignment', 'Status', 'HP', 'Location', 'Organisation',
    ])
  })

  it('creates a custom field on a type', async () => {
    const type: any = await callCommand('create_entity_type', {
      name: 'Faction',
      icon: 'flag',
      color: '#00AAFF',
    })
    const field: any = await callCommand('create_entity_type_field', {
      entity_type_id: type.id,
      name: 'Motto',
      field_type: 'text',
    })
    expect(field.name).toBe('Motto')
    expect(field.field_type).toBe('text')
    expect(field.entity_type_id).toBe(type.id)

    const fields: any[] = await callCommand('list_entity_type_fields', {
      entity_type_id: type.id,
    })
    expect(fields.length).toBe(1)
  })

  it('sets and gets field values on a page', async () => {
    const page: any = await callCommand('create_page', {
      input: { title: 'Aragorn', entity_type_id: 'builtin-character' },
    })

    // Set race
    const fv: any = await callCommand('set_field_value', {
      page_id: page.id,
      field_id: 'field-char-race',
      value: '"Human"',
    })
    expect(fv.value).toBe('"Human"')

    // Get all field values for page
    const values: any[] = await callCommand('get_page_field_values', {
      page_id: page.id,
    })
    expect(values.length).toBe(1)
    expect(values[0].field_id).toBe('field-char-race')
    expect(values[0].value).toBe('"Human"')
  })

  it('upserts field values (updates existing)', async () => {
    const page: any = await callCommand('create_page', {
      input: { title: 'Legolas', entity_type_id: 'builtin-character' },
    })

    await callCommand('set_field_value', {
      page_id: page.id,
      field_id: 'field-char-race',
      value: '"Elf"',
    })
    const updated: any = await callCommand('set_field_value', {
      page_id: page.id,
      field_id: 'field-char-race',
      value: '"Wood Elf"',
    })
    expect(updated.value).toBe('"Wood Elf"')

    const values: any[] = await callCommand('get_page_field_values', {
      page_id: page.id,
    })
    // Should have exactly 1 value, not 2
    const raceValues = values.filter((v) => v.field_id === 'field-char-race')
    expect(raceValues.length).toBe(1)
    expect(raceValues[0].value).toBe('"Wood Elf"')
  })

  it('deletes a field value', async () => {
    const page: any = await callCommand('create_page', {
      input: { title: 'Gimli', entity_type_id: 'builtin-character' },
    })

    await callCommand('set_field_value', {
      page_id: page.id,
      field_id: 'field-char-race',
      value: '"Dwarf"',
    })
    await callCommand('delete_field_value', {
      page_id: page.id,
      field_id: 'field-char-race',
    })

    const values: any[] = await callCommand('get_page_field_values', {
      page_id: page.id,
    })
    const raceValues = values.filter((v) => v.field_id === 'field-char-race')
    expect(raceValues.length).toBe(0)
  })

  it('queries pages by field value', async () => {
    const p1: any = await callCommand('create_page', {
      input: { title: 'Gandalf', entity_type_id: 'builtin-character' },
    })
    const p2: any = await callCommand('create_page', {
      input: { title: 'Saruman', entity_type_id: 'builtin-character' },
    })
    const p3: any = await callCommand('create_page', {
      input: { title: 'Frodo', entity_type_id: 'builtin-character' },
    })

    await callCommand('set_field_value', {
      page_id: p1.id,
      field_id: 'field-char-class',
      value: '"Wizard"',
    })
    await callCommand('set_field_value', {
      page_id: p2.id,
      field_id: 'field-char-class',
      value: '"Wizard"',
    })
    await callCommand('set_field_value', {
      page_id: p3.id,
      field_id: 'field-char-class',
      value: '"Hobbit"',
    })

    const wizards: any[] = await callCommand('query_pages_by_field', {
      field_id: 'field-char-class',
      value: '"Wizard"',
    })
    expect(wizards.length).toBe(2)
    const names = wizards.map((p) => p.title).sort()
    expect(names).toEqual(['Gandalf', 'Saruman'])
  })

  it('reorders entity type fields', async () => {
    const type: any = await callCommand('create_entity_type', {
      name: 'Reorderable',
      icon: 'list',
      color: '#AABB00',
    })
    const f1: any = await callCommand('create_entity_type_field', {
      entity_type_id: type.id,
      name: 'First',
      field_type: 'text',
    })
    const f2: any = await callCommand('create_entity_type_field', {
      entity_type_id: type.id,
      name: 'Second',
      field_type: 'text',
    })

    // Swap order
    await callCommand('reorder_entity_type_fields', {
      moves: [
        { id: f1.id, sort_order: 2 },
        { id: f2.id, sort_order: 1 },
      ],
    })

    const fields: any[] = await callCommand('list_entity_type_fields', {
      entity_type_id: type.id,
    })
    expect(fields[0].name).toBe('Second')
    expect(fields[1].name).toBe('First')
  })

  it('deleting a type cascades to its fields and field values', async () => {
    const type: any = await callCommand('create_entity_type', {
      name: 'CascadeTest',
      icon: 'x',
      color: '#000',
    })
    const field: any = await callCommand('create_entity_type_field', {
      entity_type_id: type.id,
      name: 'TestField',
      field_type: 'text',
    })
    const page: any = await callCommand('create_page', {
      input: { title: 'CascadePage', entity_type_id: type.id },
    })
    await callCommand('set_field_value', {
      page_id: page.id,
      field_id: field.id,
      value: '"test"',
    })

    // Delete the type
    await callCommand('delete_entity_type', { id: type.id })

    // Fields should be gone
    const fields: any[] = await callCommand('list_entity_type_fields', {
      entity_type_id: type.id,
    })
    expect(fields.length).toBe(0)

    // Page should have entity_type_id cleared
    const updatedPage: any = await callCommand('get_page', { id: page.id })
    expect(updatedPage.entity_type_id).toBeNull()
  })
})
