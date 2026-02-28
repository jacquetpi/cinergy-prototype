pub mod cpu_time;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::thread;
use self::cpu_time::{get_qemu_pids, get_cpu_time};

const MAX_HISTORY: usize = 10;

#[derive(Clone)]
pub struct CpuUsageTracker {
    history: HashMap<u32, Vec<f64>>,
    vm_names_to_pid: HashMap<String, u32>,
    vm_vcpus: HashMap<String, u32>,
    last_times: HashMap<u32, (u64, Instant)>,
}

impl CpuUsageTracker {
    pub fn new() -> Self {
        CpuUsageTracker {
            history: HashMap::new(),
            vm_names_to_pid: HashMap::new(),
            vm_vcpus: HashMap::new(),
            last_times: HashMap::new(),
        }
    }

    /// Start periodic tracking of CPU usage for QEMU processes in a background thread.
    pub fn track_cpu_usage(tracker: Arc<Mutex<Self>>) {
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(1));

                let mut tracker = tracker.lock().unwrap();
                let qemu_processes = get_qemu_pids();

                for proc in qemu_processes {
                    tracker.vm_names_to_pid.insert(proc.vm_name.clone(), proc.pid);
                    tracker.vm_vcpus.insert(proc.vm_name.clone(), proc.vcpu_count);
                    if let Some(cpu_time) = get_cpu_time(proc.pid) {
                        tracker.update_cpu_usage(proc.pid, cpu_time);
                    }
                }
            }
        });
    }

    fn update_cpu_usage(&mut self, pid: u32, current_cpu_time: u64) {
        let current_time = Instant::now();

        if let Some(&(last_cpu_time, last_instant)) = self.last_times.get(&pid) {
            let elapsed_secs = current_time.duration_since(last_instant).as_secs_f64();
            if elapsed_secs > 0.0 {
                let cpu_usage = (current_cpu_time.saturating_sub(last_cpu_time) as f64) / elapsed_secs;

                let history = self.history.entry(pid).or_insert_with(Vec::new);
                history.push(cpu_usage);
                if history.len() > MAX_HISTORY {
                    history.remove(0);
                }
            }
        }

        self.last_times.insert(pid, (current_cpu_time, current_time));
    }

    /// Retrieve the most recent CPU usage for a VM by name.
    pub fn get_last_cpu_usage_for_vm(&self, vm_name: &str) -> Option<f64> {
        let pid = self.vm_names_to_pid.get(vm_name)?;
        self.history.get(pid)?.last().copied()
    }

    /// Retrieve the detected vCPU count for a VM by name.
    pub fn get_vcpu_count_for_vm(&self, vm_name: &str) -> Option<u32> {
        self.vm_vcpus.get(vm_name).copied()
    }
}
