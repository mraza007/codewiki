use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::config;

// ---------------------------------------------------------------------------
// Skill content (embedded)
// ---------------------------------------------------------------------------

const SKILL_MD: &str = include_str!("../skills/codewiki-session.md");

const CODEX_AGENTS_SECTION: &str = r#"

## CodeWiki — Codebase Knowledge

You have a compiled wiki of this codebase at `~/.codewiki/`. Use it.

### Session start — MANDATORY

Before doing any work, check the wiki state:

```bash
cw status
```

If the wiki is stale or uncompiled, update it:
- Read changed source files
- Update the corresponding wiki articles in `~/.codewiki/<project>/`
- Run `cw index` then `cw meta update`

If no wiki exists, run `cw init` and compile from scratch.

### During work

When you need to understand how a module works, read the wiki article first:
```bash
cat ~/.codewiki/<project>/modules/<name>.md
```

### Session end — MANDATORY

Before finishing any task that involved code changes:

1. If you fixed a bug, create `~/.codewiki/<project>/learnings/<slug>.md`
2. If you made a design decision, create `~/.codewiki/<project>/decisions/<slug>.md`
3. Update any wiki articles affected by your code changes
4. Run `cw index` then `cw meta update`

### Article format

All wiki articles use YAML frontmatter:

```yaml
---
title: Module Name
type: module
source_files:
  - path/to/file
tags: [relevant, tags]
---
```

### Rules

- Check wiki before working. Update wiki before finishing. No exceptions.
- Write for a future agent with zero context.
- Include `source_files` in frontmatter so `cw status` can detect staleness.
"#;

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

fn claude_home() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
}

fn codex_home() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".codex")
}

// ---------------------------------------------------------------------------
// Claude Code setup
// ---------------------------------------------------------------------------

pub fn setup_claude_code() -> Result<()> {
    let home = claude_home();
    let skill_dir = home.join("skills").join("codewiki");
    let skill_path = skill_dir.join("SKILL.md");

    if skill_path.exists() {
        println!("Skill already installed at {}", skill_path.display());
        println!("Updating...");
    }

    std::fs::create_dir_all(&skill_dir)
        .with_context(|| format!("Failed to create {}", skill_dir.display()))?;
    std::fs::write(&skill_path, SKILL_MD)
        .with_context(|| format!("Failed to write {}", skill_path.display()))?;

    println!("Installed codewiki skill to {}", skill_path.display());
    println!();
    println!("Claude Code will now use the codewiki skill to maintain");
    println!("your codebase wiki at session start and end.");

    Ok(())
}

// ---------------------------------------------------------------------------
// Codex setup
// ---------------------------------------------------------------------------

pub fn setup_codex() -> Result<()> {
    let home = codex_home();
    let agents_path = home.join("AGENTS.md");

    let existing = std::fs::read_to_string(&agents_path).unwrap_or_default();

    if existing.contains("## CodeWiki") {
        println!("CodeWiki already installed in {}", agents_path.display());
        return Ok(());
    }

    std::fs::create_dir_all(&home)
        .with_context(|| format!("Failed to create {}", home.display()))?;

    let content = format!("{}\n{}", existing.trim_end(), CODEX_AGENTS_SECTION);
    std::fs::write(&agents_path, content)
        .with_context(|| format!("Failed to write {}", agents_path.display()))?;

    println!("Installed codewiki instructions to {}", agents_path.display());

    Ok(())
}

// ---------------------------------------------------------------------------
// QMD setup
// ---------------------------------------------------------------------------

pub fn setup_qmd() -> Result<()> {
    let wiki_home = config::wiki_home()?;

    if !wiki_home.exists() {
        println!("No wikis found at {}.", wiki_home.display());
        println!("Run `cw init` in a repo first.");
        return Ok(());
    }

    // Check if qmd is available
    let qmd_check = std::process::Command::new("qmd")
        .arg("--help")
        .output();

    if qmd_check.is_err() || !qmd_check.unwrap().status.success() {
        println!("qmd not found in PATH.");
        println!("Install QMD first: https://github.com/tobi/qmd");
        return Ok(());
    }

    // Add collection
    let output = std::process::Command::new("qmd")
        .args(["collection", "add", &wiki_home.to_string_lossy(), "--name", "codewiki"])
        .output()
        .context("Failed to run qmd collection add")?;

    if output.status.success() {
        println!("Added codewiki collection to QMD: {}", wiki_home.display());
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("already exists") {
            println!("QMD collection 'codewiki' already exists.");
        } else {
            println!("QMD collection add output: {}", String::from_utf8_lossy(&output.stdout));
            if !stderr.is_empty() {
                println!("stderr: {}", stderr);
            }
        }
    }

    // Run embed to index
    println!("Indexing wiki articles...");
    let embed = std::process::Command::new("qmd")
        .arg("embed")
        .output()
        .context("Failed to run qmd embed")?;

    if embed.status.success() {
        println!("QMD indexing complete.");
    } else {
        println!("QMD embed warning: {}", String::from_utf8_lossy(&embed.stderr));
    }

    println!();
    println!("You can now search your wiki:");
    println!("  qmd query \"how does auth work\" -c codewiki");

    Ok(())
}

// ---------------------------------------------------------------------------
// Uninstall helpers
// ---------------------------------------------------------------------------

pub fn uninstall_claude_code() -> Result<()> {
    let skill_dir = claude_home().join("skills").join("codewiki");
    if skill_dir.exists() {
        std::fs::remove_dir_all(&skill_dir)?;
        println!("Removed codewiki skill from Claude Code.");
    } else {
        println!("Nothing to remove.");
    }
    Ok(())
}

pub fn uninstall_codex() -> Result<()> {
    let agents_path = codex_home().join("AGENTS.md");
    if let Ok(content) = std::fs::read_to_string(&agents_path) {
        if content.contains("## CodeWiki") {
            // Remove the CodeWiki section
            let cleaned = remove_section(&content, "## CodeWiki");
            std::fs::write(&agents_path, cleaned.trim().to_string() + "\n")?;
            println!("Removed codewiki from Codex AGENTS.md.");
        } else {
            println!("Nothing to remove.");
        }
    } else {
        println!("Nothing to remove.");
    }
    Ok(())
}

fn remove_section(content: &str, header: &str) -> String {
    let mut result = String::new();
    let mut skipping = false;

    for line in content.lines() {
        if line.starts_with(header) {
            skipping = true;
            continue;
        }
        if skipping && line.starts_with("## ") {
            skipping = false;
        }
        if !skipping {
            result.push_str(line);
            result.push('\n');
        }
    }

    result
}
