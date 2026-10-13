//! macOS adapter for selection clipboard fallback. NSPasteboard has no atomic
//! compare-and-swap: guarded restoration avoids detected later writers, but a
//! writer can still race the final count check and clearContents.

use std::sync::Mutex;
use std::time::{Duration, Instant};
use std::{
    ffi::{c_ulong, c_void},
    ptr::NonNull,
};

use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use objc2::rc::{autoreleasepool, Retained};
use objc2::runtime::ProtocolObject;
use objc2_app_kit::{
    NSEvent, NSEventModifierFlags, NSPasteboard, NSPasteboardItem, NSPasteboardWriting,
};
use objc2_foundation::{NSArray, NSData, NSString};
use zeroize::Zeroizing;

use crate::selection_clipboard::{
    capture_selection as capture_transaction, is_promised_type, press_and_release_copy_key,
    select_command_copy_key, ClipboardBackend, ClipboardSnapshot, Clock, CopyKey, CopyKeyboard,
    FallbackError, Representation, TransactionConfig, MAX_SNAPSHOT_BYTES, MAX_SNAPSHOT_ITEMS,
    MAX_TYPES_PER_ITEM, MAX_TYPE_NAME_BYTES,
};

static CAPTURE_GATE: Mutex<()> = Mutex::new(());

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    // CGEventSourceStateID is int32_t; CGKeyCode is unsigned short.
    fn CGEventSourceKeyState(state_id: i32, key: u16) -> bool;
    fn CGPreflightListenEventAccess() -> bool;
}

#[link(name = "Carbon", kind = "framework")]
extern "C" {
    fn TISCopyCurrentKeyboardLayoutInputSource() -> *const c_void;
    fn TISGetInputSourceProperty(source: *const c_void, key: *const c_void) -> *const c_void;
    static kTISPropertyUnicodeKeyLayoutData: *const c_void;
    fn LMGetKbdType() -> u8;
    fn UCKeyTranslate(
        layout: *const u8,
        key: u16,
        action: u16,
        modifiers: u32,
        keyboard_type: u32,
        options: u32,
        dead_key_state: *mut u32,
        max_length: c_ulong,
        actual_length: *mut c_ulong,
        output: *mut u16,
    ) -> i32;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(object: *const c_void);
    fn CFEqual(left: *const c_void, right: *const c_void) -> u8;
    fn CFGetTypeID(object: *const c_void) -> usize;
    fn CFDataGetTypeID() -> usize;
    fn CFDataGetLength(data: *const c_void) -> isize;
    fn CFDataGetBytePtr(data: *const c_void) -> *const u8;
}

struct KeyboardLayoutSource(NonNull<c_void>);

impl Drop for KeyboardLayoutSource {
    fn drop(&mut self) {
        unsafe { CFRelease(self.0.as_ptr()) };
    }
}

impl KeyboardLayoutSource {
    fn current() -> Result<Self, FallbackError> {
        NonNull::new(unsafe { TISCopyCurrentKeyboardLayoutInputSource() }.cast_mut())
            .map(Self)
            .ok_or(FallbackError::KeyboardLayoutUnavailable)
    }

    fn data(&self) -> Result<&[u8], FallbackError> {
        let data =
            unsafe { TISGetInputSourceProperty(self.0.as_ptr(), kTISPropertyUnicodeKeyLayoutData) };
        if data.is_null() || unsafe { CFGetTypeID(data) != CFDataGetTypeID() } {
            return Err(FallbackError::KeyboardLayoutUnavailable);
        }
        let length = usize::try_from(unsafe { CFDataGetLength(data) })
            .ok()
            .filter(|length| *length > 0 && *length <= 1024 * 1024)
            .ok_or(FallbackError::KeyboardLayoutUnavailable)?;
        let bytes = unsafe { CFDataGetBytePtr(data) };
        if bytes.is_null() {
            return Err(FallbackError::KeyboardLayoutUnavailable);
        }
        // The borrowed property is retained by this live input source.
        Ok(unsafe { std::slice::from_raw_parts(bytes, length) })
    }
}

struct CopyLayout {
    source: KeyboardLayoutSource,
    bytes: Vec<u8>,
    keyboard_type: u32,
    keycode: u16,
}

impl CopyLayout {
    fn current() -> Result<Self, FallbackError> {
        let source = KeyboardLayoutSource::current()?;
        let native_bytes = source.data()?;
        let bytes = native_bytes.to_vec();
        let keyboard_type = unsafe { LMGetKbdType() } as u32;
        let keycode = select_command_copy_key(|key| {
            let mut dead_key = 0u32;
            // UniCharCount is unsigned long, not UInt32, in the LP64 SDK.
            let mut length = 0 as c_ulong;
            let mut output = [0u16; 8];
            // Apple UCKeyTranslate: kUCKeyActionDown=0; cmdKey >> 8 = 1;
            // kUCKeyTranslateNoDeadKeysMask=1. Fail closed on unknown layouts.
            let status = unsafe {
                UCKeyTranslate(
                    native_bytes.as_ptr(),
                    key,
                    0,
                    1,
                    keyboard_type,
                    1,
                    &mut dead_key,
                    output.len() as c_ulong,
                    &mut length,
                    output.as_mut_ptr(),
                )
            };
            (status == 0 && length == 1).then_some(output[0])
        })
        .ok_or(FallbackError::KeyboardLayoutUnavailable)?;
        Ok(Self {
            source,
            bytes,
            keyboard_type,
            keycode,
        })
    }

    fn is_current(&self) -> bool {
        let Ok(source) = KeyboardLayoutSource::current() else {
            return false;
        };
        (unsafe { CFEqual(self.source.0.as_ptr(), source.0.as_ptr()) != 0 })
            && unsafe { LMGetKbdType() } as u32 == self.keyboard_type
            && source
                .data()
                .is_ok_and(|bytes| bytes == self.bytes.as_slice())
    }
}

const COMBINED_SESSION_STATE: i32 = 0;

pub fn capture_selection(
    keys: &[i32],
    input_monitoring_allowed: bool,
) -> Result<String, FallbackError> {
    let _capture = CAPTURE_GATE.try_lock().map_err(|_| FallbackError::Busy)?;
    if keys.iter().any(|key| u16::try_from(*key).is_err()) {
        return Err(FallbackError::InvalidTrigger);
    }
    // A denied state query may look like an already released key. Never use
    // that ambiguous false result to authorize synthetic input.
    if !input_monitoring_allowed {
        return Err(FallbackError::InputUnavailable);
    }
    autoreleasepool(|_| {
        // These AnyThread objects are !Send/!Sync. Keep them in this worker;
        // the only data leaving it is the captured Rust string.
        let mut clipboard = MacClipboard {
            board: NSPasteboard::generalPasteboard(),
        };
        let mut keyboard = MacKeyboard {
            enigo: None,
            command_attempted: false,
            copy_key_attempted: None,
            copy_layout: None,
            target_pid: None,
            trigger_keys: keys,
        };
        let mut clock = MonotonicClock(Instant::now());
        capture_transaction(
            &mut clipboard,
            &mut keyboard,
            &mut clock,
            keys,
            TransactionConfig::default(),
        )
    })
}

struct MonotonicClock(Instant);

impl Clock for MonotonicClock {
    fn now(&self) -> Duration {
        self.0.elapsed()
    }

    fn wait(&mut self, duration: Duration) {
        std::thread::sleep(duration);
    }
}

struct MacClipboard {
    board: Retained<NSPasteboard>,
}

fn snapshot_board(
    board: &NSPasteboard,
    byte_budget: usize,
) -> Result<ClipboardSnapshot, FallbackError> {
    let change_count = board.changeCount();
    let mut items_snapshot = Vec::new();
    let mut bytes_used = 0usize;
    if let Some(items) = board.pasteboardItems() {
        if items.count() > MAX_SNAPSHOT_ITEMS {
            return Err(FallbackError::SnapshotTooLarge);
        }
        if items.count() == 0 && board.types().is_some_and(|types| types.count() != 0) {
            return Err(FallbackError::SnapshotIncomplete);
        }
        for item_index in 0..items.count() {
            let item = items.objectAtIndex(item_index);
            let types = item.types();
            if types.count() == 0 {
                return Err(FallbackError::SnapshotIncomplete);
            }
            if types.count() > MAX_TYPES_PER_ITEM {
                return Err(FallbackError::SnapshotTooLarge);
            }
            let mut representations = Vec::new();
            for type_index in 0..types.count() {
                let type_name = types.objectAtIndex(type_index);
                if type_name.length() > MAX_TYPE_NAME_BYTES {
                    return Err(FallbackError::SnapshotTooLarge);
                }
                let type_name_string = type_name.to_string();
                if type_name_string.len() > MAX_TYPE_NAME_BYTES {
                    return Err(FallbackError::SnapshotTooLarge);
                }
                if is_promised_type(&type_name_string) {
                    return Err(FallbackError::SnapshotPromise);
                }
                let data = item
                    .dataForType(&type_name)
                    .ok_or(FallbackError::SnapshotIncomplete)?;
                // Check the NSData length before allocating an owned copy.
                bytes_used = bytes_used
                    .checked_add(data.length())
                    .filter(|total| *total <= byte_budget)
                    .ok_or(FallbackError::SnapshotTooLarge)?;
                representations.push(Representation {
                    type_name: type_name_string,
                    bytes: Zeroizing::new(data.to_vec()),
                });
            }
            // Materialization can change an item's advertised representations
            // without taking ownership. Reject a changed list as incomplete.
            let final_types = item.types();
            if final_types.count() != representations.len()
                || (0..final_types.count()).any(|index| {
                    final_types.objectAtIndex(index).to_string() != representations[index].type_name
                })
            {
                return Err(FallbackError::SnapshotIncomplete);
            }
            items_snapshot.push(representations);
        }
    } else if board.types().is_some_and(|types| types.count() != 0) {
        return Err(FallbackError::SnapshotIncomplete);
    }
    if board.changeCount() != change_count {
        return Err(FallbackError::ClipboardChanged);
    }
    Ok(ClipboardSnapshot {
        change_count,
        items: items_snapshot,
    })
}

fn prepare_items(
    snapshot: &ClipboardSnapshot,
) -> Result<Retained<NSArray<ProtocolObject<dyn NSPasteboardWriting>>>, FallbackError> {
    let mut objects = Vec::new();
    for representations in &snapshot.items {
        let item = NSPasteboardItem::new();
        for representation in representations {
            let type_name = NSString::from_str(&representation.type_name);
            let data = NSData::with_bytes(&representation.bytes);
            if !item.setData_forType(&data, &type_name) {
                return Err(FallbackError::RestoreFailed);
            }
        }
        objects.push(ProtocolObject::from_retained(item));
    }
    Ok(NSArray::from_retained_slice(&objects))
}

impl ClipboardBackend for MacClipboard {
    fn snapshot(&mut self) -> Result<ClipboardSnapshot, FallbackError> {
        snapshot_board(&self.board, MAX_SNAPSHOT_BYTES)
    }

    fn change_count(&mut self) -> Result<isize, FallbackError> {
        Ok(self.board.changeCount())
    }

    fn read_text(&mut self) -> Result<Option<String>, FallbackError> {
        let plain_text = NSString::from_str("public.utf8-plain-text");
        let Some(text) = self.board.stringForType(&plain_text) else {
            return Ok(None);
        };
        if text.length() > MAX_SNAPSHOT_BYTES {
            return Err(FallbackError::ClipboardReadFailed);
        }
        Ok(Some(text.to_string()))
    }

    fn restore(
        &mut self,
        snapshot: &ClipboardSnapshot,
        expected_count: isize,
    ) -> Result<(), FallbackError> {
        // Rebuild everything before clearing. Original NSPasteboardItems are
        // invalid after their owner changes and must never be written back.
        let objects = prepare_items(snapshot)?;
        if self.board.changeCount() != expected_count {
            return Err(FallbackError::ClipboardChanged);
        }
        // Public NSPasteboard offers no atomic expected-count write. There is
        // still a race between this check and clearContents; do not retry.
        let ownership_count = self.board.clearContents();
        if self.board.changeCount() != ownership_count {
            return Err(FallbackError::ClipboardChanged);
        }
        if !snapshot.items.is_empty() && !self.board.writeObjects(&objects) {
            return Err(FallbackError::RestoreFailed);
        }
        Ok(())
    }
}

struct MacKeyboard<'a> {
    enigo: Option<Enigo>,
    command_attempted: bool,
    copy_key_attempted: Option<u16>,
    copy_layout: Option<CopyLayout>,
    target_pid: Option<i32>,
    trigger_keys: &'a [i32],
}

impl MacKeyboard<'_> {
    fn keyboard(&mut self) -> Result<&mut Enigo, FallbackError> {
        if self.enigo.is_none() {
            self.enigo = Some(
                Enigo::new(&Settings {
                    release_keys_when_dropped: true,
                    open_prompt_to_get_permissions: false,
                    independent_of_keyboard_state: true,
                    ..Settings::default()
                })
                .map_err(|_| FallbackError::InputFailed)?,
            );
        }
        self.enigo.as_mut().ok_or(FallbackError::InputFailed)
    }
}

impl CopyKeyboard for MacKeyboard<'_> {
    fn foreground_pid(&mut self) -> Option<i32> {
        let system = axuielement::system_wide()?;
        let application = system.focused_application().ok()??;
        let pid = application.pid().ok().filter(|pid| *pid > 0);
        if self.target_pid.is_none() {
            self.target_pid = pid;
        }
        pid
    }

    fn trigger_keys_released(&mut self, keys: &[i32]) -> Result<bool, FallbackError> {
        if !unsafe { CGPreflightListenEventAccess() } || !axuielement::is_process_trusted() {
            return Err(FallbackError::InputUnavailable);
        }
        let blocked = NSEventModifierFlags::Command
            | NSEventModifierFlags::Control
            | NSEventModifierFlags::Shift
            | NSEventModifierFlags::Option;
        if NSEvent::modifierFlags_class().intersects(blocked) {
            return Ok(false);
        }
        for key in keys {
            let key = u16::try_from(*key).map_err(|_| FallbackError::InvalidTrigger)?;
            if unsafe { CGEventSourceKeyState(COMBINED_SESSION_STATE, key) } {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn prepare_copy(&mut self) -> Result<(), FallbackError> {
        self.copy_layout = Some(CopyLayout::current()?);
        let _ = self.keyboard()?;
        Ok(())
    }

    fn press_command(&mut self) -> Result<(), FallbackError> {
        if !self
            .copy_layout
            .as_ref()
            .is_some_and(CopyLayout::is_current)
        {
            return Err(FallbackError::KeyboardLayoutUnavailable);
        }
        if self.foreground_pid() != self.target_pid {
            return Err(FallbackError::FocusChanged);
        }
        let keys = self.trigger_keys;
        if !self.trigger_keys_released(keys)? {
            return Err(FallbackError::KeysNotReleased);
        }
        self.command_attempted = true;
        self.keyboard()?
            .key(Key::Meta, Direction::Press)
            .map_err(|_| FallbackError::InputFailed)
    }

    fn click_copy(&mut self) -> Result<(), FallbackError> {
        if !self
            .copy_layout
            .as_ref()
            .is_some_and(CopyLayout::is_current)
        {
            return Err(FallbackError::KeyboardLayoutUnavailable);
        }
        if !unsafe { CGPreflightListenEventAccess() } || !axuielement::is_process_trusted() {
            return Err(FallbackError::InputUnavailable);
        }
        if self.foreground_pid() != self.target_pid {
            return Err(FallbackError::FocusChanged);
        }
        press_and_release_copy_key(self)
    }

    fn release_command(&mut self) -> Result<(), FallbackError> {
        if !self.command_attempted {
            return Ok(());
        }
        let result = self.enigo.as_mut().map_or(Ok(()), |keyboard| {
            keyboard
                .key(Key::Meta, Direction::Release)
                .map_err(|_| FallbackError::ReleaseFailed)
        });
        if result.is_ok() {
            self.command_attempted = false;
        }
        result
    }
}

impl CopyKey for MacKeyboard<'_> {
    fn press_copy_key(&mut self) -> Result<(), FallbackError> {
        let code = self
            .copy_layout
            .as_ref()
            .map(|layout| layout.keycode)
            .ok_or(FallbackError::KeyboardLayoutUnavailable)?;
        self.copy_key_attempted = Some(code);
        self.keyboard()?
            .raw(code, Direction::Press)
            .map_err(|_| FallbackError::InputFailed)
    }

    fn release_copy_key(&mut self) -> Result<(), FallbackError> {
        let Some(code) = self.copy_key_attempted else {
            return Ok(());
        };
        let result = self.enigo.as_mut().map_or(Ok(()), |keyboard| {
            keyboard
                .raw(code, Direction::Release)
                .map_err(|_| FallbackError::ReleaseFailed)
        });
        if result.is_ok() {
            self.copy_key_attempted = None;
        }
        result
    }
}

impl Drop for MacKeyboard<'_> {
    fn drop(&mut self) {
        if self.copy_key_attempted.is_some() {
            let _ = self.release_copy_key();
        }
        if self.command_attempted {
            // Retry only key release, never clipboard restoration. Enigo's
            // own release-on-drop remains the last best-effort cleanup.
            let _ = self.release_command();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_default_layout_resolves_a_current_command_copy_key() {
        // Reads only public keyboard-layout metadata; no event, AX query,
        // permission request, or pasteboard access is involved.
        let layout = CopyLayout::current()
            .expect("macOS runner must expose a keyboard layout for this native API check");
        assert!(
            layout.is_current(),
            "runner layout changed during native API check"
        );
        assert!(layout.keycode < 128);
    }

    struct PrivateBoard(Retained<NSPasteboard>);

    impl Drop for PrivateBoard {
        fn drop(&mut self) {
            // The generated bindings skip this `oneway void` method. This
            // board was created with a unique name and is owned by this test.
            unsafe { objc2::msg_send![&*self.0, releaseGlobally] }
        }
    }

    fn fixture(items: &[&[(&str, &[u8])]]) -> ClipboardSnapshot {
        ClipboardSnapshot {
            change_count: 0,
            items: items
                .iter()
                .map(|item| {
                    item.iter()
                        .map(|(type_name, bytes)| Representation {
                            type_name: (*type_name).into(),
                            bytes: Zeroizing::new(bytes.to_vec()),
                        })
                        .collect()
                })
                .collect(),
        }
    }

    fn write_fixture(board: &NSPasteboard, snapshot: &ClipboardSnapshot) {
        let items = prepare_items(snapshot).unwrap();
        board.clearContents();
        if !snapshot.items.is_empty() {
            assert!(board.writeObjects(&items));
        }
    }

    #[test]
    fn private_board_roundtrip_preserves_every_item_type_and_binary_byte() {
        autoreleasepool(|_| {
            let board = PrivateBoard(NSPasteboard::pasteboardWithUniqueName());
            let original = fixture(&[
                &[
                    ("public.utf8-plain-text", b"fixture"),
                    ("public.html", b"<b>fixture</b>"),
                    ("public.rtf", br"{\rtf1\ansi fixture}"),
                ],
                &[
                    ("public.png", include_bytes!("../icons/32x32.png")),
                    ("public.file-url", b"file:///tmp/quickpick-test-fixture.txt"),
                    ("org.quickpick.empty-fixture", &[]),
                ],
            ]);
            write_fixture(&board.0, &original);
            let mut clipboard = MacClipboard {
                board: board.0.clone(),
            };
            let before = clipboard.snapshot().unwrap();
            write_fixture(
                &board.0,
                &fixture(&[&[("public.utf8-plain-text", b"replacement")]]),
            );
            let copied_count = board.0.changeCount();
            clipboard.restore(&before, copied_count).unwrap();
            let restored = clipboard.snapshot().unwrap();
            assert_eq!(restored.items.len(), original.items.len());
            for (actual, expected) in restored.items.iter().zip(&original.items) {
                assert_eq!(actual.len(), expected.len());
                for (actual, expected) in actual.iter().zip(expected) {
                    assert_eq!(actual.type_name, expected.type_name);
                    assert_eq!(actual.bytes.as_slice(), expected.bytes.as_slice());
                }
            }
        });
    }

    #[test]
    fn private_board_empty_snapshot_restores_empty() {
        autoreleasepool(|_| {
            let board = PrivateBoard(NSPasteboard::pasteboardWithUniqueName());
            board.0.clearContents();
            let mut clipboard = MacClipboard {
                board: board.0.clone(),
            };
            let before = clipboard.snapshot().unwrap();
            assert!(before.items.is_empty());
            write_fixture(
                &board.0,
                &fixture(&[&[("public.utf8-plain-text", b"replacement")]]),
            );
            clipboard.restore(&before, board.0.changeCount()).unwrap();
            assert!(clipboard.snapshot().unwrap().items.is_empty());
        });
    }

    #[test]
    fn private_board_rejects_promise_and_budget_before_backup_copy() {
        autoreleasepool(|_| {
            let board = PrivateBoard(NSPasteboard::pasteboardWithUniqueName());
            write_fixture(
                &board.0,
                &fixture(&[&[("com.apple.pasteboard.promised-file-url", b"fixture")]]),
            );
            assert!(matches!(
                snapshot_board(&board.0, MAX_SNAPSHOT_BYTES),
                Err(FallbackError::SnapshotPromise)
            ));
            write_fixture(
                &board.0,
                &fixture(&[&[("public.utf8-plain-text", b"three")]]),
            );
            assert!(matches!(
                snapshot_board(&board.0, 2),
                Err(FallbackError::SnapshotTooLarge)
            ));
        });
    }

    #[test]
    fn private_board_concurrent_owner_is_not_replaced() {
        autoreleasepool(|_| {
            let board = PrivateBoard(NSPasteboard::pasteboardWithUniqueName());
            write_fixture(
                &board.0,
                &fixture(&[&[("public.utf8-plain-text", b"original")]]),
            );
            let mut clipboard = MacClipboard {
                board: board.0.clone(),
            };
            let snapshot = clipboard.snapshot().unwrap();
            let expected_count = board.0.changeCount();
            write_fixture(
                &board.0,
                &fixture(&[&[("public.utf8-plain-text", b"new owner")]]),
            );
            assert_eq!(
                clipboard.restore(&snapshot, expected_count),
                Err(FallbackError::ClipboardChanged)
            );
            assert_eq!(clipboard.read_text().unwrap().as_deref(), Some("new owner"));
        });
    }
}
