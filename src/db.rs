use std::ffi::OsString;
use std::path::Path;
use serde::de::Unexpected::Str;

use tokio::io;

pub struct DB {
   pub dbDir : String,
    indexDir: String
}

impl DB {
    pub fn new() -> io::Result<DB> {

        let relative_path = ".michelleDB";
        let dbDir =  match std::env::home_dir() {
            Some(path) => format!("{}",path.join(relative_path).display()),
            None => String::from(relative_path),
        };

        let dir= Path::new(&dbDir);
        let indexDir = format!("{}.index", &dbDir);
        if(dir.exists()) {
            Ok(DB {
                indexDir,
                dbDir
            })
        } else {
            std::fs::create_dir(&dbDir)?;
            Ok(DB {
                indexDir,
                dbDir
            })
        }
    }

    pub fn getById(&self, id : String) -> Option<String> {
        match std::fs::read(Path::new(&self.dbDir).join(&id)) {
            Ok(contents) => unsafe { Some(String::from_utf8_unchecked(contents)) },
            Err(e) => None,
        }
    }

}
