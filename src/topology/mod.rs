#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::*;
#[cfg(target_os = "windows")]
pub use windows::*;

/// A partial schema of `lscpu`'s representation of a CPU core.
#[derive(Debug, Clone, Copy)]
pub struct Cpu {
    /// The ID of the physical core.
    pub physical_core: usize,
    /// The ID of the logical core, there may be multiple logical ores per `cpu` if the processor
    /// supports SMT.
    #[cfg(target_os = "linux")]
    pub logical_core: usize,
    /// The mask of the CPU affinity the CPU applies to.
    #[cfg(target_os = "windows")]
    pub affinity_mask: usize,
    /// The ID of the core's L3 cache.
    pub l3_group: CacheGroup,
}

#[derive(Debug, Clone, Copy)]
pub struct CacheGroup {
    /// The ID of the cache group.
    pub id: usize,
    /// The size of the cache.
    pub size: usize,
}
