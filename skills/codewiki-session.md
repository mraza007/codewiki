---
name: codewiki-session
description: Maintain a living wiki of the current codebase. Run at session start to check wiki freshness, compile if needed, and update stale articles.
---

# CodeWiki Session

## When to trigger
At the start of any coding session in a git repository.

## Session Start

1. Run `cw status` to check wiki state.

2. **If no wiki exists** (first time):
   - Run `cw init`
   - Perform full compile (see below)

3. **If "not yet compiled":**
   - Perform full compile (see below)

4. **If stale articles listed:**
   - Read each changed source file
   - Update the corresponding wiki article in `~/.codewiki/<project>/`
   - Run `cw index` to rebuild the index
   - Run `cw meta update`

5. **If "up to date":**
   - Skip, proceed with user's task

## Full Compile

Walk the codebase and create wiki articles:

1. Use `find` or `glob` to list all source files (respect .gitignore)
2. Group files by directory/module
3. For each module, create `modules/<name>.md` with this format:

```
---
title: <Module Name>
type: module
source_files:
  - path/to/file1
  - path/to/file2
tags: [relevant, tags]
---

## Overview
What this module does in 2-3 sentences.

## Key Components
List main functions/classes and their purpose.

## Data Flow
How data enters, transforms, and exits this module.

## Connections
Links to related modules: [[other-module]]

## Known Issues
Anything fragile, incomplete, or worth noting.
```

4. Identify cross-cutting concepts and write `concepts/<name>.md`
5. Write `_architecture.md` — system overview connecting all modules
6. Write `_patterns.md` — recurring patterns in the codebase
7. Run `cw index` then `cw meta update`

## Session End

1. If you fixed a bug, write `learnings/<slug>.md`:
```
---
title: <Short description>
type: learning
source_files: [affected files]
tags: [relevant, tags]
---
What happened, root cause, and fix.
```

2. If you made a design decision, write `decisions/<slug>.md`:
```
---
title: <Decision>
type: decision
tags: [relevant, tags]
---
What was decided, why, and what alternatives were considered.
```

3. If you changed code covered by existing wiki articles, update those articles.
4. Run `cw index` then `cw meta update`.

## Article Guidelines

- Write for a future agent with zero context about this codebase
- Be specific: name files, functions, and types — not vague descriptions
- Use Obsidian `[[backlinks]]` to connect related articles
- Keep articles focused — one module or concept per article
- Include `source_files` in frontmatter so `cw status` can detect staleness
