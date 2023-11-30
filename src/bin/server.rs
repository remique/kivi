use std::net::SocketAddr;

use kivi::server;

fn initialize_logger() {
    let env = env_logger::Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);
}

fn main() {
    initialize_logger();

    let addr = SocketAddr::from(([0, 0, 0, 0], 7878));

    let mut s = server::KiviServer::new().unwrap();

    log::info!("Server listening at {:?}", addr);

    s.run(addr).unwrap();
}
