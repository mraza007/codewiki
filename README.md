<p align="center">
  <img src="assets/codewiki-logo.svg" alt="CodeWiki" width="128" height="128">
</p>

<h1 align="center">CodeWiki</h1>

<p align="center">
  <strong>Your codebase, compiled into a living wiki.</strong><br>
  Maintained by LLMs. Searchable by LLMs. Viewable in Obsidian.
</p>

<p align="center">
  <a href="#installation">Installation</a> &middot;
  <a href="#quick-start">Quick Start</a> &middot;
  <a href="#how-it-works">How It Works</a> &middot;
  <a href="#commands">Commands</a> &middot;
  <a href="#integrations">Integrations</a>
</p>

---

## The Problem

Every coding session starts from zero. Your AI agent re-reads files, re-discovers architecture, re-learns patterns. Past decisions and bug fixes are forgotten. There is no persistent understanding of a codebase across sessions.

## The Solution

CodeWiki compiles your codebase into a structured wiki of markdown articles — module overviews, architecture docs, cross-cutting concepts, decisions, and learnings. Your AI agent maintains it automatically. Knowledge compounds across every session.

```
Your Codebase ──► LLM reads code ──► Wiki (.md files) ──► LLM queries wiki
                                          │
                                          ▼
                                     Obsidian / QMD
```

CodeWiki is a thin CLI (`cw`) paired with a Claude Code skill. The CLI handles git ops, metadata, and scaffolding. Claude Code does all the reading, understanding, and writing. No API keys needed — your agent **is** the intelligence.

## Installation

### From source (requires Rust)

```bash
git clone https://github.com/mraza007/codewiki.git
cd codewiki
cargo install --path .
```

### Verify

```bash
cw --help
```

## Quick Start

```bash
# 1. Initialize a wiki for your project
cd your-project
cw init

# 2. Set up your AI agent
cw setup claude-code   # or: cw setup codex

# 3. Start a Claude Code session — the skill triggers automatically
#    Claude reads your codebase and compiles the wiki

# 4. Open in Obsidian
#    Point Obsidian at ~/.codewiki/ as a vault
```

That's it. From now on, every session starts with understanding and ends with updated knowledge.

## How It Works

### The Wiki

Each project gets a wiki at `~/.codewiki/<project>/`:

```
~/.codewiki/
├── my-project/
│   ├── _index.md              # Master index of all articles
│   ├── _architecture.md       # System overview
│   ├── _patterns.md           # Recurring patterns
│   ├── _meta.yaml             # Last compiled commit, timestamps
│   │
│   ├── modules/               # One article per logical module
│   │   ├── auth.md
│   │   ├── database.md
│   │   └── api.md
│   │
│   ├── concepts/              # Cross-cutting concerns
│   │   ├── data-flow.md
│   │   └── error-handling.md
│   │
│   ├── decisions/             # Why things are the way they are
│   │   └── why-postgres.md
│   │
│   ├── learnings/             # Bugs fixed, patterns discovered
│   │   └── auth-token-bug.md
│   │
│   └── queries/               # Past Q&A, filed back in
│       └── how-caching-works.md
│
├── another-project/
│   └── ...
```

### The Session Lifecycle

**Session start:**
1. `cw status` checks what changed since the last compile
2. If first time: full compile — agent reads entire codebase, writes all articles
3. If stale: incremental update — only re-analyze changed files
4. If fresh: skip, start working immediately

**During work:**
The agent queries the wiki instead of re-reading raw source files. Cross-cutting questions like "how does authentication work?" are answered by a single article that connects 8 files across 4 directories.

**Session end:**
1. Bugs fixed become `learnings/<slug>.md`
2. Decisions made become `decisions/<slug>.md`
3. Changed code triggers article updates
4. `cw index` + `cw meta update` keeps everything current

### Article Format

Every article uses YAML frontmatter for metadata:

```markdown
---
title: Authentication Module
type: module
source_files:
  - src/auth/middleware.py
  - src/auth/tokens.py
tags: [auth, middleware, jwt]
---

## Overview
JWT-based authentication using middleware chain...

## Key Components
- `AuthMiddleware` — validates tokens on every request
- `TokenService` — issues and refreshes JWTs

## Data Flow
Request → AuthMiddleware → TokenService → Handler

## Connections
- [[database]] — stores refresh tokens
- [[api]] — all routes pass through auth middleware

## Known Issues
Token refresh has a race condition under high concurrency.
```

The `source_files` field is what makes `cw status` work — when those files change, the article is marked stale.

## Commands

```bash
cw init                  # Scaffold wiki for current repo
cw status                # Show changed files and stale articles
cw path                  # Print wiki path for current repo
cw projects              # List all wikis
cw index                 # Rebuild _index.md from article frontmatter
cw meta update           # Record current commit as "compiled"

cw setup claude-code     # Install skill into Claude Code
cw setup codex           # Install instructions into Codex
cw setup qmd             # Register wiki as QMD search collection

cw uninstall claude-code # Remove from Claude Code
cw uninstall codex       # Remove from Codex
```

## Integrations

### Claude Code

`cw setup claude-code` installs a skill at `~/.claude/skills/codewiki/`. The skill tells Claude Code when and how to compile, query, and update the wiki. It triggers automatically at session start.

### Codex

`cw setup codex` appends wiki instructions to `~/.codex/AGENTS.md`. Codex will check the wiki at session start and update it at session end.

### QMD

[QMD](https://github.com/tobi/qmd) is a local search engine for markdown files with hybrid BM25 + vector + LLM reranker search.

```bash
cw setup qmd                                    # register collection
qmd query "how does auth work" -c codewiki       # search the wiki
```

With QMD as an MCP server, your agent can search the wiki programmatically during a session.

### EchoVault

[EchoVault](https://github.com/mraza007/echovault) provides persistent memory for coding agents. CodeWiki and EchoVault complement each other:

- **CodeWiki** = what the code *is* (compiled understanding)
- **EchoVault** = what *happened* while working on it (decisions, bugs, patterns)

Both write into the same `~/.codewiki/<project>/` directory, so memories and code knowledge live side by side.

### Obsidian

Open `~/.codewiki/` as an Obsidian vault. All wiki articles use `[[backlinks]]` natively, so you get a connected knowledge graph of your codebases out of the box. No plugins required.

## Architecture

```
┌──────────────────────────────────────────────┐
│              Your AI Agent                    │
│         (Claude Code / Codex)                │
│                                              │
│  Reads code ─► Writes articles ─► Queries    │
└──────────┬───────────────┬───────────────────┘
           │               │
     ┌─────▼─────┐  ┌─────▼──────┐
     │  cw CLI   │  │    QMD     │
     │           │  │  (search)  │
     │ git ops   │  │            │
     │ metadata  │  │ BM25 +     │
     │ scaffold  │  │ vector +   │
     │ indexing  │  │ reranker   │
     └─────┬─────┘  └─────┬──────┘
           │               │
     ┌─────▼───────────────▼──────┐
     │    ~/.codewiki/<project>/  │
     │                            │
     │  Markdown files            │
     │  Viewable in Obsidian      │
     │  Searchable by QMD         │
     │  Version-controllable      │
     └────────────────────────────┘
```

**Design principles:**

- **The CLI is not smart.** It handles git diffs, file scaffolding, and metadata. Your AI agent does all the understanding and writing.
- **Plain markdown.** No databases, no proprietary formats. Just `.md` files with YAML frontmatter.
- **Central location.** All wikis live at `~/.codewiki/`. One Obsidian vault for all projects. Survives repo deletion.
- **Session-boundary updates.** Wiki updates happen at session start and end — not continuously. Cost-contained, natural rhythm.

## Why Not Just RAG?

Traditional RAG chunks code, embeds it, and retrieves fragments. You get decontextualized snippets.

CodeWiki is different: an LLM **reads and understands** the code, then writes structured knowledge articles. When you query, you get pre-digested understanding, not raw chunks. The wiki is the semantic layer that RAG skips.

## Inspiration

This project was inspired by [Andrej Karpathy's tweet](https://x.com/karpathy/status/2039805659525644595) about using LLMs to build personal knowledge bases — raw data compiled into wikis, queried and enhanced incrementally.

CodeWiki applies that same pattern to codebases.

## Contributing

Contributions welcome! Some areas that need work:

- **More agent integrations** — Cursor, Windsurf, OpenCode
- **`cw lint`** — LLM health checks over the wiki (find contradictions, gaps)
- **`cw export`** — Generate slides, diagrams from wiki content
- **Multi-repo concepts** — Cross-project articles for shared patterns
- **Team sharing** — Git-backed wikis that teammates can pull

## License

[MIT](LICENSE)
