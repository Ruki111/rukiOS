use std::process::Command;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ServiceUnit {
    pub name: String,
    pub load: String,
    pub active: String,
    pub sub: String,
    pub description: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ServiceAction {
    Start,
    Stop,
    Restart,
}

impl ServiceAction {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Stop => "stop",
            Self::Restart => "restart",
        }
    }
}

pub(crate) fn list_services() -> Result<Vec<ServiceUnit>, String> {
    let output = Command::new("systemctl")
        .args([
            "list-units",
            "--type=service",
            "--all",
            "--no-pager",
            "--no-legend",
            "--plain",
        ])
        .output()
        .map_err(|error| format!("Could not run systemctl: {error}"))?;
    if !output.status.success() {
        return Err(command_error(&output.stderr, "Could not list services."));
    }
    let services = parse_services(&String::from_utf8_lossy(&output.stdout));
    Ok(services)
}

pub(crate) fn perform_service_action(action: ServiceAction, name: &str) -> Result<(), String> {
    if !is_service_unit_name(name) {
        return Err("Invalid service unit name.".into());
    }
    let output = Command::new("systemctl")
        .arg(action.as_str())
        .arg(name)
        .output()
        .map_err(|error| format!("Could not run systemctl: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(command_error(
            &output.stderr,
            &format!("systemctl {} failed for {name}.", action.as_str()),
        ))
    }
}

fn command_error(stderr: &[u8], fallback: &str) -> String {
    let detail = String::from_utf8_lossy(stderr).trim().to_string();
    if detail.is_empty() {
        fallback.to_string()
    } else {
        detail
    }
}

fn parse_services(output: &str) -> Vec<ServiceUnit> {
    let mut services = output
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let name = fields.next()?;
            if !name.ends_with(".service") {
                return None;
            }
            Some(ServiceUnit {
                name: name.to_string(),
                load: fields.next()?.to_string(),
                active: fields.next()?.to_string(),
                sub: fields.next()?.to_string(),
                description: fields.collect::<Vec<_>>().join(" "),
            })
        })
        .collect::<Vec<_>>();
    services.sort_by(|left, right| left.name.cmp(&right.name));
    services
}

fn is_service_unit_name(name: &str) -> bool {
    name.ends_with(".service")
        && name.len() > ".service".len()
        && !name.starts_with('-')
        && !name
            .chars()
            .any(|character| character.is_whitespace() || character == '/' || character == '\0')
}

#[cfg(test)]
mod tests {
    use super::{ServiceAction, is_service_unit_name, parse_services};

    #[test]
    fn parses_service_state_and_description() {
        let services = parse_services(
            "dbus.service loaded active running D-Bus System Message Bus\nnot-a-service loaded active running ignored\n",
        );
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "dbus.service");
        assert_eq!(services[0].load, "loaded");
        assert_eq!(services[0].active, "active");
        assert_eq!(services[0].sub, "running");
        assert_eq!(services[0].description, "D-Bus System Message Bus");
    }

    #[test]
    fn sorts_services_and_ignores_unrelated_or_malformed_rows() {
        let services = parse_services(
            "z-last.service loaded inactive dead Last service\na-first.service loaded active running First service\n0 loaded units listed.\nbad.service loaded\n",
        );
        assert_eq!(
            services
                .iter()
                .map(|service| service.name.as_str())
                .collect::<Vec<_>>(),
            ["a-first.service", "z-last.service"]
        );
    }

    #[test]
    fn service_unit_validation_rejects_option_injection_and_paths() {
        assert!(is_service_unit_name("cups.service"));
        assert!(!is_service_unit_name("--help.service"));
        assert!(!is_service_unit_name("../../cups.service"));
        assert!(!is_service_unit_name("cups"));
        assert_eq!(ServiceAction::Restart.as_str(), "restart");
    }
}
