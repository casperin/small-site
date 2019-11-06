use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub struct FileReader {
    cache: HashMap<String, Option<String>>,
}

impl FileReader {
    pub fn new() -> FileReader {
        FileReader {
            cache: HashMap::new(),
        }
    }

    pub fn get<'a>(&'a mut self, p: PathBuf) -> &'a Option<String> {
        match p.to_str() {
            None => &None,
            Some(s) => self
                .cache
                .entry(s.to_string())
                .or_insert_with(|| fs::read_to_string(&p).ok()),
        }
    }
}
