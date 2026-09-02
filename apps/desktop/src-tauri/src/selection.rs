#[cfg(target_os = "macos")]
#[path = "selection_macos.rs"]
mod selection_macos;
#[cfg(windows)]
#[path = "selection_windows.rs"]
mod selection_windows;

#[cfg(target_os = "macos")]
pub use selection_macos::*;
#[cfg(windows)]
pub use selection_windows::*;
