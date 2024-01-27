use glob::glob;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::io::prelude::*;
use std::path::PathBuf;
use std::{collections::BTreeMap, fs::File, fs::OpenOptions};

use crate::core::{
    config::Config,
    error::{KiviError, Result},
};
use log;

pub struct KiviStore {
    mem_index: BTreeMap<String, InternalRecord>,
    active_file: File,
    stale_files: Vec<PathBuf>,
    config: Config,
}

#[derive(Debug)]
struct InternalRecord {
    file_id: String,
    value_size: i32,
    value_pos: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum KiviCommand {
    Set { key: String, value: String },
    Delete { key: String },
}

#[derive(Debug, PartialEq)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

impl Engine for KiviStore {}

impl KiviStore {
    fn create_directories(config: &Config) -> Result<()> {
        std::fs::create_dir_all(config.get_full_path())?;
        log::trace!(
            "Checking and creting directories at path: {}",
            config.get_full_path()
        );

        Ok(())
    }

    fn initialize(config: Config) -> Result<Self> {
        // Create directories if they dont exist
        Self::create_directories(&config)?;

        let stale_file_list = data_files_sorted(&config)?;
        let new_active_file_index = last_file_index(&stale_file_list) + 1;
        let stale_files = stale_file_list;

        log::info!("Current active file index: {}", new_active_file_index);

        let active_file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(config.new_active_file_path(new_active_file_index))?;

        let mem_index = build_index(&stale_files)?;

        Ok(Self {
            mem_index,
            active_file,
            stale_files,
            config,
        })
    }

    pub fn new() -> Result<Self> {
        let res = Self::initialize(Config::default())?;

        Ok(Self {
            mem_index: res.mem_index,
            active_file: res.active_file,
            stale_files: res.stale_files,
            config: res.config,
        })
    }

    pub fn with_config(config: Config) -> Result<Self> {
        let res = Self::initialize(config)?;

        Ok(Self {
            mem_index: res.mem_index,
            active_file: res.active_file,
            stale_files: res.stale_files,
            config: res.config,
        })
    }

    fn get_internal(&self, record: &InternalRecord) -> Result<KiviCommand> {
        // Read from file
        let mut file = OpenOptions::new()
            .read(true)
            .open(record.file_id.as_str())?;

        // We use String as a buffer
        let mut s = String::new();
        file.read_to_string(&mut s)?;

        let get = s
            .get(record.value_pos as usize..record.value_pos as usize + record.value_size as usize);

        match get {
            Some(x) => Ok(serde_json::from_str(x)?),
            None => Err(KiviError::Generic("Internal failed".to_string())),
        }
    }

    pub fn get(&self, key: String) -> Option<KeyValue> {
        log::trace!("GET command key: {}", key);

        match self.mem_index.get(&key) {
            Some(i) => match self.get_internal(i) {
                Ok(kv) => {
                    if let KiviCommand::Set { key, value } = kv {
                        Some(KeyValue { key, value })
                    } else {
                        None
                    }
                }
                Err(_) => None,
            },
            None => None,
        }
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        log::trace!("SET command key: {}, value: {}", key, value);

        let set = KiviCommand::Set {
            key: key.clone(),
            value,
        };
        let j = serde_json::to_string(&set)?;

        self.active_file.write_all(j.as_bytes())?;

        // TODO: cleaner
        let path = self
            .config
            .new_active_file_path(last_file_index(&self.stale_files) + 1);

        let rec = InternalRecord {
            file_id: path,
            value_size: j.len() as i32,
            value_pos: self.active_file.metadata().unwrap().len() as i32 - j.len() as i32,
        };

        log::info!("InternalRecord: {:?}", rec);
        self.mem_index.insert(key, rec);

        Ok(())
    }

    pub fn delete(&mut self, _key: String) {
        unimplemented!();
    }

    // TODO: Can simplify this shit
    pub fn compact(&mut self) -> Result<()> {
        let new_file_test_path = format!("./db/data/temp/{}", "costam.test");
        std::fs::create_dir_all("./db/data/temp").unwrap();

        let mut new_file_test = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(new_file_test_path.clone()) // TODO: change str
            .expect("openoptions fails");

        for record in self.mem_index.values() {
            let internal = self.get_internal(record)?;
            let as_str = serde_json::to_string(&internal)?;

            new_file_test.write_all(as_str.as_bytes())?;
        }

        // 1. Delete all log files in db/data/
        data_files_sorted(&self.config)?.iter().for_each(|f| {
            std::fs::remove_file(f).unwrap();
        });

        // 2. Move new_file_test to db/data directory
        drop(new_file_test);
        std::fs::rename(new_file_test_path, "db/data/1.log").unwrap();
        let new_stale_files = data_files_sorted(&self.config)?;
        self.stale_files = new_stale_files;
        // 3. Set new_file_test as stale files
        // 4. Create new active_file

        let active_file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(
                self.config
                    .new_active_file_path(last_file_index(&self.stale_files) + 1),
            )
            .unwrap();

        std::fs::remove_dir("./db/data/temp").unwrap();

        self.active_file = active_file;

        Ok(())
    }
}

fn last_file_index(input: &[PathBuf]) -> usize {
    let res = input
        .last()
        .and_then(|x| x.file_stem())
        .and_then(|x| x.to_str())
        .and_then(|x| x.parse::<usize>().ok());

    match res {
        Some(value) => value,
        None => 0_usize,
    }
}

fn build_index(stales: &[PathBuf]) -> Result<BTreeMap<String, InternalRecord>> {
    let mut index = BTreeMap::new();

    for file in stales {
        let file_d = OpenOptions::new().read(true).open(file)?;

        let reader = std::io::BufReader::new(file_d);

        let mut pos: i32 = 0;

        let mut comms = Deserializer::from_reader(reader).into_iter::<KiviCommand>();

        while let Some(command) = comms.next() {
            let new_pos = comms.byte_offset() as i32;

            match command {
                Ok(c) => {
                    if let KiviCommand::Set { key, value: _ } = c {
                        let as_str = file.as_path().display().to_string();

                        let rec = InternalRecord {
                            file_id: as_str,
                            value_size: new_pos - pos,
                            value_pos: pos,
                        };
                        index.insert(key, rec);
                    }
                }
                Err(e) => {
                    return Err(KiviError::Generic(e.to_string()));
                }
            }
            pos = new_pos;
        }
    }

    // log::debug!("KeyDir: {:?}", index);

    Ok(index)
}

fn data_files_sorted(config: &Config) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();

    let paths = glob(config.get_glob_pattern().as_ref())?;

    for path in paths {
        match path {
            Ok(file) => files.push(file),
            Err(_) => {
                return Err(KiviError::Generic("a".to_string()));
            }
        }
    }

    files.sort();

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempdir::TempDir;

    #[test]
    fn test_last_file_index() {
        let first = PathBuf::from("./1.log");
        let second = PathBuf::from("./2.log");
        let third = PathBuf::from("./3.log");

        let zero: Vec<PathBuf> = vec![];
        let only_first = vec![first.clone()];
        let first_and_third = vec![first.clone(), third.clone()];
        let all = vec![first, second, third];

        assert_eq!(last_file_index(&zero), 0_usize);
        assert_eq!(last_file_index(&only_first), 1_usize);
        assert_eq!(last_file_index(&first_and_third), 3_usize);
        assert_eq!(last_file_index(&all), 3_usize);
    }

    #[test]
    fn test_last_file_index_fuzzed() {
        let first = PathBuf::from("./aaa.log");
        let second = PathBuf::from("./bbb.log");

        let all = vec![first, second];

        assert_eq!(last_file_index(&all), 0_usize);
    }

    #[test]
    fn test_creating() {
        let tempdir = TempDir::new("creating").unwrap();

        let kv = KiviStore::with_config(
            Config::new()
                .set_db_path(tempdir.path().to_path_buf())
                .build(),
        );

        assert!(kv.is_ok());
    }

    #[test]
    fn test_empty_get() {
        let tempdir = TempDir::new("empty_get").unwrap();

        let kv = KiviStore::with_config(
            Config::new()
                .set_db_path(tempdir.path().to_path_buf())
                .build(),
        )
        .unwrap();

        assert_eq!(kv.get("a".to_string()), None)
    }

    #[test]
    fn test_single_set() {
        let tempdir = TempDir::new("single_set").unwrap();

        let mut kv = KiviStore::with_config(
            Config::new()
                .set_db_path(tempdir.path().to_path_buf())
                .build(),
        )
        .unwrap();

        let set = kv.set("a".to_string(), "b".to_string());
        assert!(set.is_ok());

        assert_eq!(
            kv.get("a".to_string()),
            Some(KeyValue {
                key: "a".to_string(),
                value: "b".to_string()
            })
        );
        assert_eq!(kv.get("c".to_string()), None);
    }

    #[test]
    fn test_multiple_set() {
        let tempdir = TempDir::new("multiple_set").unwrap();

        let mut kv = KiviStore::with_config(
            Config::new()
                .set_db_path(tempdir.path().to_path_buf())
                .build(),
        )
        .unwrap();

        let set1 = kv.set("a".to_string(), "b".to_string());
        let set2 = kv.set("c".to_string(), "d".to_string());
        let set3 = kv.set("e".to_string(), "f".to_string());
        assert!(set1.is_ok());
        assert!(set2.is_ok());
        assert!(set3.is_ok());

        assert_eq!(
            kv.get("a".to_string()),
            Some(KeyValue {
                key: "a".to_string(),
                value: "b".to_string()
            })
        );
        assert_eq!(
            kv.get("c".to_string()),
            Some(KeyValue {
                key: "c".to_string(),
                value: "d".to_string()
            })
        );
        assert_eq!(
            kv.get("e".to_string()),
            Some(KeyValue {
                key: "e".to_string(),
                value: "f".to_string()
            })
        );
        assert_eq!(kv.get("g".to_string()), None);
    }

    #[test]
    fn test_file_indexing_works() {
        let tempdir = TempDir::new("file_indexing").unwrap();

        let config = Config::new()
            .set_db_path(tempdir.path().to_path_buf())
            .build();

        // Create directories
        std::fs::create_dir_all(config.get_full_path()).unwrap();

        // Create file with data extension
        let file_path =
            Path::new(&config.get_full_path()).join(format!("1.{}", config.get_data_extension()));

        assert!(!&file_path.exists());

        let _ = File::create(&file_path).unwrap();

        assert!(&file_path.exists());

        let new_file_path =
            Path::new(&config.get_full_path()).join(format!("2.{}", config.get_data_extension()));

        let _ = KiviStore::with_config(config).unwrap();

        assert!(new_file_path.exists());
    }

    #[test]
    fn after_drop_works() {
        let tempdir = TempDir::new("after_drop").unwrap();

        let mut kv1 = KiviStore::with_config(
            Config::new()
                .set_db_path(tempdir.path().to_path_buf())
                .build(),
        )
        .unwrap();

        let set = kv1.set("a".to_string(), "b".to_string());
        assert!(set.is_ok());

        drop(kv1);

        let kv2 = KiviStore::with_config(
            Config::new()
                .set_db_path(tempdir.path().to_path_buf())
                .build(),
        )
        .unwrap();

        assert_eq!(
            kv2.get("a".to_string()),
            Some(KeyValue {
                key: "a".to_string(),
                value: "b".to_string()
            })
        );
        assert_eq!(kv2.get("c".to_string()), None);
    }

    #[test]
    fn test_bad_inside_files_fail() {
        // What if i write some corrupted file 1.log?
    }
}
