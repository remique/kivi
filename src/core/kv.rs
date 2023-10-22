use glob::glob;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::{collections::BTreeMap, fs::File, fs::OpenOptions};

use crate::core::config::Config;
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

pub struct KiviStore {
    mem_index: BTreeMap<String, InternalRecord>,
    active_file: File,
    stale_files: Vec<File>,
    config: Config,
}

impl KiviStore {
    pub fn new() -> Self {
        let config = Config::default();

        let mut mem_index = BTreeMap::new();

        // get read-only stale files
        let stale_file_list = get_files(&config);
        let new_active_file_index = stale_file_list
            .last()
            .and_then(|x| x.file_stem())
            .and_then(|x| x.to_str())
            .and_then(|x| x.parse::<usize>().ok())
            .unwrap()
            + 1;

        let mut stale_files = Vec::new();

        for item in stale_file_list {
            let file_d = OpenOptions::new()
                .create(true)
                .append(true)
                .read(true)
                .open(item)
                .unwrap();

            stale_files.push(file_d);
        }

        let active_file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(config.new_active_file_path(new_active_file_index))
            .unwrap();

        log::debug!(
            "New active file path: {}",
            config.new_active_file_path(new_active_file_index)
        );

        build_keydir(&active_file, &mut mem_index);

        log::debug!("Stale files: {:?}", stale_files);

        Self {
            mem_index,
            active_file,
            stale_files,
            config,
        }
    }

    pub fn get(&self, key: String) {
        let get_by_key = self.mem_index.get(&key);
        if let Some(val) = get_by_key {
            println!("KiviStore GET: Got {:?}", val);
        } else {
            println!("KiviStore GET: not found");
        }
    }

    pub fn set(&mut self, key: String, value: String) {
        println!("Doing KiviStore SET. Key: {}, Value: {}", key, value);

        let set_com = KiviCommand::Set { key, value };

        let j = serde_json::to_string(&set_com).unwrap();

        self.active_file.write_all(j.as_bytes()).unwrap();
    }

    fn compact(&mut self) {
        unimplemented!();
    }
}

fn build_keydir(active_file: &File, mem_index: &mut BTreeMap<String, InternalRecord>) {
    let reader = std::io::BufReader::new(active_file);
    let mut pos: i32 = 0;
    let mut commands = serde_json::Deserializer::from_reader(reader).into_iter::<KiviCommand>();

    while let Some(cos) = commands.next() {
        let new_pos = commands.byte_offset() as i32;

        if let Ok(kivi_command) = cos {
            if let KiviCommand::Set { key, value } = kivi_command {
                let rec = InternalRecord {
                    file_id: "1.log".to_string(),
                    value_size: value.len() as i32,
                    value_pos: pos,
                };
                mem_index.insert(key, rec);
            }
        }
        pos = new_pos;
    }

    log::debug!("Successfully built keydir");
}

fn get_files(config: &Config) -> Vec<std::path::PathBuf> {
    let mut stale_file_list = Vec::new();

    let paths = glob(config.get_glob_path().as_ref()).unwrap();

    for path in paths {
        if let Ok(item) = path {
            stale_file_list.push(item);
        }
    }

    stale_file_list.sort();

    stale_file_list
}
