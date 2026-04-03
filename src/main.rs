mod config;
mod frontmatter;
mod meta;
mod setup;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cw", about = "Manage LLM-compiled code wikis")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a wiki for the current repo
    Init,
    /// Show what changed since last compile
    Status,
    /// Update wiki metadata with current commit
    Meta {
        #[command(subcommand)]
        action: MetaAction,
    },
    /// Rebuild _index.md from article frontmatter
    Index,
    /// List all wiki projects
    Projects,
    /// Print wiki path for current repo
    Path,
    /// Set up codewiki for an agent or tool
    Setup {
        #[command(subcommand)]
        target: SetupTarget,
    },
    /// Remove codewiki from an agent or tool
    Uninstall {
        #[command(subcommand)]
        target: UninstallTarget,
    },
}

#[derive(Subcommand)]
enum MetaAction {
    /// Update _meta.yaml with current commit hash
    Update,
}

#[derive(Subcommand)]
enum SetupTarget {
    /// Install codewiki skill for Claude Code
    ClaudeCode,
    /// Install codewiki instructions for Codex
    Codex,
    /// Add codewiki collection to QMD search
    Qmd,
}

#[derive(Subcommand)]
enum UninstallTarget {
    /// Remove codewiki from Claude Code
    ClaudeCode,
    /// Remove codewiki from Codex
    Codex,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init => cmd_init(),
        Commands::Status => cmd_status(),
        Commands::Meta { action } => match action {
            MetaAction::Update => cmd_meta_update(),
        },
        Commands::Index => cmd_index(),
        Commands::Projects => cmd_projects(),
        Commands::Path => cmd_path(),
        Commands::Setup { target } => match target {
            SetupTarget::ClaudeCode => setup::setup_claude_code(),
            SetupTarget::Codex => setup::setup_codex(),
            SetupTarget::Qmd => setup::setup_qmd(),
        },
        Commands::Uninstall { target } => match target {
            UninstallTarget::ClaudeCode => setup::uninstall_claude_code(),
            UninstallTarget::Codex => setup::uninstall_codex(),
        },
    }
}

fn cmd_init() -> anyhow::Result<()> {
    let repo_path = std::env::current_dir()?;
    let wiki = config::wiki_path(&repo_path)?;

    if wiki.exists() {
        let m = meta::WikiMeta::load(&wiki)?;
        println!("Wiki already exists at {}", wiki.display());
        if let Some(ref commit) = m.last_compiled_commit {
            println!("Last compiled: {}", &commit[..7.min(commit.len())]);
        } else {
            println!("Not yet compiled");
        }
        return Ok(());
    }

    let subdirs = ["modules", "concepts", "decisions", "learnings", "queries"];
    for dir in &subdirs {
        std::fs::create_dir_all(wiki.join(dir))?;
    }

    let project = config::project_name(&repo_path)?;
    let m = meta::WikiMeta::new(&project, &repo_path.to_string_lossy());
    m.save(&wiki)?;

    println!("Initialized wiki at {}", wiki.display());
    println!("Project: {}", project);
    println!();
    println!("Wiki is ready for compilation. Subdirectories created:");
    for dir in &subdirs {
        println!("  {}/", dir);
    }

    Ok(())
}

fn cmd_status() -> anyhow::Result<()> {
    let repo_path = std::env::current_dir()?;
    let wiki = config::wiki_path(&repo_path)?;

    if !wiki.exists() {
        println!("No wiki found. Run `cw init` first.");
        return Ok(());
    }

    let m = meta::WikiMeta::load(&wiki)?;

    let last_commit = match m.last_compiled_commit {
        Some(ref c) => c.clone(),
        None => {
            println!("Wiki has not been compiled yet.");
            println!("Run a full compile to generate articles.");
            return Ok(());
        }
    };

    let output = std::process::Command::new("git")
        .args(["diff", "--name-only", &format!("{}..HEAD", last_commit)])
        .current_dir(&repo_path)
        .output()?;

    if !output.status.success() {
        println!(
            "Last compiled commit {} no longer in history.",
            &last_commit[..7.min(last_commit.len())]
        );
        println!("Full recompile recommended.");
        return Ok(());
    }

    let changed_files: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect();

    if changed_files.is_empty() {
        println!("Wiki is up to date.");
        return Ok(());
    }

    println!(
        "Changed since last compile ({}):",
        &last_commit[..7.min(last_commit.len())]
    );
    for f in &changed_files {
        println!("  M {}", f);
    }

    let mut stale_articles: Vec<String> = Vec::new();
    for entry in walkdir::WalkDir::new(&wiki)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
    {
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            if let Ok(Some(fm)) = frontmatter::parse(&content) {
                if let Some(ref sources) = fm.source_files {
                    if sources.iter().any(|s| changed_files.contains(s)) {
                        let rel = entry.path().strip_prefix(&wiki).unwrap_or(entry.path());
                        stale_articles.push(rel.display().to_string());
                    }
                }
            }
        }
    }

    if !stale_articles.is_empty() {
        println!();
        println!("Stale articles:");
        for a in &stale_articles {
            println!("  ! {}", a);
        }
    }

    Ok(())
}

fn cmd_meta_update() -> anyhow::Result<()> {
    let repo_path = std::env::current_dir()?;
    let wiki = config::wiki_path(&repo_path)?;

    if !wiki.exists() {
        anyhow::bail!("No wiki found. Run `cw init` first.");
    }

    let mut m = meta::WikiMeta::load(&wiki)?;
    let commit = meta::current_commit(&repo_path)?;

    m.last_compiled_commit = Some(commit.clone());
    m.last_compiled_at = Some(chrono::Utc::now());
    m.save(&wiki)?;

    println!("Updated meta: commit {}", &commit[..7.min(commit.len())]);
    Ok(())
}

fn cmd_index() -> anyhow::Result<()> {
    let repo_path = std::env::current_dir()?;
    let wiki = config::wiki_path(&repo_path)?;

    if !wiki.exists() {
        anyhow::bail!("No wiki found. Run `cw init` first.");
    }

    let mut entries: Vec<(String, String, String)> = Vec::new();

    for entry in walkdir::WalkDir::new(&wiki)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
    {
        let rel = entry.path().strip_prefix(&wiki).unwrap_or(entry.path());
        let rel_str = rel.display().to_string();

        if rel_str.starts_with('_') {
            continue;
        }

        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            if let Ok(Some(fm)) = frontmatter::parse(&content) {
                let title = fm.title.unwrap_or_else(|| rel_str.clone());
                let article_type = fm.article_type.unwrap_or_else(|| "unknown".to_string());
                entries.push((article_type, rel_str, title));
            }
        }
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0).then(a.2.cmp(&b.2)));

    let mut index = String::from("# Wiki Index\n\n");
    let mut current_type = String::new();

    for (article_type, path, title) in &entries {
        if *article_type != current_type {
            current_type.clone_from(article_type);
            index.push_str(&format!("## {}\n\n", capitalize(&current_type)));
        }
        let link_name = path.trim_end_matches(".md");
        index.push_str(&format!("- [[{}|{}]]\n", link_name, title));
    }

    if entries.is_empty() {
        index.push_str("_No articles yet. Wiki needs compilation._\n");
    }

    let index_path = wiki.join("_index.md");
    std::fs::write(&index_path, index)?;
    println!("Index updated: {} articles", entries.len());

    Ok(())
}

fn cmd_projects() -> anyhow::Result<()> {
    let home = config::wiki_home()?;
    if !home.exists() {
        println!("No wikis found. Run `cw init` in a repo.");
        return Ok(());
    }

    let mut found = false;
    let mut entries: Vec<_> = std::fs::read_dir(&home)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let meta_path = path.join("_meta.yaml");
        if meta_path.exists() {
            if let Ok(m) = meta::WikiMeta::load(&path) {
                found = true;
                let status = match m.last_compiled_commit {
                    Some(ref c) => format!("compiled ({})", &c[..7.min(c.len())]),
                    None => "not compiled".to_string(),
                };
                println!("  {} - {} [{}]", m.project, m.repo_path, status);
            }
        }
    }

    if !found {
        println!("No wikis found. Run `cw init` in a repo.");
    }
    Ok(())
}

fn cmd_path() -> anyhow::Result<()> {
    let repo_path = std::env::current_dir()?;
    let wiki = config::wiki_path(&repo_path)?;
    println!("{}", wiki.display());
    Ok(())
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().to_string() + c.as_str(),
    }
}
