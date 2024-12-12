#[cfg(windows)]
use windows::Win32::System::Threading::{
    GetProcessAffinityMask, OpenProcess, SetProcessAffinityMask, PROCESS_QUERY_INFORMATION,
    PROCESS_SET_INFORMATION,
};

#[cfg(target_os = "linux")]
use nix::sched;

use crate::{ccx::Ccx, config::CcxPreference, watch::Process};

#[cfg(windows)]
pub fn set_affinity(
    process: &Process,
    preference: &CcxPreference,
    ccxes: &[Ccx],
) -> Result<bool, Box<dyn std::error::Error>> {
    let mask = preference.affinity(ccxes);

    unsafe {
        let mut curr_affinity = 0;
        let handle = OpenProcess(
            PROCESS_SET_INFORMATION | PROCESS_QUERY_INFORMATION,
            false,
            process.pid,
        )?;

        GetProcessAffinityMask(handle, &mut curr_affinity as _, &mut 0usize as _)?;

        // Only change the process affinity if the current affinity doesn't match
        if curr_affinity == mask {
            return Ok(false);
        }

        SetProcessAffinityMask(handle, mask)?;
    };

    Ok(true)
}

#[cfg(target_os = "linux")]
pub fn set_affinity(
    process: &Process,
    preference: &CcxPreference,
    ccxes: &[Ccx],
) -> Result<bool, Box<dyn std::error::Error>> {
    let prefered_cpu_set = preference.cpuset(&ccxes)?;
    let proc_cpu_set = sched::sched_getaffinity(process.pid)?;

    // Only change the process affinity if the current affinity doesn't match
    if prefered_cpu_set == proc_cpu_set {
        return Ok(false);
    }

    sched::sched_setaffinity(process.pid, &prefered_cpu_set)?;
    Ok(true)
}
