use glob::glob;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::{collections::BTreeMap, fs::OpenOptions};

use clap::{Arg, Command};

type FileId = i32;

const KIVI_DIR: &str = "db";
const DATA_DIR: &str = "data";
const HINTS_DIR: &str = "hints";

struct KiviStore {
    mem_index: BTreeMap<String, InternalRecord>,
    active_file: File,
    // stale_files: Vec<FileId>,
}

#[derive(Serialize, Deserialize)]
struct Record {
    key: String,
    value: String,
}

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

impl KiviStore {
    fn new() -> Self {
        let mut mem_index = BTreeMap::new();

        let active_file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open("1.log")
            .unwrap();

        build_keydir(&active_file, &mut mem_index);

        Self {
            mem_index,
            active_file,
        }
    }

    fn get(&self, key: String) {
        // println!("Doing KiviStore GET. Key: {}", key);
        let get_by_key = self.mem_index.get(&key);
        if let Some(val) = get_by_key {
            println!("KiviStore GET: Got {:?}", val);
        } else {
            println!("KiviStore GET: not found");
        }
    }

    fn set(&mut self, key: String, value: String) {
        println!("Doing KiviStore SET. Key: {}, Value: {}", key, value);

        let set_com = KiviCommand::Set { key, value };

        let j = serde_json::to_string(&set_com).unwrap();

        self.active_file.write_all(j.as_bytes()).unwrap();
        // self.mem_index.insert(key, value);
    }
}

fn build_keydir(active_file: &File, mem_index: &mut BTreeMap<String, InternalRecord>) {
    // first we iterate over all stale files decreasing order
    // and then insert keys and values
    let reader = std::io::BufReader::new(active_file);
    let mut commands = serde_json::Deserializer::from_reader(reader).into_iter::<KiviCommand>();

    while let Some(cos) = commands.next() {
        if let Ok(kivi_command) = cos {
            if let KiviCommand::Set { key, value } = kivi_command {
                let rec = InternalRecord {
                    file_id: "1.log".to_string(),
                    value_size: value.len() as i32,
                    // TODO: write proper  pos, so need to remember buffer position
                    value_pos: 0,
                };
                mem_index.insert(key, rec);
            }
        }
    }
}

fn get_files() {
    let xd = glob(format!("./{}/{}/*.data", KIVI_DIR, DATA_DIR).as_ref()).unwrap();

    for item in xd {
        println!("{:?}", item);
    }
}

fn main() {
    let mut ks = KiviStore::new();

    let m = Command::new("kivi")
        .subcommand(
            Command::new("set")
                .args([
                    Arg::new("KEY").required(true),
                    Arg::new("VALUE").required(true),
                ])
                .about("Sets a value to a key"),
        )
        .subcommand(
            Command::new("get")
                .arg(Arg::new("KEY").required(true))
                .about("Gets a value by key"),
        )
        .get_matches();

    match m.subcommand() {
        Some(("set", m)) => {
            // We can unwrap here as they are both required
            let key = m.get_one::<String>("KEY").unwrap().to_owned();
            let value = m.get_one::<String>("VALUE").unwrap().to_owned();

            ks.set(key, value);
        }
        Some(("get", m)) => {
            let key = m.get_one::<String>("KEY").unwrap().to_owned();

            ks.get(key);
        }
        _ => {}
    }
}
