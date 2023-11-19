use std::fs;

use crate::topology::CacheGroup;

use super::Cpu;

/// Read the CPU topology from `/sys/devices/system/cpu`.
pub fn cpus() -> impl Iterator<Item = Cpu> {
    fs::read_dir("/sys/devices/system/cpu/")
        .expect("failed to read /sys/devices/system/cpu/")
        .into_iter()
        .map(|entry| entry.expect("failed to read entry in /sys/devices/system/cpu/"))
        .filter_map(|entry| {
            let path = entry.path();
            let path = path.to_str()?;

            if !path.starts_with("/sys/devices/system/cpu/cpu") {
                return None;
            }

            let path = path.trim_start_matches("/sys/devices/system/cpu/cpu");
            path.parse::<usize>().ok()
        })
        .map(|logical_core| {
            read_cpu(logical_core)
                .unwrap_or_else(|err| panic!("failed to read cpu {logical_core}: {err}"))
        })
}

macro_rules! read_num {
    ($logical_core:expr, $path:expr) => {
        std::fs::read_to_string(format!(
            "/sys/devices/system/cpu/cpu{}/{}",
            $logical_core, $path
        ))?
        .trim()
        .parse()
    };
}

fn read_cpu(logical_core: usize) -> Result<Cpu, Box<dyn std::error::Error>> {
    Ok(Cpu {
        physical_core: read_num!(logical_core, "topology/core_id")?,
        logical_core,
        l3_group: CacheGroup {
            id: read_num!(logical_core, "cache/index3/id")?,
            size: 0,
        },
    })
}
