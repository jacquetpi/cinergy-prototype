// libvirt_interface.rs

use virt::connect::Connect;
use virt::error::Error;
use virt::sys;

pub struct LibvirtInterface {
    conn: Connect,
}

impl LibvirtInterface {
    pub fn new(uri: &str) -> Result<Self, Error> {
        let conn = Connect::open(Some(uri))?;
        Ok(LibvirtInterface { conn })
    }

    pub fn show_hypervisor_info(&self) -> Result<(), Error> {
        if let Ok(hv_type) = self.conn.get_type() {
            if let Ok(mut hv_ver) = self.conn.get_hyp_version() {
                let major = hv_ver / 1000000;
                hv_ver %= 1000000;
                let minor = hv_ver / 1000;
                let release = hv_ver % 1000;
                println!(
                    "Hypervisor: '{}' version: {}.{}.{}",
                    hv_type, major, minor, release
                );
                return Ok(());
            }
        }
        Err(Error::last_error())
    }

    pub fn show_domains(&self) -> Result<(), Error> {
        let flags = sys::VIR_CONNECT_LIST_DOMAINS_ACTIVE | sys::VIR_CONNECT_LIST_DOMAINS_INACTIVE;

        if let Ok(num_active_domains) = self.conn.num_of_domains() {
            if let Ok(num_inactive_domains) = self.conn.num_of_defined_domains() {
                println!(
                    "There are {} active and {} inactive domains",
                    num_active_domains, num_inactive_domains
                );
                if let Ok(doms) = self.conn.list_all_domains(flags) {
                    for dom in doms {
                        let id = dom.get_id().unwrap_or(0);
                        let name = dom.get_name().unwrap_or_else(|_| String::from("no-name"));
                        let active = dom.is_active().unwrap_or(false);
                        println!("ID: {}, Name: {}, Active: {}", id, name, active);
                    }
                }
                return Ok(());
            }
        }
        Err(Error::last_error())
    }

    pub fn disconnect(mut self) {
        if let Err(e) = self.conn.close() {
            panic!("Failed to disconnect from hypervisor: {}", e);
        }
        println!("Disconnected from hypervisor");
    }
}
