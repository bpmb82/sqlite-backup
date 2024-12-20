use rusqlite::{Connection, backup::Backup};
use std::env;
use std::process;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::thread;
use walkdir::WalkDir;

#[derive(Debug)]
struct BackupInfo {
    source_path: PathBuf,
    backup_path: PathBuf,
}

fn check_required_env_vars() -> (String, String) {
    let source_dir = env::var("SOURCE_DIR").unwrap_or_else(|_| {
        eprintln!("SOURCE_DIR environment variable is not set");
        process::exit(1);
    });

    let backup_dir = env::var("BACKUP_DIR").unwrap_or_else(|_| {
        eprintln!("BACKUP_DIR environment variable is not set");
        process::exit(1);
    });

    (source_dir, backup_dir)
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
            Err(e) => {
                eprintln!("Error accessing path: {}", e);
            }
        }
    }

    backup_files
}

fn backup_database(source_path: &Path, backup_path: &Path) -> Result<(), rusqlite::Error> {
    let source = Connection::open(source_path)?;
    let mut dest = Connection::open(backup_path)?;
    
    let backup = Backup::new(&source, &mut dest)?;
    backup.step(-1)?;
    
    Ok(())
}

fn main() {
    let interval_hours = env::var("INTERVAL")
        .unwrap_or_else(|_| "24".to_string())
        .parse::<u64>()
        .unwrap_or(24);
    
    let interval = Duration::from_secs(interval_hours * 3600);
    
    println!("Starting backup service with interval of {} hours", interval_hours);
    
    loop {
        let (source_dir, backup_dir) = check_required_env_vars();
        let backup_files = find_db_files(&source_dir, &backup_dir);
        
        for backup_info in backup_files {
            match backup_database(&backup_info.source_path, &backup_info.backup_path) {
                Ok(_) => println!("Successfully backed up: {}", backup_info.source_path.display()),
                Err(e) => eprintln!("Failed to backup {}: {}", backup_info.source_path.display(), e),
            }
        }
        
        println!("Waiting {} hours until next backup...", interval_hours);
        thread::sleep(interval);
    }
}