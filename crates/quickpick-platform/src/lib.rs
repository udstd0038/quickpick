//! QuickPick Windows 平台适配：捕获、剪贴板、取词、密钥和系统集成。

#[cfg(windows)]
pub mod autostart;
#[cfg(windows)]
pub mod capture;
#[cfg(windows)]
pub mod window_events;

/// 所有取词和截图工作共用此入口锁；取消不可中断的系统调用时仍持锁至返回。
pub static CAPTURE_ENTRY: std::sync::Mutex<()> = std::sync::Mutex::new(());
#[cfg(windows)]
pub mod probe;
pub mod security;
#[cfg(windows)]
pub mod selection;
#[cfg(windows)]
pub mod system;

#[cfg(any(windows, test))]
mod bitmap;
