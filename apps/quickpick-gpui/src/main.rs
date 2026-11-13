#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#[cfg(windows)]
mod native_probe;
#[cfg(windows)]
mod probe;

#[cfg(windows)]
mod app;
#[cfg(windows)]
mod native;
#[cfg(windows)]
mod overlay;
#[cfg(windows)]
mod settings;
#[cfg(any(windows, test))]
mod settings_choices;
#[cfg(windows)]
mod tool;
#[cfg(windows)]
mod ui;

#[cfg(windows)]
fn main() {
    if std::env::args().any(|arg| arg == "--hotkey-bindings") {
        app::print_bindings();
        return;
    }
    if std::env::args().skip(1).any(|arg| arg == "--feasibility") {
        probe::run();
    } else {
        app::run();
    }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("QuickPick GPUI 候选仅支持 Windows 11 x64；本机可运行 core 单元测试。");
    std::process::exit(2);
}
