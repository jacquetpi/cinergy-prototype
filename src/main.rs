mod energy_model;
mod rest_api;
mod scope;

use std::time::Duration;
use std::env;
use std::process;

//use libvirt_interface::LibvirtInterface;
use scope::process_interface::ProcessManager;
use energy_model::rapl::Rapl;

fn main() {
    let mut rapl = Rapl::new();

    // Launch the REST API
    //rest_api::launch();

    // let hypervisor_uri = "qemu:///system"; // Update this to your Hypervisor URI
    // println!("Attempting to connect to hypervisor: '{}'", hypervisor_uri);
    // let libvirt = match LibvirtInterface::new(&hypervisor_uri) {
    //     Ok(conn) => conn,
    //     Err(e) => panic!("No connection to hypervisor: {}", e),
    // };

    let mut manager = ProcessManager::new("qemu".to_string());
    if let Err(e) = manager.retrieve_process_pids() {
        eprintln!("Error retrieving process PIDs: {}", e);
        process::exit(1);
    }
    manager.display_process_infos();
    manager.display_cpu_usage_between_children();

    // Simulate a call to read power consumption
    loop {
        if let Ok(power) = rapl.read_power() {
            println!("Power Consumption: {:.2} W", power);
        } else {
            eprintln!("Failed to read power consumption");
        }
        std::thread::sleep(Duration::from_secs(1)); // Adjust the sleep duration as needed
    }
}
