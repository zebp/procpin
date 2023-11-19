#[cfg(target_os = "linux")]
use nix::sched::CpuSet;

use crate::ccx::Ccx;

use serde::Deserialize;
use std::collections::HashMap;

/// The configuration of the program.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    // The number os milliseconds to wait between each check for new processes
    pub interval: Option<usize>,
    // The list of processes to watch and their CCX preferences.
    pub programs: HashMap<String, CcxPreference>,
}

impl Config {
    /// Loads the [Config] from `/etc/procpin.toml`.
    pub fn load() -> Self {
        let config = std::fs::read_to_string("./procpin.toml").unwrap();
        toml::from_str(&config).unwrap()
    }

    /// Get the CCX preference for a given program.
    pub fn preference(&self, name: &str) -> Option<&CcxPreference> {
        self.programs.get(name)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum CcxPreference {
    Singlular(usize),
    Multiple(Vec<usize>),
}

impl CcxPreference {
    /// Construct a [CpuSet] from the CCX preference.
    #[cfg(target_os = "linux")]
    pub fn cpuset(&self, ccxes: &[Ccx]) -> nix::Result<CpuSet> {
        let mut cpuset = CpuSet::default();

        match self {
            CcxPreference::Singlular(idx) => {
                for cpu in &ccxes[*idx].cpus {
                    cpuset.set(cpu.logical_core)?;
                }
            }
            CcxPreference::Multiple(indicies) => {
                for idx in indicies {
                    for cpu in &ccxes[*idx].cpus {
                        cpuset.set(cpu.logical_core)?;
                    }
                }
            }
        }

        Ok(cpuset)
    }

    /// Construct an afinity from the CCX preference.
    #[cfg(windows)]
    pub fn affinity(&self, ccxes: &[Ccx]) -> usize {
        let mut mask = 0;

        match self {
            CcxPreference::Singlular(idx) => {
                mask |= ccxes[*idx].affinity_mask();
            }
            CcxPreference::Multiple(indicies) => {
                for idx in indicies {
                    mask |= ccxes[*idx].affinity_mask();
                }
            }
        }

        mask
    }
}
