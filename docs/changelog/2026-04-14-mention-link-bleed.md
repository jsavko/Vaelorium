# Fix @-mention link bleed into following text

**Date:** 2026-04-14

## Summary
After selecting a page from the `@`-mention menu, subsequent typing (including the space the user typed to continue the sentence) was being absorbed into the link. The caret landed inside the TipTap Link mark, and Link is `inclusive: true` by default, so stored marks extended the link to the next insertion.

## Bug Fixes
- Mention insert now chains `.unsetMark('link')` after `insertContent`, clearing the caret's stored link mark so the next keystroke starts a plain text node.

## Files Modified
- `src/lib/editor/MentionExtension.ts` — added `.unsetMark('link')` to the suggestion `command` chain.

## Verification
Verified in browser-mock (`npm run dev`, localhost:5173): typing `Hello @Target` → Enter → ` and more text` produces `<p>Hello <a class="wiki-link-inline" href="#page:…">Target Page</a> and more text</p>`. Only the page title is inside the `<a>`.

## Rationale
The link mark's `inclusive: true` default is the right UX for manually-authored links (users should be able to extend a link by typing at its end), so changing it globally would regress that case. Clearing stored marks on the mention insert path targets the exact failure without side effects on other link-editing flows.
