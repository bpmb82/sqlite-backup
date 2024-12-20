use clap::{Parser, Subcommand};
use rusqlite::{backup, Connection};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::thread;
use std::time::Duration;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true)]
    source_dir: Option<String>,

    #[arg(long, global = true)]
    backup_dir: Option<String>,

    #[arg(long, global = true)]
    interval: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    Backup,
    Restore {
        #[arg(short = 'y')]
        yes: bool,
    },
    Daemon,
}

#[derive(Debug)]
struct BackupInfo {
    source_path: PathBuf,
    backup_path: PathBuf,
}

fn get_config(cli: &Cli, is_daemon: bool) -> (String, String, u64) {
    let source_dir = if is_daemon {
        cli.source_dir.clone().or_else(|| env::var("SOURCE_DIR").ok())
    } else {
        cli.source_dir.clone()
    }.unwrap_or_else(|| {
        eprintln!("SOURCE_DIR not provided via argument or environment variable");
        process::exit(1);
    });

    let backup_dir = if is_daemon {
        cli.backup_dir.clone().or_else(|| env::var("BACKUP_DIR").ok())
    } else {
        cli.backup_dir.clone()
    }.unwrap_or_else(|| {
        eprintln!("BACKUP_DIR not provided via argument or environment variable");
        process::exit(1);
    });

    let interval = if is_daemon {
        cli.interval.clone()
            .or_else(|| env::var("INTERVAL").ok())
            .unwrap_or_else(|| "24".to_string())
    } else {
        cli.interval.clone().unwrap_or_else(|| "24".to_string())
    }.parse::<u64>()
    .unwrap_or(24);

    (source_dir, backup_dir, interval)
}


fn find_db_files(source_dir: &str, backup_dir: &str) -> Vec<BackupInfo> {
    let mut backup_files = Vec::new();
    let source_base = Path::new(source_dir);
    let backup_base = Path::new(backup_dir);

    for entry in WalkDir::new(source_dir) {
        match entry {
            Ok(entry) => {
                if entry.path().extension().and_then(|ext| ext.to_str()) == Some("db") {
                    let relative_path = entry.path().strip_prefix(source_base).unwrap();
                    let backup_full_path = backup_base.join(relative_path);

                    if let Some(parent) = backup_full_path.parent() {
                        match fs::create_dir_all(parent) {
                            Ok(_) => {
                                backup_files.push(BackupInfo {
                                    source_path: entry.path().to_path_buf(),
                                    backup_path: backup_full_path,
                                });
                            }
                            Err(e) => {
                                eprintln!("Failed to create backup directory {}: {}", parent.display(), e);
                                process::exit(1);
                            }
                        }
                    }
                }
            }
            Err(e) => eprintln!("Error accessing path: {}", e),
        }
    }
    backup_files
}

fn find_backup_files(backup_dir: &str, source_dir: &str) -> Vec<BackupInfo> {
    let mut restore_files = Vec::new();
    let backup_base = Path::new(backup_dir);
    let source_base = Path::new(source_dir);

    for entry in WalkDir::new(backup_dir) {
        match entry {
            Ok(entry) => {
                if entry.path().extension().and_then(|ext| ext.to_str()) == Some("db") {
                    let relative_path = entry.path().strip_prefix(backup_base).unwrap();
                    let source_full_path = source_base.join(relative_path);

                    if let Some(parent) = source_full_path.parent() {
                        match fs::create_dir_all(parent) {
                            Ok(_) => {
                                restore_files.push(BackupInfo {
                                    source_path: source_full_path,
                                    backup_path: entry.path().to_path_buf(),
                                });
                            }
                            Err(e) => {
                                eprintln!("Failed to create source directory {}: {}", parent.display(), e);
                                process::exit(1);
                            }
                        }
                    }
                }
            }
            Err(e) => eprintln!("Error accessing path: {}", e),
        }
    }
    restore_files
}

fn backup_database(source_path: &Path, backup_path: &Path) -> Result<(), rusqlite::Error> {
    let source = Connection::open(source_path)?;
    let mut dest = Connection::open(backup_path)?;
    let backup = backup::Backup::new(&source, &mut dest)?;
    backup.step(-1)?;
    Ok(())
}

fn confirm_restore(path: &Path) -> bool {
    print!("File {} exists. Overwrite? [y/N] ", path.display());
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    input.trim().eq_ignore_ascii_case("y")
}

fn run_backup(source_dir: &str, backup_dir: &str) {
    let backup_files = find_db_files(source_dir, backup_dir);
    for backup_info in backup_files {
        match backup_database(&backup_info.source_path, &backup_info.backup_path) {
            Ok(_) => println!("Successfully backed up: {}", backup_info.source_path.display()),
            Err(e) => eprintln!("Failed to backup {}: {}", backup_info.source_path.display(), e),
        }
    }
}

fn run_restore(source_dir: &str, backup_dir: &str, force: bool) {
    let restore_files = find_backup_files(backup_dir, source_dir);
    for restore_info in restore_files {
        if restore_info.source_path.exists() && !force {
            if !confirm_restore(&restore_info.source_path) {
                println!("Skipping {}", restore_info.source_path.display());
                continue;
            }
        }
        
        match backup_database(&restore_info.backup_path, &restore_info.source_path) {
            Ok(_) => println!("Successfully restored: {}", restore_info.source_path.display()),
            Err(e) => eprintln!("Failed to restore {}: {}", restore_info.source_path.display(), e),
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Backup => {
            let (source_dir, backup_dir, _) = get_config(&cli, false);
            run_backup(&source_dir, &backup_dir);
        }
        Commands::Restore { yes } => {
            let (source_dir, backup_dir, _) = get_config(&cli, false);
            run_restore(&source_dir, &backup_dir, *yes);
        }
        Commands::Daemon => {
            let (source_dir, backup_dir, interval) = get_config(&cli, true);
            println!("Starting backup service with interval of {} hours", interval);
            loop {
                run_backup(&source_dir, &backup_dir);
                println!("Waiting {} hours until next backup...", interval);
                thread::sleep(Duration::from_secs(interval * 3600));
            }
        }
    }
}

