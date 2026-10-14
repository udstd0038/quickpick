//! AppKit calls stay on Tauri's main thread; only numeric cursor coordinates leave it.
use objc2::{rc::Retained, MainThreadMarker};
use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior as Behavior};
use std::ffi::c_void;

fn popup_behavior(current: Behavior) -> Behavior {
    (current
        & !(Behavior::MoveToActiveSpace | Behavior::FullScreenPrimary | Behavior::FullScreenNone))
        | Behavior::CanJoinAllSpaces
        | Behavior::FullScreenAuxiliary
}

pub fn configure_popup(window: &tauri::WebviewWindow) -> Result<(), String> {
    if MainThreadMarker::new().is_none() {
        let owned = window.clone();
        return window
            .run_on_main_thread(move || {
                if configure_popup(&owned).is_err() {
                    eprintln!("QuickPick popup window configuration failed");
                }
            })
            .map_err(|_| "无法安排原生窗口配置".to_string());
    }
    let _main = MainThreadMarker::new().ok_or_else(|| "窗口配置需要主线程".to_string())?;
    window
        .set_visible_on_all_workspaces(true)
        .map_err(|_| "无法配置跨空间窗口".to_string())?;
    let pointer = window
        .ns_window()
        .map_err(|_| "无法读取原生窗口".to_string())?;
    // ns_window is a borrowed, live NSWindow for this WebviewWindow. Retain only
    // within this main-thread call; neither the raw pointer nor NSWindow is Send.
    let native = unsafe { Retained::<NSWindow>::retain(pointer.cast()) }
        .ok_or_else(|| "原生窗口不可用".to_string())?;
    native.setCollectionBehavior(popup_behavior(native.collectionBehavior()));
    Ok(())
}

#[repr(C)]
struct Point {
    x: f64,
    y: f64,
}

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGEventCreate(source: *const c_void) -> *const c_void;
    fn CGEventGetLocation(event: *const c_void) -> Point;
}
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFRelease(object: *const c_void);
}

/// CoreGraphics desktop coordinates use a top-left origin and points, including
/// negative origins on external displays; Tauri cursor_position is physical pixels.
pub fn cursor_position() -> Option<(i32, i32)> {
    let event = unsafe { CGEventCreate(std::ptr::null()) };
    if event.is_null() {
        return None;
    }
    let position = unsafe { CGEventGetLocation(event) };
    unsafe { CFRelease(event) };
    if !position.x.is_finite()
        || !position.y.is_finite()
        || position.x < i32::MIN as f64
        || position.x > i32::MAX as f64
        || position.y < i32::MIN as f64
        || position.y > i32::MAX as f64
    {
        return None;
    }
    Some((position.x.floor() as i32, position.y.floor() as i32))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn popup_joins_spaces_and_fullscreen_without_conflicting_flags() {
        let original = Behavior::Stationary
            | Behavior::MoveToActiveSpace
            | Behavior::FullScreenPrimary
            | Behavior::FullScreenNone;
        let behavior = popup_behavior(original);
        assert!(behavior.contains(
            Behavior::Stationary | Behavior::CanJoinAllSpaces | Behavior::FullScreenAuxiliary
        ));
        assert!(!behavior.intersects(
            Behavior::MoveToActiveSpace | Behavior::FullScreenPrimary | Behavior::FullScreenNone
        ));
        assert_eq!(popup_behavior(behavior), behavior);
    }
}
