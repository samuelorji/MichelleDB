use std::collections::HashMap;
use std::ffi::OsString;
use std::io::ErrorKind;
use std::iter::Filter;
use std::path::{Path, PathBuf};
use serde_json::Map;
use serde::de::Unexpected::Str;
use serde_json::Value;
use walkdir::{IntoIter, WalkDir};

use tokio::io;

pub struct DB {
    pub dbDir: String,
    pub indexDir: String,
}

pub type Document = Map<String, Value>;

impl DB {
    pub fn new() -> io::Result<DB> {
        let relative_path = ".michelleDB";
        let dbDir = match std::env::home_dir() {
            Some(path) => format!("{}", path.join(relative_path).display()),
            None => String::from(relative_path),
        };

        let dir = Path::new(&dbDir);
        let indexDir = format!("{}.index", &dbDir);
        let indexDir = Path::new(&indexDir);

        if (dir.exists() && indexDir.exists()) {
            Ok(DB {
                indexDir: indexDir.to_string_lossy().to_string(),
                dbDir,
            })
        } else {
            std::fs::create_dir(&dbDir)?;
            std::fs::create_dir(&indexDir.to_string_lossy().to_string())?;
            Ok(DB {
                indexDir: indexDir.to_string_lossy().to_string(),
                dbDir,
            })
        }
    }

    pub fn getById(&self, id: &str) -> Result<Option<Document>, String> {
        match std::fs::read(Path::new(&self.dbDir).join(&id)) {
            Ok(contents) => unsafe {
                Ok(String::from_utf8_unchecked(contents))
                    .and_then(|e| serde_json::from_str::<Document>(&e).map(|doc| Some(doc))
                        .map_err(|e| e.to_string()))
            },
            Err(e) => {
                //Err(e.to_string())
                // Err(e) => Err(e.to_string()),
                if (e.kind() == ErrorKind::NotFound) {
                    Ok(None)
                } else {
                    Err(e.to_string())
                }
            }
        }
    }

    pub fn documents(&self) -> WalkDir {
        WalkDir::new(&self.dbDir)
    }

    pub fn get_indexed_document(&self, pathValue: &str) -> Result<String, String> {
        match std::fs::read(Path::new(&self.indexDir).join(DB::encode(pathValue))) {
            Ok(contents) => unsafe { Ok(String::from_utf8_unchecked(contents)) }
            Err(e) => {
                if (e.kind() == ErrorKind::NotFound) {
                    Ok(String::new())
                } else {
                    Err(e.to_string())
                }
            }
        }
    }

    fn encode(id: &str) -> String {
        let mut m = sha1_smol::Sha1::new();
        m.update(id.as_bytes());
        m.digest().to_string()
    }
    fn write_to_path(path: PathBuf, content: String) -> io::Result<()> {
        std::fs::write(path, content)
    }
    pub fn write_document(&self, id: &String, content: String) -> io::Result<()> {
        DB::write_to_path(Path::new(&self.dbDir).join(&id), content)
    }
    pub fn write_to_index(&self, id: &String, content: String) -> io::Result<()> {
        DB::write_to_path(Path::new(&self.indexDir).join(DB::encode(id)), content)
    }

    pub fn index_lookup(&self, pathValue: &str) -> Result<Vec<String>, String> {
        std::fs::read(Path::new(&self.indexDir).join(DB::encode(pathValue))).map_err(|e| e.to_string())
            .and_then(|bytes| {
                std::str::from_utf8(&bytes)
                    .map_err(|e| e.to_string())
                    .map(|fileContents| fileContents.split(",").map(|e| e.to_string()).collect::<Vec<String>>())
            })
    }
}
