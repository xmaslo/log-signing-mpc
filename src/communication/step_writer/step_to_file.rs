use std::fs;
use std::path::Path;
use std::fs::File;
use std::io::{Write, Error, ErrorKind};

fn write_into_file(path: &Path, data: &[u8]) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(data)?;
    Ok(())
}

fn create_parent_dirs(path: &Path) -> std::io::Result<()> {
    if path.is_file() {
        return Ok(());
    }

    let parent = path.parent();
    return match parent {
        Some(path) => {
            if path != Path::new("") {
                fs::create_dir(path)?;
            }
            Ok(())
        },
        None => Err(Error::new(ErrorKind::InvalidInput, "Could not create parent directories")),
    };
}

pub fn write_step_to_file(path: &Path, step: &str) {
    match create_parent_dirs(path) {
        Ok(_) => {},
        Err(err) => {
            println!("Error creating parent dirs: {}", err);
            return;
        },
    };

    match write_into_file(&path, step.as_bytes()) {
        Ok(_) => {},
        Err(err) => println!("Error writing to file: {}", err),
    };
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::fs::File;
    use std::fs;
    use crate::communication::step_writer::step_to_file::write_step_to_file;

    fn was_written_correctly(file_name: &Path) -> Option<String> {
        return if let Ok(_) = File::open(file_name) {
            let content = fs::read_to_string(file_name);
            match content {
                Ok(c) => Some(c),
                Err(_) => Some(String::new()),
            }
        } else {
            None
        }
    }

    fn assert_file_creation(file_name: &str, data: &str) {
        let path = Path::new(file_name);
        write_step_to_file(path, data);

        if let Some(result) = was_written_correctly(Path::new(file_name)) {
            fs::remove_file(path).expect("The file should have been created");
            assert_eq!(result, data);
            return;
        }

        assert!(false);
    }

    #[test]
    fn file_without_dirs_test() {
        const FILE_NAME: &str = "log_file.txt";
        const DATA: &str = "random data 123";

        assert_file_creation(FILE_NAME, DATA);
    }

    #[test]
    fn file_with_dirs_test() {
        const FILE_NAME: &str = "logs/log_file.txt";
        const DATA: &str = "random data 123";

        assert_file_creation(FILE_NAME, DATA);
        fs::remove_dir("logs").expect("logs directory should exist");
    }
}
