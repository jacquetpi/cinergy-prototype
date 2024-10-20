use std::fs;
use std::io;
use std::path::{PathBuf, Path};
use std::process::Command;

#[derive(Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub cpu_time: String,
}

impl ProcessInfo {
    // Fetches the CPU time from /proc/[pid]/stat
    pub fn fetch_cpu_time(pid: u32) -> io::Result<String> {
        let stat_path = PathBuf::from(format!("/proc/{}/stat", pid));
        let content = fs::read_to_string(stat_path)?;

        // CPU time is the 14th and 15th fields in /proc/[pid]/stat
        let fields: Vec<&str> = content.split_whitespace().collect();
        if fields.len() < 15 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid stat file"));
        }

        let utime = fields[13]; // User time
        let stime = fields[14]; // System time
        Ok(format!("User time: {} ticks, System time: {} ticks", utime, stime))
    }
}

pub struct ProcessManager {
    pub process_infos: Vec<ProcessInfo>,
    pub search_term: String,
}

impl ProcessManager {
    pub fn new(search_term: String) -> Self {
        Self {
            process_infos: Vec::new(),
            search_term,
        }
    }

    // Retrieves all PIDs related to the search term
    pub fn retrieve_process_pids(&mut self) -> io::Result<()> {
        let output = Command::new("pgrep")
            .arg(&self.search_term)
            .output()?;

        let pids = String::from_utf8_lossy(&output.stdout);
        for pid in pids.lines() {
            if let Ok(pid_num) = pid.trim().parse::<u32>() {
                match ProcessInfo::fetch_cpu_time(pid_num) {
                    Ok(cpu_time) => self.process_infos.push(ProcessInfo {
                        pid: pid_num,
                        cpu_time,
                    }),
                    Err(e) => eprintln!("Failed to fetch CPU time for PID {}: {}", pid_num, e),
                }
            }
        }
        Ok(())
    }

    // Fetch and display CPU usage details for child processes of each process
    pub fn display_cpu_usage_between_children(&self) {
        for process in &self.process_infos {
            println!("Process PID: {}", process.pid);
            if let Ok(children) = self.get_children(process.pid) {
                for child in children {
                    if let Ok(cpu_time) = ProcessInfo::fetch_cpu_time(child) {
                        println!("  Child PID: {}, {}", child, cpu_time);
                    } else {
                        println!("  Child PID: {} - Failed to retrieve CPU time", child);
                    }
                }
            }
        }
    }

    // Get child processes for a given PID
    // TODO
    fn get_children(&self, pid: u32) -> io::Result<Vec<u32>> {
        let path = PathBuf::from(format!("/proc/{}/task", pid));
        let content = fs::read_to_string(path)?;
        print!("test {}",content);
        let child_pids = content
            .trim()
            .split_whitespace()
            .filter_map(|s| s.parse::<u32>().ok())
            .collect();

        Ok(child_pids)
    }

    // Display all retrieved process information
    pub fn display_process_infos(&self) {
        for process in &self.process_infos {
            println!("PID: {}, {}", process.pid, process.cpu_time);
        }
    }
}
