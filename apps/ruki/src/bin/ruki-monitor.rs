#[path = "../monitor/mod.rs"]
mod monitor;
#[path = "../monitor/ui.rs"]
mod ui;

fn main() {
    if let Err(error) = ui::run() {
        eprintln!("Could not start ruki-monitor: {error}");
        std::process::exit(1);
    }
}
