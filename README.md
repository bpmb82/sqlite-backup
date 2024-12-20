# sqlite-backup

A simple service to create backups of SQLite database files by recursively scanning a directory and backup all SQLite database files found to a specified backup directory. It will maintain the same directory structure as the source database files.

## Usage with Docker

1. Build the Docker image:

```docker build --load -t sqlite-backup:latest .```


## Environment Variables
```
SOURCE_DB: Path to the SQLite database file inside container
BACKUP_DIR: Directory where backups will be stored inside container
INTERVAL: Backup frequency (e.g., 12h, 24h, 7d)
```
## Volume Mounts
Mount your database file and backup directory:

Source directory: ```-v /host/path/db:/data```

Backup directory: ```-v /host/path/backups:/backups```

## Requirements
* Docker

## License
GNU General Public License v3.0

## Support
For issues and feature requests, please open an issue in the GitHub repository.