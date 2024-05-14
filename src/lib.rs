use std::error::Error;

pub enum SortingType {
    Month, Day
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

