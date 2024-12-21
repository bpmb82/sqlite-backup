# sqlite-backup

A simple service to create backups of SQLite database files by _recursively scanning a directory and backing up all SQLite database files found_ to a specified backup directory. It will maintain the same directory structure as the source database files.

I use this as a part of my backup strategy for my self hosted services.

## Usage with Docker

1. Build the Docker image:

    ```docker build --load -t sqlite-backup:latest .```

2. Run the Docker container:

    ```docker compose up```
    
### Environment Variables
```
SOURCE_DB: Path to the SQLite database file inside container
BACKUP_DIR: Directory where backups will be stored inside container
INTERVAL: Backup frequency (e.g., 12h, 24h, 7d)
```
### Volume Mounts
Mount your database file and backup directory:

Source directory: ```-v /host/path/db:/data```

Backup directory: ```-v /host/path/backups:/backups```

## Usage without Docker

1. Clone the repository: 

    ```git clone https://github.com/your-username/sqlite-backup.git```

2. Build the project:

    ```cargo build --release```

3. Run the executable:

    ```./target/release/sqlite-backup <command>```

### Commands

- ```backup```: Create a backup of the source directory.
- ```restore```: Restore backup files to the source directory.
- ```daemon```: Run the backup service in daemon mode, can take SOURCE_DIR and BACKUP_DIR environment variables or the arguments --source-dir and --backup-dir.

### Arguments

- ```--source-dir=<source_dir>```: Path to the SQLite database directory.
- ```--backup-dir=<backup_dir>```: Path to the backup directory.

### Examples

```
# Create a backup of the source directory
./target/release/sqlite-backup backup --source-dir=/path/to/db --backup-dir=/path/to/backups

# Restore backup files to the source directory
./target/release/sqlite-backup restore --source-dir=/path/to/db --backup-dir=/path/to/backups

# Run the backup service in daemon mode
./target/release/sqlite-backup daemon --source-dir=/path/to/db --backup-dir=/path/to/backups
```

## License
GNU General Public License v3.0

## Support
For issues and feature requests, please open an issue in the GitHub repository.