use std::error::Error;
use std::fs::{DirEntry, Metadata};
use std::path::{Component, Path, PathBuf};
use std::{fmt, fs, io};
use std::time::SystemTime;
use chrono::{DateTime, Datelike, Utc};

pub enum SortType {
    Created, Modified
}

pub enum Mode {
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
    pub mode: Mode,
    pub sort_type: SortType,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();

        let directory_path = match args.next() {
            Some(arg) => PathBuf::from(arg),
            None => return Err("Directory not specified"),
        };

        let mut recursive = false;
        let mut mode = Mode::Month;
        let mut sort_type = SortType::Created;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-r" => recursive = true,
                arg if arg.starts_with("-mode=") => {
                    let mode_str = &arg["-mode=".len()..];
                    mode = match mode_str {
                        "day" => Mode::Day,
                        "month" => Mode::Month,
                        _ => return Err("Invalid mode"),
                    }
                },
                arg if arg.starts_with("-sort") => {
                    let sort_str = &arg["-sort=".len()..];
                    sort_type = match sort_str {
                        "created" => SortType::Created,
                        "modified" => SortType::Modified,
                        _ => return Err("Invalid sort type"),                  
                    }
                }
                _ => return Err("Unknown argument"),
            }
        }

        Ok(Config { directory_path, recursive, mode, sort_type })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    process_directory(&config)?;
    Ok(())
}

fn process_directory(config: &Config) -> Result<(), Box<dyn Error>> {
    let entries = fs::read_dir(&config.directory_path)?;

    for entry in entries {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            if config.recursive {
                process_directory(config)?;
            }
        } else {
            move_file(entry, &config.mode, &config.sort_type)?;
        }
    }

    Ok(())
}


fn move_file(file: DirEntry, mode: &Mode, sort_type: &SortType) -> Result<(), Box<dyn Error>> {
    let original_path = file.path();
    let parent_dir = get_parent_dir(&original_path)
        .ok_or("Error getting the parent directory")?;

    let metadata = file.metadata()?;
    let creation_time = match sort_type {
        SortType::Created => get_creation_time(metadata)?,
        SortType::Modified => get_modification_time(metadata)?,
    };
    let (year, month, day) = get_year_month_day(creation_time);

    let new_dir = match mode {
        Mode::Month => parent_dir.join(year.to_string()).join(month.to_string()),
        Mode::Day => parent_dir.join(year.to_string()).join(month.to_string()).join(day.to_string()),
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

fn get_modification_time(metadata: Metadata) -> Result<SystemTime, MetadataError> {
    metadata.modified().map_err(|e| {
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
    if path.is_file() {
        return Some(std::env::current_dir().ok()?);
    }
    for component in path.components() {
        if let Component::Normal(root_dir) = component {
            return Some(PathBuf::from(root_dir));
        }
    }
    Some(path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempdir::TempDir;

    #[test]
    fn test_move_file_month_created() {
        let temp_dir = TempDir::new("test_dir").expect("Failed to create temp dir");
        let temp_dir_path = temp_dir.path();

        // Create a dummy file
        let file_path = temp_dir_path.join("test_file.txt");
        File::create(&file_path).expect("Failed to create test file");

        // Get the DirEntry for the dummy file
        let dir_entry = fs::read_dir(&temp_dir_path)
            .expect("Failed to read temp dir")
            .next()
            .expect("No file found in temp dir")
            .expect("Failed to get DirEntry");

        // Move the file using the Mode::Month and SortType::Created
        move_file(dir_entry, &Mode::Month, &SortType::Created).expect("Failed to move file");

        // Check if the file has been moved to the expected location
        let year_dir = temp_dir_path.join(Utc::now().year().to_string());
        let month_dir = year_dir.join(Utc::now().month().to_string());
        let moved_file_path = month_dir.join("test_file.txt");

        assert!(moved_file_path.exists());
    }
}