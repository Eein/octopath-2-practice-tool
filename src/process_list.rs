use std::{
    str,
    time::{Duration, Instant},
};
use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System, SystemExt};

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

    pub fn refresh(&mut self) {
        let now = Instant::now();
        if now - self.last_check >= Duration::from_secs(1) {
            self.system
                .refresh_processes_specifics(ProcessRefreshKind::new());
            self.last_check = now;
        }
    }

    pub fn processes_by_name<'a>(
        &'a self,
        name: &'a str,
    ) -> Box<dyn Iterator<Item = &'a sysinfo::Process> + 'a> {
        self.system.processes_by_name(name)
    }

    pub fn is_open(&mut self, pid: Pid) -> bool {
        self.refresh();
        self.system.process(pid).is_some()
    }
}
