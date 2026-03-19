use std::env;
use std::path::PathBuf;
use std::process::{self, Command};

use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Select};
use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(name = "work", version, about = "Interactive git worktree manager")]
struct Cli {
    #[command(subcommand)]
    command: Option<SubCommand>,
}

#[derive(Parser)]
enum SubCommand {
    /// Add a new worktree
    #[command(alias = "create")]
    Add,
    /// Remove a worktree
    #[command(alias = "delete")]
    Remove,
    /// Select and switch to a worktree
    Go,
    /// Print shell integration script
    Init,
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Parser)]
enum ConfigAction {
    /// Set a config value
    Set { key: String, value: String },
    /// Get a config value
    Get { key: String },
    /// List all config values
    List,
}

#[derive(Deserialize, Serialize, Default)]
struct Config {
    default_repo: Option<String>,
}

const KNOWN_KEYS: &[&str] = &["default_repo"];

fn config_path() -> PathBuf {
    dirs::home_dir()
        .map(|h| h.join(".config/work/work.toml"))
        .unwrap_or_default()
}

fn load_config() -> Config {
    std::fs::read_to_string(config_path())
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}

fn do_config(action: ConfigAction) {
    match action {
        ConfigAction::Set { key, value } => {
            if !KNOWN_KEYS.contains(&key.as_str()) {
                eprintln!("unknown config key: {key}");
                eprintln!("known keys: {}", KNOWN_KEYS.join(", "));
                process::exit(1);
            }
            let mut config = load_config();
            match key.as_str() {
                "default_repo" => config.default_repo = Some(value),
                _ => unreachable!(),
            }
            let path = config_path();
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).unwrap_or_else(|e| {
                    eprintln!("failed to create config directory: {e}");
                    process::exit(1);
                });
            }
            let toml_str = toml::to_string_pretty(&config).unwrap();
            std::fs::write(&path, toml_str).unwrap_or_else(|e| {
                eprintln!("failed to write config: {e}");
                process::exit(1);
            });
            println!("{key} = {}", config.default_repo.unwrap());
        }
        ConfigAction::Get { key } => {
            if !KNOWN_KEYS.contains(&key.as_str()) {
                eprintln!("unknown config key: {key}");
                eprintln!("known keys: {}", KNOWN_KEYS.join(", "));
                process::exit(1);
            }
            let config = load_config();
            let value = match key.as_str() {
                "default_repo" => config.default_repo,
                _ => unreachable!(),
            };
            match value {
                Some(v) => println!("{v}"),
                None => println!("not set"),
            }
        }
        ConfigAction::List => {
            let config = load_config();
            println!(
                "default_repo = {}",
                config.default_repo.as_deref().unwrap_or("not set")
            );
        }
    }
}

fn in_git_repo() -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn resolve_repo_dir(config: &Config) -> Option<PathBuf> {
    if in_git_repo() {
        return None;
    }
    config.default_repo.as_ref().map(|p| {
        let path = PathBuf::from(shellexpand::tilde(p).as_ref());
        if !path.exists() {
            eprintln!("default_repo path does not exist: {}", path.display());
            process::exit(1);
        }
        path
    })
}

/// Returns the root of the main worktree (the one that owns .git as a directory).
fn main_worktree_path(repo_dir: Option<&PathBuf>) -> Result<PathBuf, String> {
    let mut cmd = Command::new("git");
    cmd.args(["worktree", "list", "--porcelain"]);
    if let Some(dir) = repo_dir {
        cmd.current_dir(dir);
    }
    let output = cmd.output().map_err(|e| format!("failed to run git: {e}"))?;
    if !output.status.success() {
        return Err("not in a git repository".into());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    // The first entry is always the main worktree
    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            return Ok(PathBuf::from(path));
        }
    }
    Err("could not determine main worktree".into())
}

struct Worktree {
    path: PathBuf,
    branch: String,
    is_bare: bool,
}

fn list_worktrees(repo_dir: Option<&PathBuf>) -> Result<Vec<Worktree>, String> {
    let mut cmd = Command::new("git");
    cmd.args(["worktree", "list", "--porcelain"]);
    if let Some(dir) = repo_dir {
        cmd.current_dir(dir);
    }
    let output = cmd.output().map_err(|e| format!("failed to run git: {e}"))?;
    if !output.status.success() {
        return Err("not in a git repository".into());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut worktrees = Vec::new();
    let mut current_path: Option<PathBuf> = None;
    let mut current_branch = String::new();
    let mut is_bare = false;

    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            if let Some(p) = current_path.take() {
                worktrees.push(Worktree {
                    path: p,
                    branch: std::mem::take(&mut current_branch),
                    is_bare,
                });
                is_bare = false;
            }
            current_path = Some(PathBuf::from(path));
        } else if let Some(branch_ref) = line.strip_prefix("branch ") {
            current_branch = branch_ref
                .strip_prefix("refs/heads/")
                .unwrap_or(branch_ref)
                .to_string();
        } else if line == "bare" {
            is_bare = true;
        }
    }
    if let Some(p) = current_path {
        worktrees.push(Worktree {
            path: p,
            branch: current_branch,
            is_bare,
        });
    }
    Ok(worktrees)
}

/// Print a cd command that the shell wrapper can eval.
fn emit_cd(path: &std::path::Path) {
    println!("cd:{}", path.display());
}

fn prompt_text(prompt: &str) -> String {
    dialoguer::Input::<String>::new()
        .with_prompt(prompt)
        .interact_text()
        .unwrap_or_else(|_| process::exit(1))
}

fn do_add(repo_dir: Option<&PathBuf>) {
    let name = prompt_text("Worktree name");
    let name = name.trim().replace(' ', "-");
    if name.is_empty() {
        eprintln!("name cannot be empty");
        process::exit(1);
    }

    let now = chrono::Local::now();
    let name = format!("{}-{name}", now.format("%m-%d"));
    let branch = name.clone();

    let main_path = main_worktree_path(repo_dir).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        process::exit(1);
    });
    let worktree_path = main_path.parent().unwrap_or(&main_path).join(&name);

    let mut cmd = Command::new("git");
    cmd.args([
        "worktree",
        "add",
        "-b",
        &branch,
        worktree_path.to_str().unwrap(),
    ]);
    if let Some(dir) = repo_dir {
        cmd.current_dir(dir);
    }
    let status = cmd.status().unwrap_or_else(|e| {
        eprintln!("failed to run git: {e}");
        process::exit(1);
    });
    if !status.success() {
        process::exit(1);
    }

    emit_cd(&worktree_path);
}

fn select_worktree(prompt: &str, repo_dir: Option<&PathBuf>) -> Option<Worktree> {
    let worktrees = list_worktrees(repo_dir).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        process::exit(1);
    });

    let worktrees: Vec<Worktree> = worktrees.into_iter().filter(|w| !w.is_bare).collect();

    if worktrees.is_empty() {
        eprintln!("no worktrees found");
        return None;
    }

    let display: Vec<String> = worktrees
        .iter()
        .map(|w| {
            let dir_name = w
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            if w.branch.is_empty() {
                dir_name
            } else {
                format!("{dir_name} \x1b[0m\x1b[2m[{}]\x1b[0m", w.branch)
            }
        })
        .collect();

    let cwd = env::current_dir().ok();

    let default_idx = cwd
        .as_ref()
        .and_then(|cwd| worktrees.iter().position(|w| w.path == *cwd))
        .unwrap_or(0);

    let theme = ColorfulTheme {
        active_item_style: console::Style::new().green(),
        ..ColorfulTheme::default()
    };

    let selection = Select::with_theme(&theme)
        .with_prompt(prompt)
        .items(&display)
        .default(default_idx)
        .interact_opt()
        .unwrap_or_else(|_| process::exit(1));

    selection.map(|i| worktrees.into_iter().nth(i).unwrap())
}

fn do_remove(repo_dir: Option<&PathBuf>) {
    let main_path = main_worktree_path(repo_dir).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        process::exit(1);
    });

    let Some(worktree) = select_worktree("Select worktree to remove", repo_dir) else {
        return;
    };

    if worktree.path == main_path {
        eprintln!("cannot remove the main worktree");
        process::exit(1);
    }

    let cwd = env::current_dir().ok();

    let spinner = ProgressBar::new_spinner();
    spinner.set_message("Removing worktree...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let mut cmd = Command::new("git");
    cmd.args([
        "worktree",
        "remove",
        worktree.path.to_str().unwrap(),
        "--force",
    ]);
    if let Some(dir) = repo_dir {
        cmd.current_dir(dir);
    }
    let status = cmd.output().unwrap_or_else(|e| {
            spinner.finish_and_clear();
            eprintln!("failed to run git: {e}");
            process::exit(1);
        });

    if !status.status.success() {
        spinner.finish_and_clear();
        eprintln!("{}", String::from_utf8_lossy(&status.stderr));
        process::exit(1);
    }

    // Delete the branch too
    if !worktree.branch.is_empty() {
        let mut cmd = Command::new("git");
        cmd.args(["branch", "-D", &worktree.branch]);
        if let Some(dir) = repo_dir {
            cmd.current_dir(dir);
        }
        let _ = cmd.output();
    }

    spinner.finish_and_clear();

    if cwd.as_deref().is_some_and(|c| c.starts_with(&worktree.path)) {
        emit_cd(&main_path);
    }
}

const SHELL_INIT: &str = r#"work() {
    local output
    output="$(command work "$@")"
    local ret=$?

    if [[ $ret -ne 0 ]]; then
        return $ret
    fi

    while IFS= read -r line; do
        if [[ "$line" == cd:* ]]; then
            cd "${line#cd:}" || return 1
        else
            echo "$line"
        fi
    done <<< "$output"
}"#;

fn do_init() {
    print!("{SHELL_INIT}");
}

fn do_goto(repo_dir: Option<&PathBuf>) {
    let Some(worktree) = select_worktree("Select worktree", repo_dir) else {
        return;
    };
    emit_cd(&worktree.path);
}

fn main() {
    let cli = Cli::parse();
    let config = load_config();

    let repo_dir = resolve_repo_dir(&config);

    match cli.command {
        Some(SubCommand::Init) => do_init(),
        Some(SubCommand::Config { action }) => do_config(action),
        Some(SubCommand::Go) => do_goto(repo_dir.as_ref()),
        Some(SubCommand::Add) => do_add(repo_dir.as_ref()),
        Some(SubCommand::Remove) => do_remove(repo_dir.as_ref()),
        None => do_goto(repo_dir.as_ref()),
    }
}
