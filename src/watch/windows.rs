use std::{ffi::OsString, mem, os::windows::ffi::OsStringExt, sync::mpsc::Sender, time::Duration};

use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};

/// A handle to a running process.
#[derive(Debug, Clone)]
pub struct Process {
    /// The process's ID.
    pub pid: u32,
    /// The name of the executable for the process.
    pub name: String,
}

/// Watches for processes in in `/proc` and forwards them to the sender.
pub fn watch_processes(
    processes: Sender<Process>,
    interval: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let interval_duration = Duration::from_millis(interval.unwrap_or(1000) as u64);

    loop {
        unsafe {
            // Create a snapshot of the running processes.
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
            let mut process = mem::zeroed::<PROCESSENTRY32W>();
            process.dwSize = mem::size_of_val(&process) as _;

            // Populate the first process.
            Process32FirstW(snapshot, &mut process as _)?;

            loop {
                let name = OsString::from_wide(&process.szExeFile);

                // Some processes might not be valid UTF-8 but I don't care.
                if let Some(name) = name.to_str() {
                    let name = name.trim_end_matches('\0');
                    let name = name.trim_end_matches(".exe");
                    processes.send(Process {
                        pid: process.th32ProcessID,
                        name: name.into(),
                    })?;
                }

                // Attempt iterating until we get an error.
                if Process32NextW(snapshot, &mut process as _).is_err() {
                    break;
                }
            }
        }

        std::thread::sleep(interval_duration);
    }
}
