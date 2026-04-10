---
name: research
description: This skill should be used when the user asks to research a topic, investigate a technology, or uses "/research". Researches the topic using web searches and saves structured findings to docs/research/.
version: 1.0.0
---

# Research a Topic

Research the given topic thoroughly and save the findings to a structured markdown file.

## Steps

1. **Generate a slug** from the topic (lowercase, hyphenated, concise — e.g., "websocket-scaling", "sqlite-backup-strategies")
2. **Research the topic** using web searches, fetching relevant pages, and synthesizing findings. Aim for practical, actionable information rather than surface-level overviews.
3. **Create the research file** at `docs/research/<slug>.md` using the template below
4. **Print the file path** and a brief summary of key findings

## Research Template

```markdown
# <Topic Title>

**Researched:** YYYY-MM-DD
**Context:** Why this was researched (infer from the project and conversation context)

---

## Summary
2-3 sentence overview of the key findings.

## Key Findings

### <Finding 1>
Details, with sources where applicable.

### <Finding 2>
Details, with sources where applicable.

### <Finding 3>
Details, with sources where applicable.

## Recommendations
Actionable recommendations specific to this project based on the findings.

## Sources
- [Source title](URL) — brief note on what it covered
```

## Rules

- **Be thorough** — use multiple web searches to cover different angles of the topic. Don't stop at the first result.
- **Be practical** — focus on information that is actionable for this project (Callisto: Node.js/Express/SQLite, deployed on a VPS in Docker behind Cloudflare/Caddy).
- **Cite sources** — include URLs for claims that aren't common knowledge.
- **Compare options** — if the topic involves choosing between alternatives, present a comparison rather than just one option.
- **Note trade-offs** — every technical decision has downsides. Include them.
- **Check recency** — prefer recent sources. Note if information might be outdated.
- **Keep it scannable** — use headers, bullet points, and short paragraphs. Someone should be able to skim and get value.
- If a `docs/research/<slug>.md` file already exists, update it rather than creating a duplicate. Add an "Updated: YYYY-MM-DD" line below the original date.

## User's Topic

