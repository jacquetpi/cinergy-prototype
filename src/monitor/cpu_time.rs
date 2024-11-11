use std::process::Command;
use std::fs;
use regex::Regex;

// Retrieve pairs of PID/name using pgrep
pub fn get_qemu_pids() -> Vec<(u32, String)> {
    let output = Command::new("pgrep")
        .arg("qemu")
        .output()
        .expect("Failed to execute pgrep");

    let pids_and_names: Vec<(u32, String)> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            if let Ok(pid) = line.trim().parse::<u32>() {
                // Attempt to read the command line for the PID
                let cmdline_path = format!("/proc/{}/cmdline", pid);
                let vm_name = match fs::read_to_string(&cmdline_path) {
                    Ok(cmdline) => parse_vm_name(&cmdline),
                    Err(_) => "Unknown".to_string(),
                };
                Some((pid, vm_name))
            } else {
                None
            }
        })
        .collect();

    pids_and_names
}


// Retrieve CPU time for a specific PID from /proc
pub fn get_cpu_time(pid: u32) -> Option<u64> {
    let stat_path = format!("/proc/{}/stat", pid);
    let stat_content = fs::read_to_string(stat_path).ok()?;
    let fields: Vec<&str> = stat_content.split_whitespace().collect();

    // Get user time (13th field) and kernel time (14th field)
    let utime: u64 = fields.get(13)?.parse().ok()?;
    let stime: u64 = fields.get(14)?.parse().ok()?;

    Some(utime + stime)
}

fn parse_vm_name(cmdline: &str) -> String {
    // Define a regular expression to match the nameguest= parameter and extract the value
    let cmdline_cleaned = cmdline.replace('\0', "");
    let re = Regex::new(r"nameguest=([^\s,]+)").unwrap();

    // Return the first match or a default value
    if let Some(captures) = re.captures(&cmdline_cleaned) {
        captures.get(1).map_or("Unknown".to_string(), |m| m.as_str().to_string())
    } else {
        "Unknown".to_string()
    }
}
