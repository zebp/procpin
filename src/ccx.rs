use std::collections::HashMap;

#[cfg(target_os = "linux")]
use std::iter::FromIterator;

#[cfg(target_os = "linux")]
use nix::sched::CpuSet;

use crate::topology::{self, Cpu};

/// A core complex with a shared L3 cache.
#[derive(Debug, Clone)]
pub struct Ccx {
    pub cpus: Vec<Cpu>,
}

impl Ccx {
    /// Computes the id of the CCD that contains the CCX.
    #[cfg(target_os = "linux")]
    pub fn ccd_id(&self) -> usize {
        let cores_in_ccx = self.cpus.len();
        let highest_core_id = self
            .cpus
            .iter()
            .map(|cpu| cpu.physical_core)
            .max()
            .expect("no cores in ccx");
        highest_core_id / cores_in_ccx
    }

    /// Returns a mask that can be used to set the process affinity to run on these cores.
    #[cfg(windows)]
    pub fn affinity_mask(&self) -> usize {
        let mut mask = 0;

        for cpu in &self.cpus {
            mask |= cpu.affinity_mask;
        }

        mask
    }
}

/// Gets the CCXes of the user's processor based on the L3 in the [LsCpuData].
///
/// Currently in AMD Ryzen cpu core complexes (abbreviated CCX) are grouped based on their shared
/// L3 cache. In Zen 1 through Zen 2 designs this was a dual 4-core CCXes CCD and a single 8-core
/// CCD for designs past Zen 3.
pub fn find_ccxes() -> Vec<Ccx> {
    topology::cpus()
        .fold(HashMap::<usize, Vec<_>>::new(), |mut cores_l3, cpu| {
            cores_l3.entry(cpu.l3_group.id).or_default().push(cpu);
            cores_l3
        })
        .into_iter()
        .map(|(_, cpus)| Ccx { cpus })
        .collect()
}

#[cfg(target_os = "linux")]
impl<'ccx> FromIterator<&'ccx Ccx> for CpuSet {
    fn from_iter<T: IntoIterator<Item = &'ccx Ccx>>(iter: T) -> Self {
        let mut cpu_set = CpuSet::default();

        for ccx in iter {
            for cpu in &ccx.cpus {
                cpu_set
                    .set(cpu.logical_core)
                    .expect("logical core out of range");
            }
        }

        cpu_set
    }
}
