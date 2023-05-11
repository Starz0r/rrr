use std::path::PathBuf;

use anyhow::bail;

use {
    anyhow::{anyhow, Result},
    clap::{Parser, Subcommand},
    trash,
};

#[derive(Debug, Subcommand)]
enum ArgCmd {
    #[command(about = "Send files to be trashed.")]
    Trash {
        #[arg(help = "Files to be sent to the system's trash bin.")]
        files: Vec<PathBuf>,
    },
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<ArgCmd>,

    files: Option<Vec<PathBuf>>,
}

fn trash_files(files: Vec<PathBuf>) -> Result<()> {
    trash::remove_all(files).or_else(|_| Err(anyhow!("Item(s) could not be deleted.")))
}

pub fn main() -> Result<()> {
    let argv = Args::parse();

    match argv.command {
        None => match argv.files {
            Some(files) => trash_files(files)?,
            None => {
                bail!("please pass some file(s) or select a command")
            }
        },
        Some(ArgCmd::Trash { files }) => trash_files(files)?,
    }

    Ok(())
}
