---
name: brainstorm
description: Work through a product idea that isn't fully fleshed out yet. Research a problem space, then brainstorm solutions interactively with the user. Output feeds directly into /plan.
version: 1.0.0
---

# Brainstorm a Product Idea

Help the user explore and refine a product idea through structured research and collaborative brainstorming. This is a two-phase process.

## Phase 1: Research

When the user provides a problem or idea, **research it first** before brainstorming:

1. **Understand the problem space** — Use WebSearch, WebFetch, and codebase exploration to gather context:
   - What existing solutions exist? What are their strengths and gaps?
   - Who experiences this problem? How severe is it?
   - What technical approaches are common?
   - Any relevant patterns in the current codebase?

2. **Present a research brief** — Summarize findings in a concise format:
   - **Problem Statement:** Clear articulation of the problem
   - **Target Users:** Who this affects
   - **Existing Solutions:** What's out there and where they fall short
   - **Key Insights:** Surprising or important findings from research
   - **Open Questions:** Things worth discussing before brainstorming

3. **Ask the user** if they want to dig deeper on anything or move to brainstorming.

## Phase 2: Brainstorm

Once research is done and the user is ready to brainstorm:

1. **Generate ideas** — Produce 5-8 distinct solution concepts, ranging from minimal to ambitious. For each idea:
   - **Name:** A short memorable label
   - **Core Concept:** 1-2 sentence description
   - **How it works:** Brief mechanics
   - **Strengths:** Why this could work well
   - **Risks:** What could go wrong or be hard

2. **Facilitate discussion** — After presenting ideas:
   - Ask the user which ideas resonate and which don't
   - Combine, remix, or refine ideas based on feedback
   - Challenge assumptions constructively
   - Keep narrowing toward a direction the user is excited about

3. **Converge on a direction** — When the user has a preferred direction, synthesize it into a clear product concept:
   - **Product Concept:** What it is in one paragraph
   - **Key Features:** The core capabilities (prioritized)
   - **User Flow:** How someone would actually use it
   - **Technical Considerations:** High-level implementation notes
   - **MVP Scope:** The smallest version worth building

4. **Save the brainstorm** — Write the output to `.claude/brainstorms/YYYY-MM-DD-<slug>.md` using the template below.

5. **Bridge to planning** — Tell the user they can now run `/plan` with the brainstorm output to create an actionable implementation plan.

## Brainstorm Output Template

```markdown
---
status: complete
---
# <Product Idea Title>

**Date:** YYYY-MM-DD

---

## Problem Statement
What problem this solves and why it matters.

## Research Summary
Key findings from the research phase. Link to sources where relevant.

## Target Users
Who this is for and what they care about.

## Explored Ideas

### 1. <Idea Name>
- **Concept:** What it is
- **Verdict:** Chosen / Rejected / Partially incorporated
- **Notes:** Why

### 2. <Idea Name>
...

## Chosen Direction

### Product Concept
One paragraph describing the product.

### Key Features (prioritized)
1. Feature 1 — why it matters
2. Feature 2 — why it matters
3. Feature 3 — why it matters

### User Flow
Step-by-step how a user would interact with this.

### Technical Considerations
High-level notes on implementation approach.

### MVP Scope
The minimum version worth building.

## Open Questions
Anything still unresolved that should be addressed during planning.
```

## Rules

- **Always research before brainstorming.** Don't jump to solutions without understanding the problem.
- **Be a thought partner, not a yes-machine.** Push back on weak ideas, surface risks, and challenge assumptions.
- **Keep the user in the driver's seat.** Present options and facilitate decisions — don't make them unilaterally.
- **Quantity before quality in ideation.** Generate many ideas first, then refine. Bad ideas can spark good ones.
- **Stay grounded.** If the user's codebase is relevant, tie ideas back to what already exists.
- **Don't over-engineer the brainstorm.** This is exploration, not specification. Keep things loose until convergence.
- After saving the brainstorm file, print the path and remind the user to run `/plan <brief description>` to turn it into an actionable plan.

## User's Idea

$ARGUMENTS
