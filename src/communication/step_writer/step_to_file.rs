use std::fs;
use std::path::Path;
use std::fs::File;
use std::io::{Write, Error, ErrorKind};
use std::fs::OpenOptions;

pub struct StepWriter {
    step_file: File
}

impl StepWriter {
    pub fn new(path: &Path) -> Option<StepWriter> {
        if let Err(e) = StepWriter::create_parent_dir(path) {
            println!("Error creating parent dir: {e}");
            return None;
        }

        let file = OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(path);

        return match file {
            Ok(file) => {
                Some(StepWriter {
                    step_file: file,
                })
            },
            Err(e) => {
                println!("Error creating file: {e}");
                None
            }
        }
    }

    fn create_parent_dir(path: &Path) -> std::io::Result<()> {
        if path.is_file() {
            return Ok(());
        }

        let parent = path.parent();
        return match parent {
            Some(path) => {
                if path.exists() {
                    return Ok(())
                }
                if path != Path::new("") {
                    fs::create_dir(path)?;
                }
                Ok(())
            },
            None => Err(Error::new(ErrorKind::InvalidInput, "Could not create parent directories")),
        };
    }
    fn write_into_file(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.step_file.write_all(data)?;
        Ok(())
    }

    pub fn write_step_to_file(&mut self, step: &str) {
        match self.write_into_file(step.as_bytes()) {
            Ok(_) => {},
            Err(err) => println!("Error writing to file: {}", err),
        };
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::fs::File;
    use std::fs;
    use rocket::mtls::oid::asn1_rs::nom::AsBytes;
    use crate::communication::step_writer::step_to_file::StepWriter;

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
        if let Some(mut s_writer) = StepWriter::new(path) {
            s_writer.write_step_to_file(data);

            if let Some(result) = was_written_correctly(Path::new(file_name)) {
                fs::remove_file(path).expect("The file should have been created");
                assert_eq!(result, data);
                return;
            }
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
        const DIR: &str = "logs1";
        let file_name = format!("{DIR}/log_file.txt");
        const DATA: &str = "random data 123";

        assert_file_creation(file_name.as_str(), DATA);
        fs::remove_dir(DIR).expect("Directory should exist");
    }

    #[test]
    fn parent_dir_already_exists_test() {
        const DIR: &str = "logs2";
        let file_name_first = format!("{DIR}/log_file.txt");
        let file_name_second= format!("{DIR}/log_file2.txt");
        const DATA: &str = "random data 123";

        assert_file_creation(file_name_first.as_str(), DATA);
        assert_file_creation(file_name_second.as_str(), DATA);

        fs::remove_dir(DIR).expect("Directory should exist");
    }

    #[test]
    fn file_already_exists() {
        const DIR: &str = "logs3";
        let file_name = format!("{DIR}/log_file.txt");

        if let Some(_) = StepWriter::new(Path::new(file_name.as_str())) {
            if let Some(_) = StepWriter::new(Path::new(file_name.as_str())) {
                assert!(false);
            }
        }

        fs::remove_file(file_name).expect("log_file.txt should exist");
        fs::remove_dir(DIR).expect("Directory either does not exist or is full");
    }

    #[test]
    fn append_to_file() {
        const FILE_NAME: &str = "file_to_append_to.txt";
        const DATA_FIRST: &str = "first\n";
        const DATA_SECOND: &str = "second";
        let path = Path::new(FILE_NAME);
        if let Some(mut s_writer) = StepWriter::new(path) {
            s_writer.write_into_file(DATA_FIRST.as_bytes())
                .expect("Should be able to write to a file");
            s_writer.write_into_file(DATA_SECOND.as_bytes())
                .expect("Should be able to write to a file");

            if let Some(result) = was_written_correctly(Path::new(FILE_NAME)) {
                fs::remove_file(path).expect("The file should have been created");
                assert_eq!(result,
                           String::from(DATA_FIRST) + DATA_SECOND);
                return;
            }
        }

        assert!(false);
    }
}
