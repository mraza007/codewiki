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
   - For each stale article, launch a background Agent to re-read the source files and update the article
   - Once all agents complete, run `cw index` then `cw meta update`

5. **If "up to date":**
   - Skip, proceed with user's task

## Full Compile

Use subagents to parallelize the work. One agent per module means each gets full attention and the compile finishes faster.

### Step 1: Discover modules

List all source files (respect .gitignore), group them by directory/module. Identify 
logical groupings (e.g. `main/models.py` + `main/admin.py` = data layer, `psa_saml/` = auth module).

### Step 2: Launch parallel subagents

For each module, launch a background Agent with this prompt:

> Read every file in [module files list]. Write a wiki article at ~/.codewiki/<project>/modules/<name>.md.
> 
> The article MUST include:
> - YAML frontmatter with title, type: module, source_files (list every file you read), and tags
> - Overview: what this module does in 2-3 sentences
> - Key Components: list ALL public functions, classes, constants with their line numbers
> - Every field on every model/struct, including timestamps and metadata fields
> - Every method, including __str__, clean(), save(), and property methods
> - Data Flow: how data enters, transforms, and exits
> - Connections: Obsidian [[backlinks]] to related modules
> - Known Issues: anything fragile, disabled, or incomplete
>
> IMPORTANT: Do NOT summarize or omit details. List every function, every field, every method.
> Verify line numbers by reading the actual code. Do not guess.
> If a feature is disabled, say WHERE it's disabled (which file, which line), not just "disabled."

### Step 3: Cross-cutting articles

After module agents complete, launch agents for:

- `concepts/` articles for patterns that span multiple modules (error handling, data flow, auth flow)
- `_architecture.md` — system overview connecting all modules
- `_patterns.md` — recurring patterns in the codebase

### Step 4: Verify

Launch a background Agent to verify the wiki:

> Read every article in ~/.codewiki/<project>/. For each article:
> 1. Check that every file listed in source_files actually exists in the repo
> 2. Spot-check 3 function names and line numbers against the actual source
> 3. Check that no major source file is missing from all articles
> 4. Check that _index.md lists every article that exists on disk
> Report any issues found.

Fix any issues the verification agent finds.

### Step 5: Finalize

Run `cw index` then `cw meta update`.

## After Every Completed Task

Do NOT wait until session end. After completing each task (bug fix, feature, refactor), immediately:

1. **Update affected wiki articles.** If you changed `src/auth/middleware.py` and `modules/auth.md` describes that file, update the article now. You have full context right now — you won't remember the details later.

2. **Write learnings.** If you fixed a bug, write `learnings/<slug>.md` immediately:
```
---
title: <Short description>
type: learning
source_files: [affected files]
tags: [relevant, tags]
---
What happened, root cause, and fix.
```

3. **Write decisions.** If you made a design decision, write `decisions/<slug>.md` immediately:
```
---
title: <Decision>
type: decision
tags: [relevant, tags]
---
What was decided, why, and what alternatives were considered.
```

4. **Run `cw meta update`** to record the current commit.

This is part of completing the task, not a separate step. A task isn't done until the wiki reflects what changed.

## Session End

Run `cw index` to rebuild the master index. That's it — everything else was already handled after each task.

## Article Quality Rules

These rules prevent the most common wiki inaccuracies:

**Completeness:**
- List ALL fields on models/structs, including created_at, updated_at, created_by, and any auto-generated fields
- List ALL methods, including properties, clean(), save(), __str__, post(), get()
- If a module has multiple classes, document every one

**Accuracy:**
- Verify line numbers by reading the code, not guessing
- If something is "disabled," state exactly where: "routes commented out in urls.py line 45" not just "disabled"
- Line ranges must cover the full function/block, not stop in the middle
- When describing type inference or detection logic, list the exact trigger values, not a summary

**Frontmatter:**
- source_files must list EVERY file you read to write the article, including test files
- If a module has tests in a separate file (e.g. tests_docx_template.py), include it

**Index:**
- _index.md must list every .md article in the wiki, including _patterns.md and _architecture.md

## Article Format

```
---
title: <Module Name>
type: module
source_files:
  - path/to/file1
  - path/to/file2
  - path/to/test_file.py
tags: [relevant, tags]
---

## Overview
What this module does in 2-3 sentences.

## Key Components
List ALL functions/classes with line numbers and purpose.

## Architecture
Box diagram showing how components in this module interact (include if 3+ components or non-trivial flow).

## Data Flow
How data enters, transforms, and exits this module.

## Connections
Links to related modules: [[other-module]]

## Known Issues
Anything fragile, incomplete, or worth noting. Be specific about what and where.
```

## Markdown Formatting Rules

All wiki articles MUST follow proper markdown formatting:

- **Headings:** Use `##` for top-level sections, `###` for subsections. Never skip levels (no `####` after `##`).
- **Lists:** Use `-` for unordered lists. Use `1.` for ordered/sequential steps. Indent nested lists with 2 spaces.
- **Code:** Use backticks for inline code (`function_name`, `FileName.py`). Use fenced code blocks with language tags for multi-line code:
  ````
  ```python
  def example():
      pass
  ```
  ````
- **Tables:** Use markdown tables for structured comparisons (fields, endpoints, config values):
  ```
  | Name | Type | Description |
  |------|------|-------------|
  | id   | int  | Primary key |
  ```
- **Bold/Italic:** Use `**bold**` for key terms on first mention. Use `*italic*` for file paths in prose.
- **Links:** Use Obsidian `[[backlinks]]` for internal wiki references. Use `[text](url)` for external links.
- **Blank lines:** Always leave a blank line before and after headings, code blocks, and lists.
- **Line length:** No hard wrapping. One sentence per line is acceptable but not required.

## Architecture Diagrams

Every wiki MUST include architecture diagrams to visualize how components connect. Diagrams live in `_architecture.md` and in module articles where they clarify complex flows.

### Required Diagrams

During full compile, generate these diagrams in `_architecture.md`:

1. **System Overview** — High-level box diagram showing all modules and their relationships:
   ```
   ┌─────────────┐     ┌─────────────┐
   │   API Layer  │────►│  Auth Module │
   │  (routes.py) │     │ (middleware) │
   └──────┬──────┘     └──────┬──────┘
          │                    │
          ▼                    ▼
   ┌─────────────┐     ┌─────────────┐
   │  Data Layer  │◄────│   Models    │
   │  (services)  │     │  (orm/db)   │
   └─────────────┘     └─────────────┘
   ```

2. **Request/Data Flow** — Show the path of a typical request or data transformation through the system, labeling each step.

3. **Dependency Graph** — Which modules import/depend on which. Use arrows to show direction.

### Module-Level Diagrams

Each module article (`modules/<name>.md`) should include a diagram when the module has:
- More than 3 interacting classes or components
- A non-trivial data flow (pipeline, middleware chain, event system)
- External integrations (API calls, database, file I/O)

Use this format in the article under a `## Architecture` section:
```
## Architecture

```text
Request ──► Middleware ──► Handler ──► Service ──► DB
                │                        │
                ▼                        ▼
           Auth Check              Cache Layer
`` `
```

### Diagram Style

- Use UTF-8 box-drawing characters (`┌ ┐ └ ┘ ─ │ ► ◄ ▼ ▲`) for boxes and arrows
- Label every box with the component name AND its file/class in parentheses
- Label arrows with the relationship (calls, imports, returns, emits)
- Keep diagrams under 20 lines — split into multiple diagrams if needed
- Wrap diagrams in ` ```text ` fenced blocks so they render correctly in Obsidian

### When to Update Diagrams

- During full compile: generate all diagrams
- After a task that changes module boundaries, adds new modules, or alters data flow: update the affected diagrams
- If a new integration is added (new API, database, queue): add it to the system overview

## General Guidelines

- Write for a future agent with zero context about this codebase
- Be specific: name files, functions, types, and line numbers
- Use Obsidian `[[backlinks]]` to connect related articles
- Keep articles focused — one module or concept per article
- When in doubt, include more detail, not less
