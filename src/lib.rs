use std::error::Error;
use std::fs::{DirEntry, Metadata};
use std::path::{Component, Path, PathBuf};
use std::{fmt, fs, io};
use std::time::SystemTime;
use chrono::{DateTime, Datelike, Utc};

pub enum SortingType {
    Month, Day
}

#[derive(Debug)]
pub enum MetadataError {
    CreationTimeUnavailable,
    IoError(io::Error)
}


impl fmt::Display for MetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetadataError::CreationTimeUnavailable => write!(f, "Creation time is unavailable"),
            MetadataError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl Error for MetadataError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            MetadataError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for MetadataError {
    fn from(error: io::Error) -> Self {
        MetadataError::IoError(error)
    }
}

pub struct Config {
    pub directory_path: PathBuf,
    pub recursive: bool,
    pub sorting: SortingType
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();

        let directory_path = match args.next() {
            Some(arg) => PathBuf::from(arg),
            None => return Err("Directory not specified"),
        };

        let mut recursive = false;
        let mut sorting = SortingType::Month;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-r" => recursive = true,
                arg if arg.starts_with("-sorting=") => {
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
    process_directory(&config.directory_path, config.recursive, &config.sorting)?;
    Ok(())
}

fn process_directory(path: &Path, recursive: bool, sorting: &SortingType) -> Result<(), Box<dyn Error>> {
    let entries = fs::read_dir(path)?;

    for entry in entries {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            if recursive {
                process_directory(&entry.path(), recursive, sorting)?;
            }
        } else {
            move_file(entry, &sorting)?;
        }
    }

    Ok(())
}


fn move_file(file: DirEntry, sorting: &SortingType) -> Result<(), Box<dyn Error>> {
    let original_path = file.path();
    let parent_dir = get_parent_dir(&original_path)
        .ok_or("Error getting the parent directory")?;

    let metadata = file.metadata()?;
    let creation_time = get_creation_time(metadata)?;
    let (year, month, day) = get_year_month_day(creation_time);

    let new_dir = match sorting {
        SortingType::Month => parent_dir.join(year.to_string()).join(month.to_string()),
        SortingType::Day => parent_dir.join(year.to_string()).join(month.to_string()).join(day.to_string()),
    };

    let new_path = new_dir.join(file.file_name());

    fs::create_dir_all(&new_dir)?;
    fs::rename(&original_path, &new_path)?;

    println!("File moved to {:?}", new_path);
    Ok(())
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

fn get_year_month_day(system_time: SystemTime) -> (i32, u32, u32) {
    let datetime: DateTime<Utc> = system_time.into();

    let year = datetime.year();
    let month = datetime.month();
    let day = datetime.day();

    (year, month, day)
}

fn get_parent_dir(path: &Path) -> Option<PathBuf> {
    for component in path.components() {
        if let Component::Normal(root_dir) = component {
            return Some(PathBuf::from(root_dir));
        }
    }
    Some(path.to_path_buf())
}