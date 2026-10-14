//! Clipboard fallback policy. Native I/O is supplied by platform adapters so
//! failure and concurrency decisions can be tested without reading user data.

use std::time::Duration;
use zeroize::Zeroizing;

pub const MAX_SNAPSHOT_ITEMS: usize = 64;
pub const MAX_TYPES_PER_ITEM: usize = 128;
pub const MAX_SNAPSHOT_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_TYPE_NAME_BYTES: usize = 1024;

// Deliberately neither Debug nor Serialize: snapshots contain user content.
pub struct Representation {
    pub type_name: String,
    pub bytes: Zeroizing<Vec<u8>>,
}

pub struct ClipboardSnapshot {
    pub change_count: isize,
    pub items: Vec<Vec<Representation>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FallbackError {
    ClipboardAccessDenied,
    Busy,
    InvalidTrigger,
    SnapshotIncomplete,
    SnapshotTooLarge,
    SnapshotPromise,
    ClipboardChanged,
    ClipboardReadFailed,
    CopyTimedOut,
    EmptyCopiedText,
    KeysNotReleased,
    InputUnavailable,
    KeyboardLayoutUnavailable,
    FocusChanged,
    InputFailed,
    ReleaseFailed,
    RestoreFailed,
    ReleaseAndRestoreFailed,
    ReleaseAndClipboardChanged,
}

pub trait CopyKey {
    fn press_copy_key(&mut self) -> Result<(), FallbackError>;
    fn release_copy_key(&mut self) -> Result<(), FallbackError>;
}

pub fn press_and_release_copy_key<K: CopyKey>(keyboard: &mut K) -> Result<(), FallbackError> {
    let pressed = keyboard.press_copy_key();
    let released = keyboard.release_copy_key();
    if released.is_err() {
        Err(FallbackError::ReleaseFailed)
    } else {
        pressed
    }
}

pub fn select_command_copy_key<F: FnMut(u16) -> Option<u16>>(mut translate: F) -> Option<u16> {
    (0..128).find(|key| matches!(translate(*key), Some(0x63 | 0x43)))
}

fn cleanup_error(input_error: Option<FallbackError>, error: FallbackError) -> FallbackError {
    if input_error == Some(FallbackError::ReleaseFailed) {
        match error {
            FallbackError::RestoreFailed => FallbackError::ReleaseAndRestoreFailed,
            FallbackError::ClipboardChanged => FallbackError::ReleaseAndClipboardChanged,
            _ => error,
        }
    } else {
        error
    }
}

pub trait ClipboardBackend {
    fn snapshot(&mut self) -> Result<ClipboardSnapshot, FallbackError>;
    fn change_count(&mut self) -> Result<isize, FallbackError>;
    fn read_text(&mut self) -> Result<Option<String>, FallbackError>;
    /// Construct the entire replacement before mutating the board, and check
    /// expected_count immediately before writing. This is not an atomic CAS.
    fn restore(
        &mut self,
        snapshot: &ClipboardSnapshot,
        expected_count: isize,
    ) -> Result<(), FallbackError>;
}

pub trait CopyKeyboard {
    fn foreground_pid(&mut self) -> Option<i32>;
    fn trigger_keys_released(&mut self, keys: &[i32]) -> Result<bool, FallbackError>;
    /// Prepare native input/layout resources without posting any key events.
    fn prepare_copy(&mut self) -> Result<(), FallbackError> {
        Ok(())
    }
    fn press_command(&mut self) -> Result<(), FallbackError>;
    fn click_copy(&mut self) -> Result<(), FallbackError>;
    fn release_command(&mut self) -> Result<(), FallbackError>;
}

pub trait Clock {
    fn now(&self) -> Duration;
    fn wait(&mut self, duration: Duration);
}

pub struct TransactionConfig {
    pub release_timeout: Duration,
    pub copy_timeout: Duration,
    pub poll_interval: Duration,
}

impl Default for TransactionConfig {
    fn default() -> Self {
        Self {
            release_timeout: Duration::from_secs(1),
            copy_timeout: Duration::from_secs(1),
            poll_interval: Duration::from_millis(10),
        }
    }
}

pub fn is_promised_type(type_name: &str) -> bool {
    // A byte copy cannot retain an external file-promise provider. Conservatively
    // skip promise representations instead of converting them to inert bytes.
    type_name.to_ascii_lowercase().contains("promise")
}

pub fn capture_selection<C: ClipboardBackend, K: CopyKeyboard, T: Clock>(
    clipboard: &mut C,
    keyboard: &mut K,
    clock: &mut T,
    keys: &[i32],
    config: TransactionConfig,
) -> Result<String, FallbackError> {
    if keys.iter().any(|key| u16::try_from(*key).is_err()) {
        return Err(FallbackError::InvalidTrigger);
    }
    let target_pid = keyboard
        .foreground_pid()
        .filter(|pid| *pid > 0)
        .ok_or(FallbackError::FocusChanged)?;
    let started = clock.now();
    loop {
        if keyboard.foreground_pid() != Some(target_pid) {
            return Err(FallbackError::FocusChanged);
        }
        if keyboard.trigger_keys_released(keys)? {
            break;
        }
        if clock.now().saturating_sub(started) >= config.release_timeout {
            return Err(FallbackError::KeysNotReleased);
        }
        clock.wait(config.poll_interval.max(Duration::from_millis(1)));
    }

    let snapshot = clipboard.snapshot()?;
    keyboard.prepare_copy()?;
    if clipboard.change_count()? != snapshot.change_count {
        return Err(FallbackError::ClipboardChanged);
    }
    if keyboard.foreground_pid() != Some(target_pid) {
        return Err(FallbackError::FocusChanged);
    }
    // Snapshotting may materialize lazy data and take time. Check the keys
    // again so a second shortcut or newly held primary cannot become Cmd+Q etc.
    if !keyboard.trigger_keys_released(keys)? {
        return Err(FallbackError::KeysNotReleased);
    }
    if clipboard.change_count()? != snapshot.change_count {
        return Err(FallbackError::ClipboardChanged);
    }

    let pressed = keyboard.press_command();
    let copy_attempted = pressed.is_ok();
    let copied = if copy_attempted {
        keyboard.click_copy()
    } else {
        Err(FallbackError::InputFailed)
    };
    // Never short-circuit this release, even if a press or click failed.
    let released = keyboard.release_command();
    let input_error = if released.is_err() || copied == Err(FallbackError::ReleaseFailed) {
        Some(FallbackError::ReleaseFailed)
    } else {
        pressed.err().or(copied.err())
    };
    if !copy_attempted
        || matches!(
            copied,
            Err(FallbackError::KeyboardLayoutUnavailable
                | FallbackError::ClipboardAccessDenied
                | FallbackError::InputUnavailable
                | FallbackError::FocusChanged
                | FallbackError::KeysNotReleased)
        )
    {
        return Err(input_error.unwrap_or(FallbackError::InputFailed));
    }

    // A click may have posted events before returning an error. Keep the
    // snapshot and perform the same bounded observation/cleanup on that path.
    let started = clock.now();
    let mut observed_count = None;
    let result = loop {
        let count = match clipboard.change_count() {
            Ok(count) => count,
            Err(error) => break Err(error),
        };
        if let Some(observed) = observed_count {
            if count != observed {
                // A second writer owns the board now. Do not restore over it.
                return Err(cleanup_error(input_error, FallbackError::ClipboardChanged));
            }
        } else if count != snapshot.change_count {
            observed_count = Some(count);
        }

        if observed_count.is_some() {
            if let Some(error) = input_error {
                break Err(error);
            }
            if keyboard.foreground_pid() != Some(target_pid) {
                break Err(FallbackError::FocusChanged);
            }
            let text = match clipboard.read_text() {
                Ok(text) => text,
                Err(error) => break Err(error),
            };
            if clipboard.change_count()? != count {
                return Err(FallbackError::ClipboardChanged);
            }
            if keyboard.foreground_pid() != Some(target_pid) {
                break Err(FallbackError::FocusChanged);
            }
            if let Some(text) = text.filter(|text| !text.trim().is_empty()) {
                break Ok(text.trim().to_string());
            }
        }
        if clock.now().saturating_sub(started) >= config.copy_timeout {
            break Err(input_error.unwrap_or(if observed_count.is_some() {
                FallbackError::EmptyCopiedText
            } else {
                FallbackError::CopyTimedOut
            }));
        }
        clock.wait(config.poll_interval.max(Duration::from_millis(1)));
    };

    if let Some(count) = observed_count {
        if clipboard.change_count()? != count {
            return Err(cleanup_error(input_error, FallbackError::ClipboardChanged));
        }
        // Restoration failure is an error even if text was successfully read.
        if let Err(error) = clipboard.restore(&snapshot, count) {
            return Err(cleanup_error(input_error, error));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct State {
        now: Duration,
        count: isize,
        text: Option<String>,
        copy_at: Option<Duration>,
        copied_text: Option<String>,
        copied: bool,
        release_at: Duration,
        focus_change_at: Option<Duration>,
        reads: usize,
        snapshots: usize,
        restores: usize,
        key_calls: Vec<&'static str>,
        snapshot_error: Option<FallbackError>,
        read_error: bool,
        restore_error: bool,
        race_on_read: bool,
        race_on_restore: bool,
        press_error: bool,
        click_error: bool,
        release_error: bool,
        input_denied: bool,
        layout_unavailable: bool,
        layout_changes_before_copy: bool,
        copy_release_error: bool,
        permission_revoked_during_prepare: bool,
        key_checks: usize,
        race_after_key_recheck: bool,
        restored_items: Vec<Vec<(String, Vec<u8>)>>,
        empty_original: bool,
    }

    impl Default for State {
        fn default() -> Self {
            Self {
                now: Duration::ZERO,
                count: 5,
                text: Some("old text".into()),
                copy_at: Some(Duration::from_millis(20)),
                copied_text: Some("selected text".into()),
                copied: false,
                release_at: Duration::ZERO,
                focus_change_at: None,
                reads: 0,
                snapshots: 0,
                restores: 0,
                key_calls: vec![],
                snapshot_error: None,
                read_error: false,
                restore_error: false,
                race_on_read: false,
                race_on_restore: false,
                press_error: false,
                click_error: false,
                release_error: false,
                input_denied: false,
                layout_unavailable: false,
                layout_changes_before_copy: false,
                copy_release_error: false,
                permission_revoked_during_prepare: false,
                key_checks: 0,
                race_after_key_recheck: false,
                restored_items: vec![],
                empty_original: false,
            }
        }
    }

    struct FakeClipboard(Rc<RefCell<State>>);
    struct FakeKeyboard(Rc<RefCell<State>>);
    struct FakeClock(Rc<RefCell<State>>);

    impl ClipboardBackend for FakeClipboard {
        fn snapshot(&mut self) -> Result<ClipboardSnapshot, FallbackError> {
            let mut state = self.0.borrow_mut();
            state.snapshots += 1;
            if let Some(error) = state.snapshot_error {
                return Err(error);
            }
            let items = if state.empty_original {
                vec![]
            } else {
                vec![
                    vec![
                        Representation {
                            type_name: "public.utf8-plain-text".into(),
                            bytes: Zeroizing::new(b"old text".to_vec()),
                        },
                        Representation {
                            type_name: "public.html".into(),
                            bytes: Zeroizing::new(b"<b>old text</b>".to_vec()),
                        },
                    ],
                    vec![Representation {
                        type_name: "public.png".into(),
                        bytes: Zeroizing::new(vec![0, 1, 255]),
                    }],
                ]
            };
            Ok(ClipboardSnapshot {
                change_count: state.count,
                items,
            })
        }

        fn change_count(&mut self) -> Result<isize, FallbackError> {
            let mut state = self.0.borrow_mut();
            if !state.copied
                && state.key_calls.contains(&"copy")
                && state.copy_at.is_some_and(|time| state.now >= time)
            {
                state.copied = true;
                state.count += 1;
                state.text = state.copied_text.clone();
            }
            Ok(state.count)
        }

        fn read_text(&mut self) -> Result<Option<String>, FallbackError> {
            let mut state = self.0.borrow_mut();
            state.reads += 1;
            if state.race_on_read {
                state.count += 1;
                state.text = Some("concurrent user copy".into());
            }
            if state.read_error {
                Err(FallbackError::ClipboardReadFailed)
            } else {
                Ok(state.text.clone())
            }
        }

        fn restore(
            &mut self,
            snapshot: &ClipboardSnapshot,
            expected_count: isize,
        ) -> Result<(), FallbackError> {
            let mut state = self.0.borrow_mut();
            if state.race_on_restore {
                state.count += 1;
                state.text = Some("concurrent user copy".into());
            }
            if state.count != expected_count {
                return Err(FallbackError::ClipboardChanged);
            }
            if state.restore_error {
                return Err(FallbackError::RestoreFailed);
            }
            state.restores += 1;
            state.restored_items = snapshot
                .items
                .iter()
                .map(|item| {
                    item.iter()
                        .map(|representation| {
                            (
                                representation.type_name.clone(),
                                representation.bytes.to_vec(),
                            )
                        })
                        .collect()
                })
                .collect();
            state.text = if snapshot.items.is_empty() {
                None
            } else {
                Some("old text".into())
            };
            state.count += 1;
            Ok(())
        }
    }

    impl CopyKeyboard for FakeKeyboard {
        fn prepare_copy(&mut self) -> Result<(), FallbackError> {
            let mut state = self.0.borrow_mut();
            if state.permission_revoked_during_prepare {
                state.input_denied = true;
            }
            Ok(())
        }
        fn foreground_pid(&mut self) -> Option<i32> {
            let state = self.0.borrow();
            Some(
                if state.focus_change_at.is_some_and(|time| state.now >= time) {
                    99
                } else {
                    42
                },
            )
        }
        fn trigger_keys_released(&mut self, _keys: &[i32]) -> Result<bool, FallbackError> {
            let mut state = self.0.borrow_mut();
            state.key_checks += 1;
            if state.race_after_key_recheck && state.key_checks == 2 {
                state.count += 1;
                state.text = Some("concurrent user copy".into());
            }
            if state.input_denied {
                Err(FallbackError::InputUnavailable)
            } else {
                Ok(state.now >= state.release_at)
            }
        }
        fn press_command(&mut self) -> Result<(), FallbackError> {
            let mut state = self.0.borrow_mut();
            if state.layout_unavailable {
                return Err(FallbackError::KeyboardLayoutUnavailable);
            }
            state.key_calls.push("press");
            if state.press_error {
                Err(FallbackError::InputFailed)
            } else {
                Ok(())
            }
        }
        fn click_copy(&mut self) -> Result<(), FallbackError> {
            let mut state = self.0.borrow_mut();
            if state.layout_changes_before_copy {
                return Err(FallbackError::KeyboardLayoutUnavailable);
            }
            state.key_calls.push("copy");
            if state.copy_release_error {
                return Err(FallbackError::ReleaseFailed);
            }
            if state.click_error {
                Err(FallbackError::InputFailed)
            } else {
                Ok(())
            }
        }
        fn release_command(&mut self) -> Result<(), FallbackError> {
            let mut state = self.0.borrow_mut();
            state.key_calls.push("release");
            if state.release_error {
                Err(FallbackError::ReleaseFailed)
            } else {
                Ok(())
            }
        }
    }
    impl Clock for FakeClock {
        fn now(&self) -> Duration {
            self.0.borrow().now
        }
        fn wait(&mut self, duration: Duration) {
            self.0.borrow_mut().now += duration;
        }
    }

    fn run(state: &Rc<RefCell<State>>, keys: &[i32]) -> Result<String, FallbackError> {
        capture_selection(
            &mut FakeClipboard(state.clone()),
            &mut FakeKeyboard(state.clone()),
            &mut FakeClock(state.clone()),
            keys,
            TransactionConfig {
                release_timeout: Duration::from_millis(50),
                copy_timeout: Duration::from_millis(50),
                poll_interval: Duration::from_millis(10),
            },
        )
    }
    fn state() -> Rc<RefCell<State>> {
        Rc::new(RefCell::new(State::default()))
    }

    #[test]
    fn reads_new_copy_then_restores_every_item_and_type() {
        let state = state();
        assert_eq!(run(&state, &[12]), Ok("selected text".into()));
        let state = state.borrow();
        assert_eq!(state.key_calls, ["press", "copy", "release"]);
        assert_eq!(state.restores, 1);
        assert_eq!(
            state.restored_items,
            vec![
                vec![
                    ("public.utf8-plain-text".into(), b"old text".to_vec()),
                    ("public.html".into(), b"<b>old text</b>".to_vec())
                ],
                vec![("public.png".into(), vec![0, 1, 255])]
            ]
        );
    }

    #[test]
    fn no_copy_change_never_reads_old_text_or_restores() {
        let state = state();
        state.borrow_mut().copy_at = None;
        assert_eq!(run(&state, &[12]), Err(FallbackError::CopyTimedOut));
        let state = state.borrow();
        assert_eq!(state.reads, 0);
        assert_eq!(state.restores, 0);
    }

    #[test]
    fn delayed_copy_after_eighty_ms_is_captured_before_the_new_deadline() {
        let state = state();
        state.borrow_mut().copy_at = Some(Duration::from_millis(120));
        let result = capture_selection(
            &mut FakeClipboard(state.clone()),
            &mut FakeKeyboard(state.clone()),
            &mut FakeClock(state.clone()),
            &[12],
            TransactionConfig {
                release_timeout: Duration::from_millis(50),
                copy_timeout: Duration::from_millis(300),
                poll_interval: Duration::from_millis(10),
            },
        );
        assert_eq!(result, Ok("selected text".into()));
        assert_eq!(state.borrow().now, Duration::from_millis(120));
        assert_eq!(state.borrow().restores, 1);
    }

    #[test]
    fn identical_text_is_valid_when_the_owner_changed() {
        let state = state();
        state.borrow_mut().copied_text = Some("old text".into());
        assert_eq!(run(&state, &[12]), Ok("old text".into()));
        assert_eq!(state.borrow().restores, 1);
    }

    #[test]
    fn empty_original_board_is_restored_as_empty() {
        let state = state();
        state.borrow_mut().empty_original = true;
        assert_eq!(run(&state, &[12]), Ok("selected text".into()));
        let state = state.borrow();
        assert_eq!(state.restores, 1);
        assert!(state.restored_items.is_empty());
        assert_eq!(state.text, None);
    }

    #[test]
    fn incomplete_or_unrestorable_snapshot_never_sends_keys() {
        for error in [
            FallbackError::ClipboardAccessDenied,
            FallbackError::SnapshotIncomplete,
            FallbackError::SnapshotTooLarge,
            FallbackError::SnapshotPromise,
        ] {
            let state = state();
            state.borrow_mut().snapshot_error = Some(error);
            assert_eq!(run(&state, &[12]), Err(error));
            let state = state.borrow();
            assert!(state.key_calls.is_empty());
            assert_eq!(state.restores, 0);
        }
    }

    #[test]
    fn unrecognized_trigger_never_touches_the_board() {
        for keys in [vec![-1], vec![65_536]] {
            let state = state();
            assert_eq!(run(&state, &keys), Err(FallbackError::InvalidTrigger));
            let state = state.borrow();
            assert_eq!(state.snapshots, 0);
            assert!(state.key_calls.is_empty());
        }
    }

    #[test]
    fn tray_entry_without_primary_still_checks_input_and_captures() {
        let state = state();
        assert_eq!(run(&state, &[]), Ok("selected text".into()));
        assert_eq!(state.borrow().restores, 1);
    }

    #[test]
    fn input_permission_and_held_primary_fail_before_snapshot() {
        let denied = state();
        denied.borrow_mut().input_denied = true;
        assert_eq!(run(&denied, &[12]), Err(FallbackError::InputUnavailable));
        assert_eq!(denied.borrow().snapshots, 0);
        let held = state();
        held.borrow_mut().release_at = Duration::from_millis(100);
        assert_eq!(run(&held, &[12]), Err(FallbackError::KeysNotReleased));
        assert!(held.borrow().key_calls.is_empty());
        assert_eq!(held.borrow().snapshots, 0);
    }

    #[test]
    fn waits_for_primary_before_pressing_command() {
        let state = state();
        state.borrow_mut().release_at = Duration::from_millis(30);
        assert_eq!(run(&state, &[12]), Ok("selected text".into()));
        assert!(state.borrow().now >= Duration::from_millis(30));
    }

    #[test]
    fn command_release_is_attempted_after_press_and_click_errors() {
        let press = state();
        press.borrow_mut().press_error = true;
        assert_eq!(run(&press, &[12]), Err(FallbackError::InputFailed));
        assert_eq!(press.borrow().key_calls, ["press", "release"]);
        let click = state();
        click.borrow_mut().click_error = true;
        assert_eq!(run(&click, &[12]), Err(FallbackError::InputFailed));
        assert_eq!(click.borrow().key_calls, ["press", "copy", "release"]);
        assert_eq!(click.borrow().restores, 1);
    }

    #[test]
    fn release_and_restore_failures_are_never_captured() {
        let release = state();
        release.borrow_mut().release_error = true;
        assert_eq!(run(&release, &[12]), Err(FallbackError::ReleaseFailed));
        assert_eq!(release.borrow().restores, 1);
        let restore = state();
        restore.borrow_mut().restore_error = true;
        assert_eq!(run(&restore, &[12]), Err(FallbackError::RestoreFailed));
        let both = state();
        {
            let mut state = both.borrow_mut();
            state.release_error = true;
            state.restore_error = true;
        }
        assert_eq!(
            run(&both, &[12]),
            Err(FallbackError::ReleaseAndRestoreFailed)
        );
    }

    #[test]
    fn unavailable_layout_does_not_inject_or_observe_a_copy() {
        let state = state();
        state.borrow_mut().layout_unavailable = true;
        assert_eq!(
            run(&state, &[12]),
            Err(FallbackError::KeyboardLayoutUnavailable)
        );
        let state = state.borrow();
        assert_eq!(state.key_calls, ["release"]);
        assert_eq!(state.reads, 0);
        assert_eq!(state.restores, 0);
    }

    #[test]
    fn copy_key_release_is_attempted_even_when_its_press_partially_fails() {
        struct FakeKey {
            calls: Vec<&'static str>,
            press_failed: bool,
            release_failed: bool,
        }
        impl CopyKey for FakeKey {
            fn press_copy_key(&mut self) -> Result<(), FallbackError> {
                self.calls.push("c-down");
                if self.press_failed {
                    Err(FallbackError::InputFailed)
                } else {
                    Ok(())
                }
            }
            fn release_copy_key(&mut self) -> Result<(), FallbackError> {
                self.calls.push("c-up");
                if self.release_failed {
                    Err(FallbackError::ReleaseFailed)
                } else {
                    Ok(())
                }
            }
        }
        for (press_failed, release_failed, expected) in [
            (false, false, Ok(())),
            (true, false, Err(FallbackError::InputFailed)),
            (false, true, Err(FallbackError::ReleaseFailed)),
            (true, true, Err(FallbackError::ReleaseFailed)),
        ] {
            let mut key = FakeKey {
                calls: vec![],
                press_failed,
                release_failed,
            };
            assert_eq!(press_and_release_copy_key(&mut key), expected);
            assert_eq!(key.calls, ["c-down", "c-up"]);
        }
    }

    #[test]
    fn command_layout_mapping_never_defaults_to_ansi_a() {
        assert_eq!(select_command_copy_key(|_| None), None);
        assert_eq!(select_command_copy_key(|_| Some(b'a' as u16)), None);
        assert_eq!(
            select_command_copy_key(|key| if key == 8 {
                Some(b'c' as u16)
            } else {
                Some(b'a' as u16)
            }),
            Some(8)
        );
        assert_eq!(
            select_command_copy_key(|key| if key == 34 { Some(b'C' as u16) } else { None }),
            Some(34)
        );
    }

    #[test]
    fn concurrent_copy_during_read_or_restore_is_kept() {
        for during_read in [true, false] {
            let state = state();
            if during_read {
                state.borrow_mut().race_on_read = true;
            } else {
                state.borrow_mut().race_on_restore = true;
            }
            assert_eq!(run(&state, &[12]), Err(FallbackError::ClipboardChanged));
            let state = state.borrow();
            assert_eq!(state.restores, 0);
            assert_eq!(state.text.as_deref(), Some("concurrent user copy"));
        }
    }

    #[test]
    fn concurrent_copy_after_snapshot_key_recheck_prevents_synthetic_copy() {
        let state = state();
        state.borrow_mut().race_after_key_recheck = true;
        assert_eq!(run(&state, &[12]), Err(FallbackError::ClipboardChanged));
        let state = state.borrow();
        assert!(state.key_calls.is_empty());
        assert_eq!(state.restores, 0);
        assert_eq!(state.text.as_deref(), Some("concurrent user copy"));
    }

    #[test]
    fn non_text_copy_and_read_failure_restore_original_content() {
        let empty = state();
        empty.borrow_mut().copied_text = None;
        assert_eq!(run(&empty, &[12]), Err(FallbackError::EmptyCopiedText));
        assert_eq!(empty.borrow().restores, 1);
        let read = state();
        read.borrow_mut().read_error = true;
        assert_eq!(run(&read, &[12]), Err(FallbackError::ClipboardReadFailed));
        assert_eq!(read.borrow().restores, 1);
    }

    #[test]
    fn focus_change_while_waiting_aborts_without_snapshot_or_keys() {
        let state = state();
        {
            let mut value = state.borrow_mut();
            value.release_at = Duration::from_millis(30);
            value.focus_change_at = Some(Duration::from_millis(10));
        }
        assert_eq!(run(&state, &[12]), Err(FallbackError::FocusChanged));
        let state = state.borrow();
        assert_eq!(state.snapshots, 0);
        assert!(state.key_calls.is_empty());
    }

    #[test]
    fn focus_change_after_copy_restores_but_does_not_return_text() {
        let state = state();
        state.borrow_mut().focus_change_at = Some(Duration::from_millis(20));
        assert_eq!(run(&state, &[12]), Err(FallbackError::FocusChanged));
        assert_eq!(state.borrow().restores, 1);
    }

    #[test]
    fn copy_key_release_error_is_preserved_and_command_is_still_released() {
        let state = state();
        state.borrow_mut().copy_release_error = true;
        assert_eq!(run(&state, &[12]), Err(FallbackError::ReleaseFailed));
        assert_eq!(state.borrow().key_calls, ["press", "copy", "release"]);
        assert_eq!(state.borrow().restores, 1);
    }

    #[test]
    fn permission_revocation_during_prepare_does_not_allow_stale_key_state() {
        let state = state();
        state.borrow_mut().permission_revoked_during_prepare = true;
        assert_eq!(run(&state, &[12]), Err(FallbackError::InputUnavailable));
        assert!(state.borrow().key_calls.is_empty());
        assert_eq!(state.borrow().restores, 0);
    }

    #[test]
    fn layout_change_before_copy_releases_command_without_reading_or_restoring() {
        let state = state();
        state.borrow_mut().layout_changes_before_copy = true;
        assert_eq!(
            run(&state, &[12]),
            Err(FallbackError::KeyboardLayoutUnavailable)
        );
        let state = state.borrow();
        assert_eq!(state.key_calls, ["press", "release"]);
        assert_eq!(state.reads, 0);
        assert_eq!(state.restores, 0);
    }

    #[test]
    fn release_failure_and_concurrent_copy_are_both_reported() {
        let state = state();
        {
            let mut state = state.borrow_mut();
            state.release_error = true;
            state.race_on_restore = true;
        }
        assert_eq!(
            run(&state, &[12]),
            Err(FallbackError::ReleaseAndClipboardChanged)
        );
        let state = state.borrow();
        assert_eq!(state.restores, 0);
        assert_eq!(state.text.as_deref(), Some("concurrent user copy"));
    }

    #[test]
    fn promise_formats_are_rejected_without_rejecting_regular_data() {
        for name in [
            "com.apple.pasteboard.promised-file-url",
            "NSFilesPromisePboardType",
            "com.apple.NSFilePromiseItemMetaData",
        ] {
            assert!(is_promised_type(name));
        }
        assert!(!is_promised_type("public.png"));
        assert!(!is_promised_type("public.file-url"));
    }
}
