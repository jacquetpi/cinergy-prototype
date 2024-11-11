pub mod cpu_time;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::thread;
use self::cpu_time::{get_qemu_pids, get_cpu_time};

const MAX_HISTORY: usize = 10;

#[derive(Clone)]
pub struct CpuUsageTracker {
    history: HashMap<u32, Vec<f64>>,     // Maps PID to a history of CPU usages
    vm_names_to_pid: HashMap<String, u32>, // Maps VM names to PIDs
    last_times: HashMap<u32, (u64, Instant)>, // Last CPU time and timestamp per PID
}

impl CpuUsageTracker {
    pub fn new() -> Self {
        CpuUsageTracker {
            history: HashMap::new(),
            vm_names_to_pid: HashMap::new(),
            last_times: HashMap::new(),
        }
    }

    // Start periodic tracking of CPU usage for QEMU processes
    pub fn track_cpu_usage(tracker: Arc<Mutex<Self>>) {
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(1));

                let mut tracker = tracker.lock().unwrap();
                let qemu_pids_and_names = get_qemu_pids();

                for (pid, vm_name) in qemu_pids_and_names {
                    tracker.vm_names_to_pid.insert(vm_name.clone(), pid);
                    if let Some(cpu_time) = get_cpu_time(pid) {
                        tracker.update_cpu_usage(pid, cpu_time);
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

                // Track the history of CPU usage for this PID
                self.history.entry(pid).or_insert_with(Vec::new).push(cpu_usage);
                if self.history[&pid].len() > MAX_HISTORY {
                    self.history.get_mut(&pid).unwrap().remove(0); // Keep history size manageable
                }
            }
        }

        // Update the last CPU time and timestamp for the next iteration
        self.last_times.insert(pid, (current_cpu_time, current_time));
    }

    // Retrieve the most recent CPU usage for a VM by name
    pub fn get_last_cpu_usage_for_vm(&self, vm_name: &str) -> Option<f64> {
        if let Some(pid) = self.vm_names_to_pid.get(vm_name) {
            if let Some(cpu_usage_history) = self.history.get(pid) {
                return cpu_usage_history.last().copied();
            }
        }
        None // Return None if no CPU usage is available for the VM
    }
}