use clap::{Parser, Subcommand};

use crate::config::{missing_config_message, Config, LoadedConfig};
use crate::error::{anyhow, bail, Result};
use crate::launcher::{
    display_binary, launch_profile, spawn_foreground, wait_for_children, LaunchOptions,
};
use crate::paths::{profile_paths, resolve_paths};

#[derive(Debug, Parser)]
#[command(author, version, about = "Launch isolated Goose desktop profiles")]
struct Cli {
    #[arg(long, global = true, help = "Alias for launch-all")]
    get_gander: bool,
    #[arg(
        short,
        long,
        global = true,
        help = "Keep geese attached and wait for children"
    )]
    foreground: bool,
    #[arg(
        short,
        long,
        global = true,
        help = "Print resolved paths, commands, and env diffs"
    )]
    verbose: bool,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Clone, Subcommand)]
enum Command {
    LaunchAll,
    Launch { name: String },
    List,
    Paths,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let paths = resolve_paths()?;

    if matches!(cli.command, Some(Command::Paths)) {
        println!("config: {}", paths.config_file.display());
        println!("data_root: {}", paths.data_root.display());
        return Ok(());
    }

    let config = match Config::load(&paths)? {
        LoadedConfig::Missing { expected_path } => {
            println!("{}", missing_config_message(&expected_path));
            return Ok(());
        }
        LoadedConfig::Loaded(config) => config,
    };

    let command = cli.command.clone().unwrap_or(Command::LaunchAll);

    match command {
        Command::LaunchAll => launch_all(&config, &paths, &cli),
        Command::Launch { name } => launch_one(&config, &paths, &cli, &name),
        Command::List => list_profiles(&config, &paths),
        Command::Paths => unreachable!(),
    }
}

fn list_profiles(config: &Config, paths: &crate::paths::ResolvedPaths) -> Result<()> {
    println!("NAME\tAPP_ID\tDATA_DIR\tBINARY");
    for name in config.profile_names() {
        let effective = config.effective_profile(name)?;
        let profile_paths = profile_paths(paths, name);
        println!(
            "{}\tgoose-{}\t{}\t{}",
            name,
            name,
            profile_paths.root.display(),
            display_binary(&effective.binary)
        );
    }
    Ok(())
}

fn launch_one(
    config: &Config,
    paths: &crate::paths::ResolvedPaths,
    cli: &Cli,
    name: &str,
) -> Result<()> {
    let effective = config.effective_profile(name)?;
    let launch_paths = profile_paths(paths, name);
    let options = LaunchOptions {
        foreground: cli.foreground,
        verbose: cli.verbose,
    };

    if cli.foreground {
        let (result, child) = spawn_foreground(&effective, &launch_paths, &options)?;
        println!(
            "▸ launching {} (app_id={}, pid={})",
            name, result.app_id, result.pid
        );
        let exit_code = wait_for_children(vec![child])?;
        if exit_code != 0 {
            bail!("profile '{}' exited with status {}", name, exit_code);
        }
        Ok(())
    } else {
        let result = launch_profile(&effective, &launch_paths, &options)?;
        println!(
            "▸ launching {} (app_id={}, pid={})",
            name, result.app_id, result.pid
        );
        Ok(())
    }
}

fn launch_all(config: &Config, paths: &crate::paths::ResolvedPaths, cli: &Cli) -> Result<()> {
    let options = LaunchOptions {
        foreground: cli.foreground,
        verbose: cli.verbose,
    };

    let mut errors = Vec::new();

    if cli.foreground {
        let mut children = Vec::new();
        for name in config.profile_names() {
            let effective = match config.effective_profile(name) {
                Ok(value) => value,
                Err(error) => {
                    eprintln!("error: {error:#}");
                    errors.push(name.to_owned());
                    continue;
                }
            };
            let launch_paths = profile_paths(paths, name);
            match spawn_foreground(&effective, &launch_paths, &options) {
                Ok((result, child)) => {
                    println!(
                        "▸ launching {} (app_id={}, pid={})",
                        name, result.app_id, result.pid
                    );
                    children.push(child);
                }
                Err(error) => {
                    eprintln!("error: failed to launch {}: {error:#}", name);
                    errors.push(name.to_owned());
                }
            }
        }

        if !children.is_empty() {
            let exit_code = wait_for_children(children)?;
            if exit_code != 0 {
                errors.push(format!("child exit status {exit_code}"));
            }
        }
    } else {
        for name in config.profile_names() {
            let effective = match config.effective_profile(name) {
                Ok(value) => value,
                Err(error) => {
                    eprintln!("error: {error:#}");
                    errors.push(name.to_owned());
                    continue;
                }
            };
            let launch_paths = profile_paths(paths, name);
            match launch_profile(&effective, &launch_paths, &options) {
                Ok(result) => println!(
                    "▸ launching {} (app_id={}, pid={})",
                    name, result.app_id, result.pid
                ),
                Err(error) => {
                    eprintln!("error: failed to launch {}: {error:#}", name);
                    errors.push(name.to_owned());
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(anyhow!("failed to launch {} profile(s)", errors.len()))
    }
}
