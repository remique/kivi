use glob::glob;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::path::PathBuf;
use std::{collections::BTreeMap, fs::File, fs::OpenOptions};

use crate::core::config::Config;
use crate::core::error::{KiviError, Result};
use log;

#[derive(Debug)]
struct InternalRecord {
    file_id: String,
    value_size: i32,
    value_pos: i32,
}

#[derive(Serialize, Deserialize, Debug)]
enum KiviCommand {
    Set { key: String, value: String },
    Delete { key: String },
}

#[derive(Debug)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

pub struct KiviStore {
    mem_index: BTreeMap<String, InternalRecord>,
    active_file: File,
    stale_files: Vec<PathBuf>,
    config: Config,
}

impl KiviStore {
    pub fn new() -> Result<Self> {
        let config = Config::default();

        let mut mem_index = BTreeMap::new();

        let stale_file_list = data_files_sorted(&config);
        let new_active_file_index = calculate_new_index(&stale_file_list);
        let stale_files = stale_file_list;

        let active_file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(config.new_active_file_path(new_active_file_index))?;

        build_keydir(&stale_files, &mut mem_index);

        Ok(Self {
            mem_index,
            active_file,
            stale_files,
            config,
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
            None => Err(KiviError::Generic(format!("Costam"))),
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

    // TODO: This should also set to keydir
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        log::trace!("SET command key: {}, value: {}", key, value);

        let set = KiviCommand::Set { key, value };
        let j = serde_json::to_string(&set)?;

        // Set to keydir
        // self.mem_index.insert(key, value);
        self.active_file.write_all(j.as_bytes())?;

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

        for (_, record) in &self.mem_index {
            let internal = self.get_internal(record)?;
            let as_str = serde_json::to_string(&internal)?;

            new_file_test.write_all(as_str.as_bytes())?;
        }

        // 1. Delete all log files in db/data/
        data_files_sorted(&self.config).iter().for_each(|f| {
            std::fs::remove_file(f).unwrap();
        });

        // 2. Move new_file_test to db/data directory
        drop(new_file_test);
        std::fs::rename(new_file_test_path, "db/data/1.log").unwrap();
        let new_stale_files = data_files_sorted(&self.config);
        self.stale_files = new_stale_files;
        // 3. Set new_file_test as stale files
        // 4. Create new active_file

        let active_file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(
                self.config
                    .new_active_file_path(calculate_new_index(&self.stale_files)),
            )
            .unwrap();

        std::fs::remove_dir("./db/data/temp").unwrap();

        self.active_file = active_file;

        Ok(())
    }
}

/// Costam jakas definicja
fn calculate_new_index(input: &Vec<std::path::PathBuf>) -> usize {
    input
        .last()
        .and_then(|x| x.file_stem())
        .and_then(|x| x.to_str())
        .and_then(|x| x.parse::<usize>().ok())
        .unwrap()
        + 1
}

fn build_keydir(stale_files: &Vec<PathBuf>, mem_index: &mut BTreeMap<String, InternalRecord>) {
    for file in stale_files {
        let file_d = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(file)
            .unwrap();

        let reader = std::io::BufReader::new(file_d);
        let mut pos: i32 = 0;
        let mut commands = serde_json::Deserializer::from_reader(reader).into_iter::<KiviCommand>();

        while let Some(cos) = commands.next() {
            let new_pos = commands.byte_offset() as i32;

            if let Ok(kivi_command) = cos {
                if let KiviCommand::Set { key, value } = kivi_command {
                    let as_str = file.as_path().display().to_string();

                    let rec = InternalRecord {
                        file_id: as_str,
                        value_size: new_pos - pos,
                        value_pos: pos,
                    };
                    mem_index.insert(key, rec);
                }
            }
            pos = new_pos;
        }
    }

    log::debug!("KeyDir: {:?}", mem_index);
}

fn data_files_sorted(config: &Config) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();

    let paths = glob(config.get_glob_path().as_ref()).unwrap();

    for path in paths {
        if let Ok(item) = path {
            files.push(item);
        }
    }

    files.sort();

    files
}
