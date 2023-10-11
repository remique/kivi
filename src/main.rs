use std::collections::BTreeMap;

use clap::{Arg, Command};

struct KiviStore {
    mem_index: BTreeMap<String, String>,
}

impl KiviStore {
    fn new() -> Self {
        let mem_index = BTreeMap::new();

        Self { mem_index }
    }

    fn get(&self, key: String) {
        println!("Doing KiviStore GET. Key: {}", key);
    }

    fn set(&self, key: String, value: String) {
        println!("Doing KiviStore SET. Key: {}, Value: {}", key, value);
    }
}

fn main() {
    let ks = KiviStore::new();

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
