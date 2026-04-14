#!/usr/bin/env bash
# Verify that every file path and named symbol referenced in
# `docs/architecture/*.md` still resolves in the working tree. Exits
# non-zero on first miss so a pre-commit hook can fail the commit.
#
# What this catches: a referenced file was renamed/deleted or a named
# function was renamed/removed without updating the brief.
# What it doesn't catch: the brief refers to a real symbol with the
# wrong description, or a refactor that changed semantics while keeping
# the name. Human review still required for those.

set -uo pipefail

cd "$(dirname "$0")/.."

EXIT=0
MISS=()

shopt -s nullglob

# Matches `foo.rs`, `path/to/foo.rs`, `foo.svelte`, `foo.ts`, etc. in
# backticks or bare — anything that looks like a source path and isn't
# obviously a URL.
file_pattern='[a-zA-Z0-9_\./-]+\.(rs|svelte|ts|tsx|js|json|md|toml|css|html)'

# Named symbol patterns: `backup_configure`, `SyncConfig::load`, etc.
# We check symbols that appear in backticks after a file reference or
# in a `mod::name` form. Plain English words get false positives, so
# we require backticks OR a `::` namespacing.

for brief in docs/architecture/*.md; do
  [ -f "$brief" ] || continue

  # 1. File paths referenced in the brief — every one must exist.
  #    Extract everything that looks like a relative path to a source
  #    file and check it.
  while IFS= read -r path; do
    # Skip obvious non-paths (URLs, external doc links, the brief's own
    # file pointer table header, etc.)
    case "$path" in
      http*|*/architecture/*|./*) continue ;;
    esac
    if [ ! -e "$path" ]; then
      MISS+=("$brief → path not found: $path")
      EXIT=1
    fi
    # Path regex requires the match to terminate in a known source
    # extension or a trailing `/` (directory). Otherwise globbing
    # patterns like `Settings*.svelte` in prose get half-captured
    # (`Settings`) and produce false positives.
  done < <(grep -oE "(src-tauri/src/[a-zA-Z0-9_/-]+(\.(rs|toml|json))?|src/lib/[a-zA-Z0-9_/-]+(\.(rs|svelte|ts|tsx|js|css))?|src/app\.css|docs/[a-zA-Z0-9_/-]+(\.(md|txt))?)(/)?" "$brief" | grep -E '\.(rs|svelte|ts|tsx|js|json|toml|md|txt|css)$|/$' | sort -u)

  # 2. Namespaced symbol references `mod::fn` — verify the symbol
  #    appears somewhere in the codebase. This is a presence check,
  #    not a type check.
  while IFS= read -r sym; do
    # Extract the final identifier after the last ::
    last="${sym##*::}"
    # Only symbols that look like real identifiers (lowercase with
    # underscores, or PascalCase). Skip anything with punctuation.
    case "$last" in
      *[!a-zA-Z0-9_]*) continue ;;
    esac
    if ! grep -rq --include='*.rs' --include='*.svelte' --include='*.ts' \
         -E "\\b${last}\\b" src-tauri/src/ src/lib/ 2>/dev/null; then
      MISS+=("$brief → symbol not found: $sym")
      EXIT=1
    fi
  done < <(grep -oE '`[a-zA-Z_][a-zA-Z0-9_]*(::[a-zA-Z_][a-zA-Z0-9_]*)+`' "$brief" | tr -d '`' | sort -u)
done

if [ $EXIT -eq 0 ]; then
  echo "architecture-docs: all references resolve"
else
  echo "architecture-docs: stale references found —"
  for m in "${MISS[@]}"; do
    echo "  $m"
  done
  echo
  echo "Fix: update the brief to point at the new location, or remove"
  echo "the reference if the feature was deleted. See also:"
  echo "docs/architecture/README.md for maintenance guidance."
fi
exit $EXIT
