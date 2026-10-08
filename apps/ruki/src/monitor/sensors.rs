use std::fs;

#[derive(Clone, Debug)]
pub struct Temperature {
    pub label: String,
    pub celsius: f64,
}

pub(super) fn read_all() -> Vec<Temperature> {
    let mut temperatures = Vec::new();
    for entry in fs::read_dir("/sys/class/hwmon")
        .into_iter()
        .flatten()
        .flatten()
    {
        let directory = entry.path();
        let chip = fs::read_to_string(directory.join("name"))
            .unwrap_or_default()
            .trim()
            .to_string();
        let Ok(files) = fs::read_dir(&directory) else {
            continue;
        };
        for file in files.flatten() {
            let filename = file.file_name().to_string_lossy().into_owned();
            if !filename.starts_with("temp") || !filename.ends_with("_input") {
                continue;
            }
            let Ok(raw) = fs::read_to_string(file.path()) else {
                continue;
            };
            let Ok(millidegrees) = raw.trim().parse::<i64>() else {
                continue;
            };
            let label_path = directory.join(filename.replace("_input", "_label"));
            let label = fs::read_to_string(label_path)
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| format!("{chip} {filename}"));
            let celsius = millidegrees as f64 / 1000.0;
            if (-20.0..=150.0).contains(&celsius) {
                temperatures.push(Temperature { label, celsius });
            }
        }
    }
    temperatures.sort_by(|a, b| a.label.cmp(&b.label));
    temperatures
}
