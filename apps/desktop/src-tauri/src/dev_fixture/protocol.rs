use serde::Deserialize;
use std::{
    fs,
    io::{BufRead, BufReader},
    net::TcpStream,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU8, Ordering},
    time::Instant,
};
const MAX_REQUEST: usize = 16 * 1024;
pub(super) struct Options {
    pub(super) root: PathBuf,
    pub(super) token: String,
}

pub(super) fn parse_options(args: &[String]) -> Result<Option<Options>, String> {
    if !args
        .iter()
        .any(|arg| arg.starts_with("--fixture-") || arg == "--quickpick-fixture")
    {
        return Ok(None);
    }
    if args.len() != 5
        || args[0] != "--quickpick-fixture"
        || args[1] != "--fixture-root"
        || args[3] != "--fixture-token"
    {
        return Err("invalid fixture arguments".into());
    }
    if args[4].len() != 64 || !args[4].bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("fixture token must contain 64 hexadecimal digits".into());
    }
    Ok(Some(Options {
        root: PathBuf::from(&args[2]),
        token: args[4].to_ascii_lowercase(),
    }))
}

pub(super) fn validate_root(root: &Path, temp: &Path) -> Result<PathBuf, String> {
    if !root.is_absolute()
        || !root
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.starts_with("quickpick-fixture-"))
    {
        return Err("fixture root must be a unique absolute temporary directory".into());
    }
    let metadata = fs::symlink_metadata(root).map_err(|_| "fixture root is unavailable")?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("fixture root must be a real directory".into());
    }
    #[cfg(windows)]
    for ancestor in root.ancestors() {
        use std::os::windows::fs::MetadataExt;
        let metadata = fs::symlink_metadata(ancestor).map_err(|_| "fixture path is unavailable")?;
        // Junctions are directories and may not be reported by is_symlink().
        if metadata.file_attributes() & 0x400 != 0 {
            return Err("fixture path must not contain a reparse point".into());
        }
    }
    let root = root
        .canonicalize()
        .map_err(|_| "fixture root is unavailable")?;
    let temp = temp
        .canonicalize()
        .map_err(|_| "temporary directory is unavailable")?;
    if root == temp
        || !root.starts_with(&temp)
        || fs::read_dir(&root)
            .map_err(|_| "fixture root is unreadable")?
            .next()
            .is_some()
    {
        return Err(
            "fixture root must be empty and contained in the system temporary directory".into(),
        );
    }
    Ok(root)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Request {
    pub(super) token: String,
    pub(super) id: u64,
    pub(super) command: String,
    pub(super) ui_language: Option<String>,
    pub(super) theme_mode: Option<String>,
    pub(super) label: Option<String>,
}

pub(super) fn token_matches(actual: &str, expected: &str) -> bool {
    if actual.len() != expected.len() {
        return false;
    }
    actual
        .bytes()
        .zip(expected.bytes())
        .fold(0_u8, |diff, (a, b)| diff | (a ^ b))
        == 0
}

pub(super) fn read_request(stream: &TcpStream) -> Result<Request, String> {
    read_bounded_request(BufReader::new(stream))
}

fn read_bounded_request(reader: impl BufRead) -> Result<Request, String> {
    let mut bytes = Vec::new();
    // take() bounds allocation even for a peer that never sends a newline.
    let n = reader
        .take((MAX_REQUEST + 1) as u64)
        .read_until(b'\n', &mut bytes)
        .map_err(|_| "fixture request could not be read")?;
    if n == 0 || n > MAX_REQUEST || bytes.last() != Some(&b'\n') {
        return Err("fixture request exceeds its limit or is incomplete".into());
    }
    serde_json::from_slice(&bytes).map_err(|_| "invalid fixture request".into())
}

#[derive(Clone, Copy)]
pub(super) struct Revision {
    pub(super) appearance: u64,
    pub(super) screenshot: u64,
}
impl Revision {
    pub(super) fn matches(self, candidate: Self, overlay: bool) -> bool {
        self.appearance == candidate.appearance
            && (!overlay || self.screenshot == candidate.screenshot)
    }
}

pub(super) struct Pending {
    // 0 queued, 1 started, 2 cancelled. A queued timed-out command never mutates UI.
    state: AtomicU8,
    deadline: Instant,
}

impl Pending {
    pub(super) fn new(deadline: Instant) -> Self {
        Self {
            state: AtomicU8::new(0),
            deadline,
        }
    }
    pub(super) fn claim(&self) -> bool {
        if Instant::now() >= self.deadline {
            self.cancel();
            return false;
        }
        self.state
            .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }
    pub(super) fn cancel(&self) {
        let _ = self
            .state
            .compare_exchange(0, 2, Ordering::AcqRel, Ordering::Acquire);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordinary_arguments_do_not_activate_fixture() {
        assert!(parse_options(&["--autostart".into()]).unwrap().is_none());
        assert!(parse_options(&[]).unwrap().is_none());
    }
    #[test]
    fn fixture_requires_explicit_complete_arguments() {
        assert!(parse_options(&["--fixture-root".into(), "/tmp/x".into()]).is_err());
        let args = vec![
            "--quickpick-fixture".into(),
            "--fixture-root".into(),
            "/tmp/quickpick-fixture-x".into(),
            "--fixture-token".into(),
            "a".repeat(64),
        ];
        assert!(parse_options(&args).unwrap().is_some());
        for token in ["short", &"g".repeat(64)] {
            let mut args = args.clone();
            args[4] = token.into();
            assert!(parse_options(&args).is_err());
        }
    }
    #[test]
    fn control_token_requires_exact_match() {
        assert!(token_matches(&"a".repeat(64), &"a".repeat(64)));
        assert!(!token_matches(&"a".repeat(63), &"a".repeat(64)));
        assert!(!token_matches(&"b".repeat(64), &"a".repeat(64)));
    }
    #[test]
    fn fixture_root_cannot_use_existing_content_or_parent() {
        let temp = std::env::temp_dir();
        let root = temp.join(format!(
            "quickpick-fixture-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        assert!(validate_root(&root, &temp).is_ok());
        fs::write(root.join("user-settings"), b"synthetic").unwrap();
        assert!(validate_root(&root, &temp).is_err());
        assert!(validate_root(&temp, &temp).is_err());
        assert!(validate_root(Path::new("quickpick-fixture-relative"), &temp).is_err());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn protocol_rejects_unknown_fields_and_arbitrary_data() {
        assert!(serde_json::from_str::<Request>(
            r#"{"token":"x","id":1,"command":"show","label":"selection","text":"user data"}"#
        )
        .is_err());
    }

    #[test]
    fn control_requires_a_complete_bounded_json_line() {
        let request = br#"{"token":"test","id":7,"command":"state"}"#;
        assert!(read_bounded_request(std::io::Cursor::new(request)).is_err());
        let mut complete = request.to_vec();
        complete.push(b'\n');
        assert_eq!(
            read_bounded_request(std::io::Cursor::new(complete))
                .unwrap()
                .id,
            7
        );
        let mut oversized = vec![b' '; MAX_REQUEST + 1];
        oversized.push(b'\n');
        assert!(read_bounded_request(std::io::Cursor::new(oversized)).is_err());
    }

    #[test]
    fn timed_out_or_cancelled_requests_never_claim_ui_work() {
        let expired = Pending::new(Instant::now() - std::time::Duration::from_secs(1));
        assert!(!expired.claim());
        let cancelled = Pending::new(Instant::now() + std::time::Duration::from_secs(60));
        cancelled.cancel();
        assert!(!cancelled.claim());
        let ready = Pending::new(Instant::now() + std::time::Duration::from_secs(60));
        assert!(ready.claim());
        assert!(!ready.claim());
    }

    #[test]
    fn stale_navigation_and_screenshot_acknowledgements_are_rejected() {
        let current = Revision {
            appearance: 2,
            screenshot: 3,
        };
        assert!(current.matches(current, true));
        assert!(!current.matches(
            Revision {
                appearance: 1,
                screenshot: 3
            },
            false
        ));
        assert!(!current.matches(
            Revision {
                appearance: 2,
                screenshot: 2
            },
            true
        ));
        assert!(current.matches(
            Revision {
                appearance: 2,
                screenshot: 2
            },
            false
        ));
    }

    #[cfg(unix)]
    #[test]
    fn fixture_root_rejects_symlinks() {
        use std::os::unix::fs::symlink;
        let temp = std::env::temp_dir();
        let root = temp.join(format!(
            "quickpick-fixture-link-test-{}",
            std::process::id()
        ));
        symlink(&temp, &root).unwrap();
        assert!(validate_root(&root, &temp).is_err());
        fs::remove_file(root).unwrap();
    }
}
