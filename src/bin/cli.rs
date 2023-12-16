use procpin::{affinity, ccx, config::*, watch};

fn main() {
    if let Err(e) = start() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn start() -> Result<(), Box<dyn std::error::Error>> {
    let ccxes = ccx::find_ccxes();
    let config = Config::load();

    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        if let Err(err) = watch::watch_processes(tx, config.interval) {
            eprintln!("failed to watch processes: {}", err);
        }
    });

    loop {
        let process = rx.recv()?;

        if let Some(ccx_preference) = config.preference(&process.name) {
            if affinity::set_affinity(&process, ccx_preference, &ccxes)? {
                println!("Set affinity of `{}`", process.name);
            }
        }
    }
}
