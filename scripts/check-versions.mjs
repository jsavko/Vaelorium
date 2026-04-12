#!/usr/bin/env node
// Verifies package.json, tauri.conf.json, and Cargo.toml all agree on version.
// If GITHUB_REF_NAME is set (tag push in CI), also verifies the tag matches.
// Exit 0 on match, 1 on mismatch.
import { readFileSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')

const pkg = JSON.parse(readFileSync(resolve(root, 'package.json'), 'utf8')).version
const tauri = JSON.parse(readFileSync(resolve(root, 'src-tauri/tauri.conf.json'), 'utf8')).version
const cargoText = readFileSync(resolve(root, 'src-tauri/Cargo.toml'), 'utf8')
const cargo = cargoText.match(/^\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m)?.[1]

const versions = { 'package.json': pkg, 'tauri.conf.json': tauri, 'Cargo.toml': cargo }
console.log('Manifest versions:')
for (const [k, v] of Object.entries(versions)) console.log(`  ${k.padEnd(18)} ${v}`)

const unique = new Set(Object.values(versions))
if (unique.size !== 1) {
  console.error('\n❌ Manifest versions disagree.')
  process.exit(1)
}

const tag = process.env.GITHUB_REF_NAME
if (tag) {
  const expected = `v${pkg}`
  console.log(`  GITHUB_REF_NAME   ${tag} (expected ${expected})`)
  if (tag !== expected) {
    console.error(`\n❌ Git tag ${tag} does not match manifest version ${pkg}.`)
    console.error(`   Run: npm run bump ${tag.replace(/^v/, '')} && commit + retag.`)
    process.exit(1)
  }
}

console.log('\n✅ All versions consistent.')
