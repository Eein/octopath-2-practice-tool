use std::time::{Duration, Instant};
use sysinfo::{ProcessRefreshKind, RefreshKind, System, SystemExt};

pub struct ProcessList {
    system: System,
    last_check: Instant,
}

impl ProcessList {
    pub fn new() -> Self {
        Self {
            system: System::new_with_specifics(
                RefreshKind::new().with_processes(ProcessRefreshKind::new()),
            ),
            last_check: Instant::now() - Duration::from_secs(1),
        }
    }
}
