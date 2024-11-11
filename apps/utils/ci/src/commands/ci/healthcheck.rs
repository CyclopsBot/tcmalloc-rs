#![allow(clippy::cast_precision_loss, clippy::missing_panics_doc)]
use clap::Args;
use human_bytes::human_bytes;
use owo_colors::OwoColorize;
use std::io::{self, Write};
use sysinfo::{Disks, Networks, System};
use tracing::info;

#[derive(Args, Clone, Debug)]
pub struct CiHealthcheckArgs {}

pub fn healthcheck(_args: &CiHealthcheckArgs) {
    let mut sysinfo = System::new_all();
    sysinfo.refresh_all();

    info!(
        "Kernel: {}",
        System::kernel_version()
            .expect("unable to get kernel version")
            .to_string()
            .bright_black()
            .bold()
    );

    for disk in &Disks::new_with_refreshed_list() {
        let disk_name = disk.name().to_string_lossy().replace('"', "");
        let disk_free = format!(
            "{}/{}",
            human_bytes(disk.available_space() as f64),
            human_bytes(disk.total_space() as f64)
        );
        info!(
            "Disk: {} Free: {}",
            disk_name.bright_black().bold(),
            disk_free.bright_black().bold(),
        );
    }

    let networks = Networks::new_with_refreshed_list();
    for (interface_name, data) in &networks {
        let network_used = format!(
            "{} (down)/{} (up)",
            human_bytes(data.total_received() as f64),
            human_bytes(data.total_transmitted() as f64)
        );
        info!(
            "NIC: {} Usage: {}",
            interface_name.bright_black().bold(),
            network_used.bright_black().bold(),
        );
    }

    let memory_used = format!(
        "{}/{}",
        human_bytes(sysinfo.used_memory() as f64),
        human_bytes(sysinfo.total_memory() as f64)
    );
    info!("Memory: {}", memory_used.bright_black().bold(),);
    let swap_used = format!(
        "{}/{}",
        human_bytes(sysinfo.used_swap() as f64),
        human_bytes(sysinfo.total_swap() as f64)
    );
    info!("Swap: {}", swap_used.bright_black().bold(),);

    let uptime = std::process::Command::new("uptime")
        .output()
        .expect("failed to find uptime");

    io::stdout().write_all(&uptime.stdout).unwrap();
}
