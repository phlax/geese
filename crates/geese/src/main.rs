use std::process;

use anyhow::Result;
use clap::{Parser, Subcommand};
use geese::Storage;

#[derive(Debug, Parser)]
#[command(name = "geese")]
#[command(about = "Manage goose profiles backed by GOOSE_PATH_ROOT")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    List,
    New {
        name: String,
    },
    Copy {
        src: String,
        dest: String,
    },
    Lock {
        name: String,
    },
    Unlock {
        name: String,
    },
    Delete {
        name: String,
    },
    Path {
        name: String,
    },
    Launch {
        name: String,
        #[arg(last = true)]
        args: Vec<String>,
    },
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let storage = Storage::from_env()?;

    match cli.command {
        Commands::List => {
            for profile in storage.list()? {
                if profile.locked {
                    println!("*{}", profile.name);
                } else {
                    println!("{}", profile.name);
                }
            }
        }
        Commands::New { name } => {
            storage.create(&name)?;
        }
        Commands::Copy { src, dest } => {
            storage.copy(&src, &dest)?;
        }
        Commands::Lock { name } => {
            let mut profile = storage.get(&name)?;
            profile.lock()?;
        }
        Commands::Unlock { name } => {
            let mut profile = storage.get(&name)?;
            profile.unlock()?;
        }
        Commands::Delete { name } => {
            storage.delete(&name)?;
        }
        Commands::Path { name } => {
            let profile = storage.get(&name)?;
            println!("{}", profile.path().display());
        }
        Commands::Launch { name, args } => {
            let profile = storage.get(&name)?;
            let mut command = profile.command("goose");
            command.args(args);

            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;

                return Err(command.exec().into());
            }

            #[cfg(not(unix))]
            {
                let status = command.status()?;
                process::exit(status.code().unwrap_or(1));
            }
        }
    }

    Ok(())
}
