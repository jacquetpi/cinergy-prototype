use std::process::Command;
use std::fs;
use regex::Regex;

pub struct QemuProcess {
    pub pid: u32,
    pub vm_name: String,
    pub vcpu_count: u32,
}

/// Discover running QEMU processes, extracting PID, VM name, and vCPU count.
pub fn get_qemu_pids() -> Vec<QemuProcess> {
    let output = match Command::new("pgrep").arg("qemu").output() {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let pid = line.trim().parse::<u32>().ok()?;
            let cmdline_path = format!("/proc/{}/cmdline", pid);
            let cmdline = fs::read_to_string(&cmdline_path).unwrap_or_default();
            Some(QemuProcess {
                pid,
                vm_name: parse_vm_name(&cmdline),
                vcpu_count: parse_vcpu_count(&cmdline),
            })
        })
        .collect()
}

/// Read user + kernel CPU time (in clock ticks) for a given PID from /proc.
pub fn get_cpu_time(pid: u32) -> Option<u64> {
    let stat_path = format!("/proc/{}/stat", pid);
    let stat_content = fs::read_to_string(stat_path).ok()?;
    let fields: Vec<&str> = stat_content.split_whitespace().collect();

    let utime: u64 = fields.get(13)?.parse().ok()?;
    let stime: u64 = fields.get(14)?.parse().ok()?;

    Some(utime + stime)
}

/// Extract VM name from QEMU command line (`-name guest=VMNAME,...`).
pub fn parse_vm_name(cmdline: &str) -> String {
    let cmdline_cleaned = cmdline.replace('\0', " ");
    let re = Regex::new(r"guest=([^\s,]+)").unwrap();

    re.captures(&cmdline_cleaned)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

/// Extract vCPU count from QEMU command line (`-smp N` or `-smp cpus=N,...`).
/// Defaults to 1 if not found.
pub fn parse_vcpu_count(cmdline: &str) -> u32 {
    let cmdline_cleaned = cmdline.replace('\0', " ");
    let re = Regex::new(r"-smp\s+(?:cpus=)?(\d+)").unwrap();

    re.captures(&cmdline_cleaned)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vm_name_with_guest() {
        let cmdline = "qemu-system-x86_64\0-name\0guest=myvm,debug-threads=on\0-smp\02";
        assert_eq!(parse_vm_name(cmdline), "myvm");
    }

    #[test]
    fn test_parse_vm_name_guest_only() {
        let cmdline = "qemu-system-x86_64\0-name\0guest=test-vm\0-m\01024";
        assert_eq!(parse_vm_name(cmdline), "test-vm");
    }

    #[test]
    fn test_parse_vm_name_unknown() {
        let cmdline = "qemu-system-x86_64\0-m\01024";
        assert_eq!(parse_vm_name(cmdline), "Unknown");
    }

    #[test]
    fn test_parse_vm_name_empty() {
        assert_eq!(parse_vm_name(""), "Unknown");
    }

    #[test]
    fn test_parse_vcpu_count_simple() {
        let cmdline = "qemu\0-smp\04\0-m\01024";
        assert_eq!(parse_vcpu_count(cmdline), 4);
    }

    #[test]
    fn test_parse_vcpu_count_with_cpus_prefix() {
        let cmdline = "qemu\0-smp\0cpus=8,maxcpus=16\0-m\01024";
        assert_eq!(parse_vcpu_count(cmdline), 8);
    }

    #[test]
    fn test_parse_vcpu_count_default() {
        let cmdline = "qemu\0-m\01024";
        assert_eq!(parse_vcpu_count(cmdline), 1);
    }

    #[test]
    fn test_parse_vcpu_count_empty() {
        assert_eq!(parse_vcpu_count(""), 1);
    }
}
