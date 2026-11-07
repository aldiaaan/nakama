// nakama
// Copyright (C) 2026  aldiaaan <62921668+aldiaaan@users.noreply.github.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

mod error;

use std::fmt;
use std::thread;
use std::time::Duration;

pub use error::DeviceInfoError;

use serde::Serialize;
use sysinfo::{CpuRefreshKind, Gpus, MemoryRefreshKind, RefreshKind, System};

/// Gap between the two samples that CPU and GPU usage percentages need.
const SAMPLE_INTERVAL: Duration = Duration::from_millis(200);

/// Snapshot of the machine Nakama is running on.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub gpus: Vec<GpuInfo>,
    /// Set when the GPU list cannot be read. CPU and memory are still filled in.
    pub gpu_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuInfo {
    pub brand: String,
    pub vendor_id: String,
    pub physical_cores: Option<usize>,
    pub logical_cores: usize,
    pub usage_percent: f32,
    pub cores: Vec<CpuCoreInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuCoreInfo {
    pub name: String,
    pub frequency_mhz: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub free_bytes: u64,
    pub total_swap_bytes: u64,
    pub used_swap_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuInfo {
    pub pci: String,
    pub vendor: Option<String>,
    pub model: Option<String>,
    pub usage_percent: Option<f32>,
    pub total_memory_bytes: Option<u64>,
    pub used_memory_bytes: Option<u64>,
}

impl fmt::Display for DeviceInfo {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let physical = self
            .cpu
            .physical_cores
            .map(|count| count.to_string())
            .unwrap_or_else(|| "unknown".to_owned());
        writeln!(
            formatter,
            "CPU: {} ({})",
            self.cpu.brand, self.cpu.vendor_id
        )?;
        writeln!(
            formatter,
            "  cores: {} logical, {physical} physical",
            self.cpu.logical_cores
        )?;
        writeln!(formatter, "  usage: {:.1}%", self.cpu.usage_percent)?;
        for core in &self.cpu.cores {
            writeln!(
                formatter,
                "  {}: {:.1}% @ {} MHz",
                core.name, core.usage_percent, core.frequency_mhz
            )?;
        }

        writeln!(
            formatter,
            "Memory: {} / {} used",
            format_bytes(self.memory.used_bytes),
            format_bytes(self.memory.total_bytes)
        )?;
        writeln!(
            formatter,
            "  available: {}, free: {}",
            format_bytes(self.memory.available_bytes),
            format_bytes(self.memory.free_bytes)
        )?;
        writeln!(
            formatter,
            "Swap: {} / {} used",
            format_bytes(self.memory.used_swap_bytes),
            format_bytes(self.memory.total_swap_bytes)
        )?;

        if let Some(error) = &self.gpu_error {
            writeln!(formatter, "GPU: unavailable ({error})")?;
            return Ok(());
        }
        if self.gpus.is_empty() {
            writeln!(formatter, "GPU: none found")?;
            return Ok(());
        }
        for gpu in &self.gpus {
            let name = gpu
                .model
                .as_deref()
                .or(gpu.vendor.as_deref())
                .unwrap_or("GPU");
            writeln!(formatter, "GPU: {name} [{}]", gpu.pci)?;
            if let Some(vendor) = &gpu.vendor {
                writeln!(formatter, "  vendor: {vendor}")?;
            }
            match gpu.usage_percent {
                Some(usage) => writeln!(formatter, "  usage: {usage:.1}%")?,
                None => writeln!(formatter, "  usage: n/a")?,
            }
            match (gpu.used_memory_bytes, gpu.total_memory_bytes) {
                (Some(used), Some(total)) => writeln!(
                    formatter,
                    "  vram: {} / {}",
                    format_bytes(used),
                    format_bytes(total)
                )?,
                (None, Some(total)) => writeln!(formatter, "  vram: {}", format_bytes(total))?,
                _ => writeln!(formatter, "  vram: n/a")?,
            }
        }
        Ok(())
    }
}

/// Read CPU, memory, and GPU once.
///
/// Usage percentages are computed from two samples taken [`SAMPLE_INTERVAL`] apart.
/// A missing GPU list does not fail the snapshot; [`DeviceInfo::gpu_error`] explains it.
pub fn device_info() -> Result<DeviceInfo, DeviceInfoError> {
    let mut system = System::new_with_specifics(
        RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything()),
    )?;
    let gpus = Gpus::new_with_refreshed_list();

    thread::sleep(SAMPLE_INTERVAL);

    system.refresh_cpu_specifics(CpuRefreshKind::everything());
    system.refresh_memory();

    let (gpus, gpu_error) = match gpus {
        Ok(mut gpus) => {
            gpus.refresh(false);
            (gpus.iter().map(gpu_info).collect(), None)
        }
        Err(error) => (Vec::new(), Some(error.to_string())),
    };

    Ok(DeviceInfo {
        cpu: cpu_info(&system),
        memory: memory_info(&system),
        gpus,
        gpu_error,
    })
}

fn cpu_info(system: &System) -> CpuInfo {
    let cpus = system.cpus();
    let cores = cpus
        .iter()
        .map(|cpu| CpuCoreInfo {
            name: cpu.name().to_owned(),
            frequency_mhz: cpu.frequency(),
            usage_percent: cpu.usage(),
        })
        .collect::<Vec<_>>();
    let (brand, vendor_id) = cpus
        .first()
        .map(|cpu| (cpu.brand().to_owned(), cpu.vendor_id().to_owned()))
        .unwrap_or_else(|| ("Unknown".to_owned(), "Unknown".to_owned()));

    CpuInfo {
        brand,
        vendor_id,
        physical_cores: System::physical_core_count().ok(),
        logical_cores: cores.len(),
        usage_percent: system.global_cpu_usage(),
        cores,
    }
}

fn memory_info(system: &System) -> MemoryInfo {
    MemoryInfo {
        total_bytes: system.total_memory(),
        used_bytes: system.used_memory(),
        available_bytes: system.available_memory(),
        free_bytes: system.free_memory(),
        total_swap_bytes: system.total_swap(),
        used_swap_bytes: system.used_swap(),
    }
}

fn gpu_info(gpu: &sysinfo::Gpu) -> GpuInfo {
    GpuInfo {
        pci: gpu.pci().to_string(),
        vendor: gpu.vendor().map(str::to_owned),
        model: gpu.model().map(str::to_owned),
        usage_percent: gpu.usage(),
        total_memory_bytes: gpu.total_memory(),
        used_memory_bytes: gpu.used_memory(),
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::device_info;

    #[test]
    fn reads_cpu_and_memory() {
        let info = device_info().expect("device info");
        assert!(info.memory.total_bytes > 0);
        assert!(info.cpu.logical_cores > 0);
        assert!(!info.cpu.brand.is_empty());
        println!("{info}");
    }
}
