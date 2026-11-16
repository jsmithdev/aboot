use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::stacks::Stack;

#[derive(Parser)]
#[command(name = "aboot", about = "Scaffold projects with an LLM agent workflow")]
pub struct Cli {
    /// Print version information and exit
    #[arg(short = 'v', long = "version")]
    pub version: bool,

    /// Emit machine-readable JSON
    #[arg(global = true, long)]
    pub json: bool,

    /// Suppress non-essential output
    #[arg(global = true, short = 'q', long)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a new project from the template
    New(NewArgs),
    /// Install workflow files into an existing project
    Add(AddArgs),
    /// Show workflow file status and board progress
    Status(PathArgs),
    /// Manage BOARD.md items
    Board {
        #[command(subcommand)]
        cmd: BoardCmd,
    },
    /// Validate workflow files
    Doctor(PathArgs),
    /// Re-apply managed sections from the current template
    Update(UpdateArgs),
    /// List aboot projects in a workspace
    List(PathArgs),
}

#[derive(Args, Clone)]
pub struct PathArgs {
    /// Project directory (default: current directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

#[derive(Args, Clone)]
pub struct NewArgs {
    /// Project name (also the directory name)
    pub name: String,

    /// Project description (seeded into AGENTS.md, .aboot/PROJECT.md and the pyproject stub)
    #[arg(long)]
    pub description: Option<String>,

    /// First BOARD.md item, placed under NOW
    #[arg(long)]
    pub task: Option<String>,

    /// Language stack preset
    #[arg(long, value_enum)]
    pub stack: Option<Stack>,

    /// Use a custom local template directory
    #[arg(long)]
    pub template: Option<PathBuf>,

    /// Skip git init/commit
    #[arg(long)]
    pub no_git: bool,

    /// Add a git remote named `origin`
    #[arg(long)]
    pub git_remote: Option<String>,

    /// Add a LICENSE file for the given SPDX identifier
    #[arg(long)]
    pub license: Option<String>,

    /// Which agent instruction file(s) to write
    #[arg(long, value_enum)]
    pub agent: Option<AgentMode>,

    /// Show what would be done without writing anything
    #[arg(long)]
    pub dry_run: bool,

    /// Allow an existing non-empty target directory and overwrite existing files
    #[arg(long)]
    pub force: bool,
}

#[derive(Args, Clone)]
pub struct AddArgs {
    /// Project directory (default: current directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Project description (inferred from README.md when omitted)
    #[arg(long)]
    pub description: Option<String>,

    /// First BOARD.md item under NOW (only applied when BOARD.md is created)
    #[arg(long)]
    pub task: Option<String>,

    /// Language stack preset (defaults to detection from build files)
    #[arg(long, value_enum)]
    pub stack: Option<Stack>,

    /// Use a custom local template directory
    #[arg(long)]
    pub template: Option<PathBuf>,

    /// Install only the markdown workflow files (skips justfile, .editorconfig, and stack files)
    #[arg(long, short = 'm')]
    pub md_only: bool,

    /// Install only the agent instruction file (AGENTS.md and/or CLAUDE.md)
    #[arg(long, short = 'a')]
    pub agent_only: bool,

    /// Which agent instruction file(s) to write
    #[arg(long, value_enum)]
    pub agent: Option<AgentMode>,

    /// Show what would be done without writing anything
    #[arg(long)]
    pub dry_run: bool,

    /// Overwrite existing files
    #[arg(long)]
    pub force: bool,
}

#[derive(Subcommand, Clone)]
pub enum BoardCmd {
    /// List BOARD.md items
    List(PathArgs),
    /// Add an item to a section
    Add {
        /// Item text
        text: Vec<String>,
        /// Project directory (default: current directory)
        #[arg(long, default_value = ".")]
        path: PathBuf,
        /// Target section
        #[arg(long, value_enum, default_value_t = BoardSection::Now)]
        section: BoardSection,
    },
    /// Mark an item done (by 1-based index or text match)
    Done {
        /// Item index or text to match
        target: String,
        /// Project directory (default: current directory)
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum BoardSection {
    Now,
    Next,
    Later,
}

impl BoardSection {
    pub fn header(&self) -> &'static str {
        match self {
            BoardSection::Now => "NOW",
            BoardSection::Next => "NEXT",
            BoardSection::Later => "LATER",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum, Default)]
pub enum AgentMode {
    /// Write AGENTS.md (default)
    #[default]
    Agents,
    /// Write CLAUDE.md
    Claude,
    /// Write both AGENTS.md and CLAUDE.md
    Both,
}

impl AgentMode {
    pub fn parse(s: &str) -> Option<AgentMode> {
        match s {
            "agents" => Some(AgentMode::Agents),
            "claude" => Some(AgentMode::Claude),
            "both" => Some(AgentMode::Both),
            _ => None,
        }
    }

    pub fn files(&self) -> Vec<&'static str> {
        match self {
            AgentMode::Agents => vec!["AGENTS.md"],
            AgentMode::Claude => vec!["CLAUDE.md"],
            AgentMode::Both => vec!["AGENTS.md", "CLAUDE.md"],
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AgentMode::Agents => "agents",
            AgentMode::Claude => "claude",
            AgentMode::Both => "both",
        }
    }
}

#[derive(Args, Clone)]
pub struct UpdateArgs {
    /// Project directory (default: current directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Show what would change without writing
    #[arg(long)]
    pub dry_run: bool,

    /// Re-adopt managed files that lost their markers (full overwrite)
    #[arg(long)]
    pub force: bool,
}
