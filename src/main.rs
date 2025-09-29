use std::{
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use {
    anyhow::{anyhow, bail, Result},
    clap::{Parser, Subcommand},
    trash,
	chrono::prelude::*,
};

#[derive(Debug, Subcommand)]
enum ArgCmd {
    #[command(about = "Send files to be trashed.")]
    Trash {
        #[arg(help = "Files to be sent to the system's trash bin.")]
        files: Vec<PathBuf>,
    },
    #[command(about = "Permanently delete files from the system's trash bin.")]
    Compact {
        #[arg(
            help = "How old a file must be (measured in days), from the time it's recycled, to be eligible for permanent deletion."
        )]
        period: u64,
    },
    #[command(about = "Restores files from the system's trash bin.")]
    Restore {},
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<ArgCmd>,

    files: Option<Vec<PathBuf>>,
}

fn trash_files(files: Vec<PathBuf>) -> Result<()> {
    trash::delete_all(files).or_else(|_| Err(anyhow!("Item(s) could not be deleted.")))?;
    println!("All file(s) trashed successfully.");
    Ok(())
}

fn compact_files(from: u64) -> Result<()> {
	// TODO: measuring this might take some time, so notify the user of that
    let eligible_trash: Vec<_> = trash::os_limited::list()?
        .into_iter()
        .filter(|garbage| {
            garbage.time_deleted as u64
                <= SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .saturating_sub(Duration::from_secs(86400 * from))
                    .as_secs()
        })
        .collect();

    let deleted = eligible_trash.len();
	for garbage in &eligible_trash {
		let dt: DateTime<Utc> = DateTime::from_utc(NaiveDateTime::from_timestamp(garbage.time_deleted, 0), Utc);
		println!("Purged: {}\\{}, originally deleted at: {}", &garbage.original_parent.display(), &garbage.name, &dt.format("%Y-%m-%d %H:%M:%S"));
	}
    trash::os_limited::purge_all(eligible_trash)?;

    println!("{deleted} files(s) permanently deleted!");

    Ok(())
}

fn restore_files() -> Result<()> {
    let mut garbage = trash::os_limited::list()?;
    garbage.sort_by(|a, b| b.time_deleted.cmp(&a.time_deleted));
    let last_item = vec![garbage.into_iter().nth(0).unwrap()];
    trash::os_limited::restore_all(last_item)?;
    Ok(())
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
        Some(ArgCmd::Compact { period }) => compact_files(period)?,
        Some(ArgCmd::Restore {}) => restore_files()?,
    }

    Ok(())
}
