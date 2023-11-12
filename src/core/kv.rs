use glob::glob;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::os::fd::{AsFd, AsRawFd};
use std::path::PathBuf;
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
    stale_files: Vec<PathBuf>,
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
            stale_files.push(item);
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

        build_keydir(&stale_files, &mut mem_index);

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

    pub fn compact(&mut self) {
        let new_file_test_path = format!("./db/data/temp/{}", "costam.test");
        std::fs::create_dir_all("./db/data/temp").unwrap();
        let mut new_file_test = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(new_file_test_path.clone()) // TODO: change str
            .expect("openoptions fails");

        for (_key, record) in &self.mem_index {
            let mut file_d = OpenOptions::new()
                .read(true)
                .open(format!("./{}", record.file_id.as_str()))
                .expect("openoptions fails");

            let mut esti = String::new();
            let foo = file_d.read_to_string(&mut esti);

            let new_str = esti
                .get(
                    record.value_pos as usize
                        ..record.value_pos as usize + record.value_size as usize,
                )
                .unwrap();

            let xd: KiviCommand = serde_json::from_str(new_str).unwrap();

            log::info!(
                "Reading from {}... Buffer: {:?}, bufer_2: {}",
                record.file_id.as_str(),
                xd,
                new_str
            );

            new_file_test.write_all(new_str.as_bytes()).unwrap();
        }

        // 1. Delete all log files in db/data/
        let files_to_delete = get_files(&self.config);
        for item in files_to_delete {
            std::fs::remove_file(item).unwrap(); // TODO: proper error handlin
        }
        // 2. Move new_file_test to db/data directory
        drop(new_file_test);
        std::fs::rename(new_file_test_path, "db/data/1.log").unwrap();
        let new_stale_files = get_files(&self.config);
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
    }
}

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

    log::debug!("Successfully built keydir");
    log::debug!("KeyDir: {:?}", mem_index);
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
