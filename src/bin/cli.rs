use clap::{Arg, Command};
use kivi::core::{error::Result, kv::KiviStore};

fn initialize_logger() {
    let env = env_logger::Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);
}

fn main() -> Result<()> {
    initialize_logger();

    let mut ks = match KiviStore::new() {
        Ok(ks) => ks,
        Err(e) => return Err(e),
    };

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
        .subcommand(Command::new("compact").about("Compacts db"))
        .get_matches();

    match m.subcommand() {
        Some(("set", m)) => {
            // We can unwrap here as they are both required
            let key = m.get_one::<String>("KEY").unwrap().to_owned();
            let value = m.get_one::<String>("VALUE").unwrap().to_owned();

            ks.set(key, value)?;
        }
        Some(("get", m)) => {
            let key = m.get_one::<String>("KEY").unwrap().to_owned();

            match ks.get(key) {
                Some(kv) => {
                    println!("Got: {:?}", kv);
                }
                None => {
                    println!("Got nothing");
                }
            }
        }
        Some(("compact", _)) => {
            ks.compact()?;
        }
        _ => {}
    }

    Ok(())
}
