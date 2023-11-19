use crate::topology::CacheGroup;

use super::Cpu;

use windows::{
    core::Result,
    Win32::System::SystemInformation::{
        CacheData, CacheUnified, GetLogicalProcessorInformationEx, RelationAll, RelationCache,
        RelationProcessorCore, SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX,
    },
};

type ProcInfoEx = SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX;

/// Read the CPU topology via [GetLogicalProcessorInformationEx].
pub fn cpus() -> impl Iterator<Item = Cpu> {
    let mut cache_groups = Vec::new();
    let mut processors: Vec<usize> = Vec::new();

    for info in ProcInfoIterator::new().unwrap() {
        if let Some(group) = WindowsCacheGroup::from_l3(&info) {
            cache_groups.push(group);
        }

        if info.Relationship == RelationProcessorCore {
            let proccssor = unsafe { info.Anonymous.Processor };
            processors.push(proccssor.GroupMask[0].Mask);
        }
    }

    processors.into_iter().map(move |mask| {
        let physical_core = mask.trailing_zeros() as usize >> 1; // Zen only has 2 way SMT.
        let (id, group) = cache_groups
            .iter()
            .enumerate()
            .find(|(_, group)| group.mask & mask > 0)
            .unwrap();

        Cpu {
            physical_core,
            affinity_mask: mask,
            l3_group: CacheGroup {
                id,
                size: group.l3_size,
            },
        }
    })
}

/// The range of logical cores the L3 [CacheGroup] applies to.
#[derive(Debug, Clone)]
pub struct WindowsCacheGroup {
    /// The logical processors the cache group belongs to.
    mask: usize,
    // How big the L3 cache size is, might vary with X3D processors.
    l3_size: usize,
}

impl WindowsCacheGroup {
    pub fn from_l3(info: &ProcInfoEx) -> Option<Self> {
        if info.Relationship != RelationCache {
            return None;
        }

        let cache = unsafe { info.Anonymous.Cache };

        if (cache.Type == CacheData || cache.Type == CacheUnified) && cache.Level == 3 {
            let group_mask = unsafe { cache.Anonymous.GroupMask };
            let mask = group_mask.Mask;
            Some(Self {
                mask,
                l3_size: cache.CacheSize as _,
            })
        } else {
            None
        }
    }
}

pub struct ProcInfoIterator {
    buffer: Vec<u8>,
    i: usize,
}

impl ProcInfoIterator {
    pub fn new() -> Result<Self> {
        let mut buffer_size: u32 = 0;

        // This function is intentionally meant to fail the first time so we can get the size of the buffer.
        let _ =
            unsafe { GetLogicalProcessorInformationEx(RelationAll, None, &mut buffer_size as _) };

        // Now lets allocate a buffer we can place the variable-size processor infos.
        let mut buffer = vec![0u8; buffer_size as _];

        // Populate the infos buffer.
        unsafe {
            GetLogicalProcessorInformationEx(
                RelationAll,
                Some(buffer.as_mut_ptr() as _),
                &mut buffer_size as _,
            )
        }?;

        Ok(Self { buffer, i: 0 })
    }
}

impl Iterator for ProcInfoIterator {
    type Item = ProcInfoEx;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.buffer.len() {
            return None;
        }

        let info: &ProcInfoEx = unsafe {
            let info_ptr: *const ProcInfoEx = self.buffer.as_mut_ptr().add(self.i) as _;
            &*info_ptr
        };

        self.i += info.Size as usize;
        Some(*info)
    }
}
