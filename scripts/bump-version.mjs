#!/usr/bin/env node
// Bumps version across package.json, src-tauri/tauri.conf.json, and src-tauri/Cargo.toml in lockstep.
// Usage: npm run bump 0.1.2
import { readFileSync, writeFileSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const version = process.argv[2]

if (!version || !/^\d+\.\d+\.\d+(-[\w.]+)?$/.test(version)) {
  console.error('Usage: npm run bump <semver>')
  console.error('  e.g. npm run bump 0.1.2')
  process.exit(1)
}

// package.json
const pkgPath = resolve(root, 'package.json')
const pkg = JSON.parse(readFileSync(pkgPath, 'utf8'))
pkg.version = version
writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n')

// src-tauri/tauri.conf.json
const tauriPath = resolve(root, 'src-tauri/tauri.conf.json')
const tauri = JSON.parse(readFileSync(tauriPath, 'utf8'))
tauri.version = version
writeFileSync(tauriPath, JSON.stringify(tauri, null, 2) + '\n')

// src-tauri/Cargo.toml — only the [package] version, not deps
const cargoPath = resolve(root, 'src-tauri/Cargo.toml')
let cargo = readFileSync(cargoPath, 'utf8')
cargo = cargo.replace(
  /(^\[package\][\s\S]*?^version\s*=\s*)"[^"]+"/m,
  `$1"${version}"`,
)
writeFileSync(cargoPath, cargo)

console.log(`Bumped all manifests to ${version}`)
console.log(`Next:
  git commit -am "Bump version to ${version}"
  git tag v${version}
  git push --follow-tags
  # CI builds and creates a DRAFT release. When ready to ship to users:
  gh release edit v${version} --draft=false`)
