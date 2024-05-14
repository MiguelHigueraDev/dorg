use std::error::Error;
use std::fs::{DirEntry, Metadata};
use std::{fs, io};
use std::time::SystemTime;

pub enum SortingType {
    Month, Day
}

pub enum MetadataError {
    CreationTimeUnavailable,
    IoError(io::Error)
}

pub struct Config {
    pub directory_path: String,
    pub recursive: bool,
    pub sorting: SortingType
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();

        let directory_path = match args.next() {
            Some(arg) => arg,
            None => return Err("Directory not specified"),
        };

        let mut recursive = false;
        let mut sorting = SortingType::Month;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-r" => recursive = true,
                arg if arg.starts_with("sorting=") => {
                    let sorting_str = &arg["-sorting=".len()..];
                    sorting = match sorting_str {
                        "day" => SortingType::Day,
                        "month" => SortingType::Month,
                        _ => return Err("Invalid sorting type"),
                    }
                }
                _ => return Err("Unknown argument"),
            }
        }

        Ok(Config { directory_path, recursive, sorting })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let files = fs::read_dir(config.directory_path)?;

    for file in files {
        match file {
            Ok(file) => {
                move_file(file);
            },
            Err(e) => {
                eprintln!("Error reading file: {e}");
            }
        }
    }

    Ok(())
}

fn move_file(file: DirEntry) {
    let path = file.path();

    let metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(e) => {
            eprintln!("Error reading file metadata: {e}");
            return;
        }
    };

    let creation_time = match get_creation_time(metadata) {
        Ok(ct) => ct,
        Err(_) => {
            eprintln!("Error reading creation time from file.");
            return;
        }
    };


}

fn get_creation_time(metadata: Metadata) -> Result<SystemTime, MetadataError> {
    metadata.created().map_err(|e| {
        if e.kind() == io::ErrorKind::Other {
            MetadataError::CreationTimeUnavailable
        } else {
            MetadataError::IoError(e)
        }
    })
}
