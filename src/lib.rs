use std::error::Error;
use std::fs::{DirEntry, Metadata};
use std::path::{Component, Path};
use std::{fs, io};
use std::time::SystemTime;
use chrono::{DateTime, Datelike, Utc};

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
    let original_path = file.path();
    let root_directory = match get_root_directory(&original_path) {
        Some(dir) => dir,
        None => {
            eprintln!("Error getting the root directory.");
            return;
        }
    };

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
            eprintln!("Error reading creation time from file. {original_path:?}");
            return;
        }
    };

    let (year, month) = get_year_month(creation_time);
    let new_dir = root_directory.join(year.to_string()).join(month.to_string());
    let new_path = new_dir.join(file.file_name());


    // Create the new directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&new_dir) {
        eprintln!("Error creating new directory: {e}");
        return;
    }

    match fs::rename(&original_path, &new_path) {
        Err(e) => eprintln!("Error while moving file: {e}"),
        Ok(_) => println!("File moved to {:?}", new_path),
    }

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

fn get_year_month(system_time: SystemTime) -> (i32, u32) {
    let datetime: DateTime<Utc> = system_time.into();

    let year = datetime.year();
    let month = datetime.month();

    (year, month)
}

fn get_root_directory(path: &Path) -> Option<&Path> {
    for component in path.components() {
        if let Component::Normal(root_dir) = component {
            return Some(Path::new(root_dir));
        }
    }
    None
}