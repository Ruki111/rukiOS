use std::{fs, process::Command};

#[derive(Clone, Debug)]
pub struct GpuMetrics {
    pub name: String,
    pub utilization_percent: Option<u16>,
    pub memory_used_bytes: Option<u64>,
    pub memory_total_bytes: Option<u64>,
    pub temperature_celsius: Option<f64>,
}

pub(super) fn read_all() -> Vec<GpuMetrics> {
    let mut gpus = read_nvidia_smi();
    for entry in fs::read_dir("/sys/class/drm")
        .into_iter()
        .flatten()
        .flatten()
    {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !is_card(&name) {
            continue;
        }
        let device = entry.path().join("device");
        let driver = fs::read_to_string(device.join("uevent")).unwrap_or_default();
        let driver = driver
            .lines()
            .find_map(|line| line.strip_prefix("DRIVER="))
            .unwrap_or("");
        if driver == "nvidia"
            && gpus
                .iter()
                .any(|gpu| gpu.name.to_lowercase().contains("nvidia"))
        {
            continue;
        }
        let vendor = fs::read_to_string(device.join("vendor")).unwrap_or_default();
        let product = fs::read_to_string(device.join("product_name"))
            .ok()
            .map(|s| s.trim().to_string());
        let vendor_name = match vendor.trim() {
            "0x8086" => "Intel",
            "0x1002" => "AMD",
            "0x10de" => "NVIDIA",
            _ => "GPU",
        };
        let name = product
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("{vendor_name} graphics ({name})"));
        let utilization_percent = read_number::<u16>(&device.join("gpu_busy_percent"))
            .or_else(|| read_number::<u16>(&entry.path().join("gt_busy")));
        let memory_used_bytes = read_number::<u64>(&device.join("mem_info_vram_used"));
        let memory_total_bytes = read_number::<u64>(&device.join("mem_info_vram_total"));
        if !gpus.iter().any(|gpu| gpu.name == name) {
            gpus.push(GpuMetrics {
                name,
                utilization_percent,
                memory_used_bytes,
                memory_total_bytes,
                temperature_celsius: None,
            });
        }
    }
    gpus
}

fn read_nvidia_smi() -> Vec<GpuMetrics> {
    let Ok(output) = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,utilization.gpu,memory.used,memory.total,temperature.gpu",
            "--format=csv,noheader,nounits",
        ])
        .output()
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let fields = line.split(',').map(str::trim).collect::<Vec<_>>();
            Some(GpuMetrics {
                name: fields.first()?.to_string(),
                utilization_percent: parse_field(fields.get(1)?),
                memory_used_bytes: parse_field::<u64>(fields.get(2)?)
                    .map(|mib| mib.saturating_mul(1024 * 1024)),
                memory_total_bytes: parse_field::<u64>(fields.get(3)?)
                    .map(|mib| mib.saturating_mul(1024 * 1024)),
                temperature_celsius: parse_field(fields.get(4)?),
            })
        })
        .collect()
}

fn is_card(name: &str) -> bool {
    name.strip_prefix("card")
        .is_some_and(|suffix| !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()))
}

fn read_number<T: std::str::FromStr>(path: &std::path::Path) -> Option<T> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

fn parse_field<T: std::str::FromStr>(value: &str) -> Option<T> {
    if value == "[N/A]" || value.eq_ignore_ascii_case("n/a") {
        None
    } else {
        value.parse().ok()
    }
}
