use std::{sync::mpsc::Sender, time::Duration};

use nix::unistd::Pid;

/// A handle to a running process.
#[derive(Debug, Clone)]
pub struct Process {
    /// The process's ID.
    pub pid: Pid,
    /// The name of the executable for the process.
    pub name: String,
}

/// Watches for processes in in `/proc` and forwards them to the sender.
pub fn watch_processes(
    process: Sender<Process>,
    interval: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let interval_duration = Duration::from_millis(interval.unwrap_or(1000) as u64);

    loop {
        let entries = std::fs::read_dir("/proc")?;

        for entry in entries {
            let path = entry?.path();
            let path = path.to_str().unwrap();

            if !path.starts_with("/proc/") {
                continue;
            }

            let path = path.trim_start_matches("/proc/");
            let pid = match path.parse::<i32>() {
                Ok(pid) => pid,
                Err(_) => continue,
            };

            // In the time that it takes to read the directory, the process may have exited.
            // TODO: filter by name before we send through channel?
            if let Ok(name) = std::fs::read_to_string(format!("/proc/{}/comm", pid)) {
                process.send(Process {
                    pid: Pid::from_raw(pid),
                    name: name.trim().to_string(),
                })?;
            }
        }

        std::thread::sleep(interval_duration);
    }
}
