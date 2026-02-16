use clap::{Parser, Subcommand};

/// Vibe Kanban CLI - Terminal-based real-time task list
#[derive(Parser, Debug)]
#[command(name = "vibe-kanban-cli")]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Vibe Kanban server URL
    #[arg(short, long, default_value = "http://localhost:5173")]
    pub server: String,

    /// Enable debug logging
    #[arg(short, long)]
    pub debug: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create a task and start an attempt immediately
    Create {
        /// Project ID or name
        #[arg(long)]
        project: String,

        /// Prompt for the task (stored as description)
        #[arg(long)]
        prompt: String,

        /// Optional title (defaults to a shortened prompt)
        #[arg(long)]
        title: Option<String>,

        /// Task status: todo, inprogress, inreview, done, cancelled
        #[arg(long, default_value = "todo")]
        status: String,

        /// Tool/executor to use (e.g. codex, claude-code, cursor, gemini)
        #[arg(long, alias = "executor", default_value = "codex")]
        tool: String,

        /// Model/variant for the executor
        #[arg(long)]
        model: Option<String>,

        /// Repo/worktree to use (name, display name, or UUID). Can be repeated.
        /// Use "repo@branch" to override per-repo branch.
        #[arg(long = "repo", alias = "worktree")]
        repos: Vec<String>,

        /// Branch name (default branch by default)
        #[arg(long)]
        branch: Option<String>,

        /// Watch the created task in real time
        #[arg(long)]
        watch: bool,
    },

    /// Watch tasks in real time (board view or single task)
    Watch {
        /// Project ID or name (required for board or slug watch)
        #[arg(long)]
        project: Option<String>,

        /// Task ID to watch
        #[arg(long)]
        task: Option<String>,

        /// Task slug (derived from title) to watch
        #[arg(long)]
        slug: Option<String>,
    },
    /// List projects available on the server
    Projects {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Manage projects
    Project {
        #[command(subcommand)]
        command: ProjectCommand,
    },
    /// Manage a local Vibe Kanban server process
    Server {
        #[command(subcommand)]
        command: ServerCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProjectCommand {
    /// Add a project from a local repository path
    Add {
        /// Path to the repository (use '.' for current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Override project name (defaults to folder name)
        #[arg(long)]
        name: Option<String>,
        /// Override repository display name (defaults to project name)
        #[arg(long)]
        display_name: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ServerCommand {
    /// Start the server (optionally in the background)
    Start {
        /// Command to launch the server
        #[arg(long, default_value = "pnpm run dev")]
        command: String,
        /// Run in the background
        #[arg(long)]
        background: bool,
        /// Log file path (used only when --background is set)
        #[arg(long, default_value = "vibe-kanban-server.log")]
        log: String,
    },
}
