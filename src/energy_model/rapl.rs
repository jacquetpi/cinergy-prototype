use std::fs;
use std::path::Path;
use std::time::Instant;

//use std::time::{Duration, Instant};

pub struct Rapl {
    last_read: Instant,
    last_energy: f64,
}

impl Rapl {
    // Create a new instance of Rapl
    pub fn new() -> Self {
        Rapl {
            last_read: Instant::now(),
            last_energy: 0.0,
        }
    }

    // Read the power consumption since the last call
    pub fn read_power(&mut self) -> Result<f64, String> {
        let current_energy = self.get_energy()?;
        let duration = self.last_read.elapsed();

        // Update the last read time and energy
        self.last_read = Instant::now();

        // Only calculate power if this is not the first call
        if self.last_energy == 0.0 {
            self.last_energy = current_energy; // Initialize the last energy for the first run
            return Ok(0.0); // Return 0 for the first run
        }

        // Calculate the power consumption in watts since the last call
        let delta_energy = current_energy - self.last_energy;
        let seconds = duration.as_secs_f64();
        self.last_energy = current_energy; // Update the last energy value

        // Return power consumption in watts
        if seconds > 0.0 {
            let power = delta_energy / seconds; // Power in watts
            Ok(power)
        } else {
            Ok(0.0) // No time elapsed
        }
    }

    // Read the energy consumption from the RAPL interface
    fn get_energy(&self) -> Result<f64, String> {
        let path = Path::new("/sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj");

        // Read the energy value from the file
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Error reading energy file: {}", e))?;

        // Parse the content as uJ (microjoules)
        let energy_uj: f64 = content.trim().parse()
            .map_err(|e| format!("Error parsing energy value: {}", e))?;

        // Convert microjoules to joules (1 J = 1,000,000 µJ)
        Ok(energy_uj / 1_000_000.0)
    }
}
