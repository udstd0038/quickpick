//! Opt-in Windows tool fixture. Never included in a default application build.
//! Only fixed demonstration content crosses this interface; it is not a user-data API.
mod protocol;
use protocol::{parse_options, read_request, token_matches, validate_root, Pending, Request};
use serde_json::{json, Value};
use std::{
    collections::HashSet,
    fs,
    io::Write,
    net::TcpListener,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex, OnceLock,
    },
    time::{Duration, Instant},
};
use tauri::{Emitter, Manager};

const TIMEOUT: Duration = Duration::from_secs(3);
const SAMPLE: &str = "QuickPick makes translation simple.";
const RESULT: &str = "QuickPick 让翻译更简单。";
static FIXTURE: OnceLock<Fixture> = OnceLock::new();

struct Fixture {
    root: PathBuf,
    token: String,
    build_marker: &'static str,
    loaded: Mutex<HashSet<String>>,
    screenshot_selection_ready: AtomicBool,
    appearance_revision: AtomicU64,
    screenshot_revision: AtomicU64,
    // No real password is read, persisted, encrypted or accepted by this store.
    keys: Mutex<[bool; 3]>,
}

pub fn initialize(build_marker: &'static str) -> Result<(), String> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let Some(options) = parse_options(&args)? else {
        return Err("fixture binary requires explicit fixture arguments".into());
    };
    if !cfg!(windows) {
        return Err("the interactive tool fixture is supported only on Windows".into());
    }
    let root = validate_root(&options.root, &std::env::temp_dir())?;
    fs::create_dir(root.join("webviews"))
        .map_err(|_| "fixture webview directory is unavailable")?;
    FIXTURE
        .set(Fixture {
            root,
            token: options.token,
            build_marker,
            loaded: Mutex::new(HashSet::new()),
            screenshot_selection_ready: AtomicBool::new(false),
            appearance_revision: AtomicU64::new(1),
            screenshot_revision: AtomicU64::new(0),
            keys: Mutex::new([true; 3]),
        })
        .map_err(|_| "fixture was already initialized".into())
}

pub fn active() -> bool {
    FIXTURE.get().is_some()
}

pub fn settings_path() -> Option<PathBuf> {
    FIXTURE.get().map(|f| f.root.join("settings.json"))
}

pub fn defaults() -> crate::app_settings::AppSettings {
    crate::app_settings::AppSettings {
        autostart_enabled: false,
        allow_clipboard_fallback: false,
        ui_language: "zh-Hans".into(),
        theme_mode: "light".into(),
        text_ai_base_url: "https://fixture.invalid/v1".into(),
        text_ai_model: "fixture-text".into(),
        vision_ai_base_url: "https://fixture.invalid/v1".into(),
        vision_ai_model: "fixture-vision".into(),
        input_ai_base_url: "https://fixture.invalid/v1".into(),
        input_ai_model: "fixture-input".into(),
        ..crate::app_settings::AppSettings::default()
    }
}

pub fn configure_context(context: &mut tauri::Context<tauri::Wry>) {
    if let Some(f) = FIXTURE.get() {
        context.config_mut().identifier = format!("com.quickpick.fixture.{}", &f.token[..16]);
        // Config dataDirectory is relative to appDataDir. Construct all windows
        // ourselves so no WebView can briefly open a production profile.
        context.config_mut().app.windows.clear();
    }
}

pub fn configure_window<'a>(
    builder: tauri::WebviewWindowBuilder<'a, tauri::Wry, tauri::AppHandle>,
    label: &str,
) -> tauri::WebviewWindowBuilder<'a, tauri::Wry, tauri::AppHandle> {
    if let Some(f) = FIXTURE.get() {
        builder
            .data_directory(f.root.join("webviews").join(label))
            .transparent(false)
            .background_color(tauri::window::Color(241, 245, 250, 255))
            .additional_browser_args("--disable-gpu --disable-gpu-compositing --disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection")
    } else {
        builder
    }
}

fn revisions(f: &Fixture) -> protocol::Revision {
    protocol::Revision {
        appearance: f.appearance_revision.load(Ordering::Acquire),
        screenshot: f.screenshot_revision.load(Ordering::Acquire),
    }
}

fn query(revision: protocol::Revision) -> String {
    format!(
        "fixtureAppearance={}&fixtureScreenshot={}",
        revision.appearance, revision.screenshot
    )
}

pub fn window_url() -> tauri::WebviewUrl {
    if let Some(f) = FIXTURE.get() {
        tauri::WebviewUrl::App(format!("/?{}", query(revisions(f))).into())
    } else {
        tauri::WebviewUrl::App("/".into())
    }
}

fn refresh_window(window: &tauri::WebviewWindow) -> Result<(), String> {
    let f = FIXTURE.get().ok_or("fixture inactive")?;
    let mut url = window
        .url()
        .map_err(|_| "fixture interface URL unavailable")?;
    url.set_query(Some(&query(revisions(f))));
    window
        .navigate(url)
        .map_err(|_| "fixture interface refresh failed".into())
}

fn key_index(scope: &str) -> usize {
    match scope {
        "vision" => 1,
        "input" => 2,
        _ => 0,
    }
}
pub fn key_status(scope: &str) -> Result<bool, String> {
    FIXTURE
        .get()
        .ok_or("fixture inactive")?
        .keys
        .lock()
        .map(|keys| keys[key_index(scope)])
        .map_err(|_| "fixture key state unavailable".into())
}
pub fn set_key(scope: &str, configured: bool) -> Result<(), String> {
    FIXTURE
        .get()
        .ok_or("fixture inactive")?
        .keys
        .lock()
        .map(|mut keys| keys[key_index(scope)] = configured)
        .map_err(|_| "fixture key state unavailable".into())
}
pub fn ai_result() -> Result<String, String> {
    if active() {
        Ok(RESULT.into())
    } else {
        Err("fixture inactive".into())
    }
}

#[tauri::command(rename_all = "camelCase")]
pub fn fixture_frontend_ready(
    window: tauri::WebviewWindow,
    ready: bool,
    screenshot_selection_ready: Option<bool>,
    appearance_revision: u64,
    screenshot_revision: Option<u64>,
) -> Result<(), String> {
    let f = FIXTURE.get().ok_or("fixture inactive")?;
    let mut loaded = f
        .loaded
        .lock()
        .map_err(|_| "fixture readiness unavailable")?;
    let current = revisions(f);
    let candidate = protocol::Revision {
        appearance: appearance_revision,
        screenshot: screenshot_revision.unwrap_or(0),
    };
    if !current.matches(candidate, window.label() == "screenshot_overlay") {
        return Ok(());
    }
    if ready {
        loaded.insert(window.label().into());
    } else {
        loaded.remove(window.label());
    }
    if window.label() == "screenshot_overlay" {
        f.screenshot_selection_ready.store(
            ready && screenshot_selection_ready.unwrap_or(false),
            Ordering::Release,
        );
    }
    Ok(())
}

pub fn observe_page(webview: &tauri::Webview, payload: &tauri::webview::PageLoadPayload<'_>) {
    if !active() || payload.event() != tauri::webview::PageLoadEvent::Finished {
        return;
    }
    // Observe actual rendered React content and two paint frames, rather than
    // declaring a navigation event to be a successfully loaded user interface.
    let selector = match webview.label() {
        "main" => ".settings-shell",
        "selection" => ".selection-window",
        "input" => ".popup-textarea",
        "result" => ".popup-result-box",
        "screenshot_overlay" => ".screenshot-overlay",
        _ => return,
    };
    let overlay = webview.label() == "screenshot_overlay";
    let Some(f) = FIXTURE.get() else {
        return;
    };
    let current = revisions(f);
    let query = payload
        .url()
        .query_pairs()
        .collect::<std::collections::HashMap<_, _>>();
    let Some(appearance_revision) = query.get("fixtureAppearance").and_then(|v| v.parse().ok())
    else {
        return;
    };
    let Some(screenshot_revision) = query.get("fixtureScreenshot").and_then(|v| v.parse().ok())
    else {
        return;
    };
    if !current.matches(
        protocol::Revision {
            appearance: appearance_revision,
            screenshot: screenshot_revision,
        },
        overlay,
    ) {
        return;
    }
    let Ok(settings) = crate::app_settings::load_app_settings(webview.app_handle()) else {
        return;
    };
    let english = settings.ui_language == "en";
    let (localized_selector, expected_text, attribute) = match webview.label() {
        "main" => (
            ".settings-nav button",
            if english { "General" } else { "通用" },
            "textContent",
        ),
        "selection" => (
            ".selection-window button",
            if english { "Copy" } else { "复制" },
            "textContent",
        ),
        "input" => (
            ".popup-textarea",
            if english {
                "Enter text to translate"
            } else {
                "输入要翻译的文本"
            },
            "placeholder",
        ),
        "result" => (
            ".popup-footer button",
            if english { "Copy" } else { "复制" },
            "textContent",
        ),
        _ => (
            ".screenshot-crop img",
            if english {
                "Current Screen"
            } else {
                "当前屏幕"
            },
            "alt",
        ),
    };
    let theme = settings.theme_mode;
    let script = format!(
        r#"(() => {{
      let previous = '';
      let pending = false;
      const sample = () => {{
        const image = document.querySelector('.screenshot-crop img');
        const label = document.querySelector('{localized_selector}');
        const localized = label && label['{attribute}'].trim() === '{expected_text}';
        const ready = !!document.querySelector('{selector}') && localized && document.documentElement.dataset.themePreference === '{theme}' && (!{overlay} || !!(image && image.complete && image.naturalWidth > 0));
        const screenshotSelectionReady = {overlay} && !!document.querySelector('.screenshot-toolbar');
        return {{ready:!!ready,screenshotSelectionReady}};
      }};
      const check = () => {{
        if (pending) return; pending = true;
        requestAnimationFrame(() => requestAnimationFrame(() => {{
          pending = false;
          const value = sample();
          const key = String(value.ready) + String(value.screenshotSelectionReady);
          if (key === previous) return; previous = key;
          window.__TAURI_INTERNALS__.invoke('fixture_frontend_ready', {{...value,appearanceRevision:{appearance_revision},screenshotRevision:{screenshot_revision}}}).catch(() => {{}});
        }}));
      }};
      const observer = new MutationObserver(check);
      observer.observe(document.documentElement, {{childList:true, subtree:true, attributes:true, characterData:true}});
      document.addEventListener('load',check,true); check();
    }})()"#
    );
    let _ = webview.eval(&script);
}

pub fn start(app: &tauri::AppHandle) -> Result<(), String> {
    let Some(f) = FIXTURE.get() else {
        return Ok(());
    };
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .map_err(|_| "fixture control listener unavailable")?;
    let ready = json!({"protocol":1,"buildMarker":f.build_marker,"pid":std::process::id(),"port":listener.local_addr().map_err(|_| "fixture listener unavailable")?.port(),"token":f.token});
    let path = f.root.join("ready.json");
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(path)
        .map_err(|_| "fixture ready file unavailable")?;
    file.write_all(ready.to_string().as_bytes())
        .map_err(|_| "fixture ready file unavailable")?;
    let app = app.clone();
    std::thread::Builder::new()
        .name("quickpick-fixture-control".into())
        .spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else {
                    break;
                };
                if !stream.peer_addr().is_ok_and(|a| a.ip().is_loopback()) {
                    continue;
                }
                let _ = stream.set_read_timeout(Some(TIMEOUT));
                let _ = stream.set_write_timeout(Some(TIMEOUT));
                let response = match read_request(&stream) {
                    Ok(req) if token_matches(&req.token, &f.token) => {
                        let id = req.id;
                        let handle = app.clone();
                        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
                        let pending = Arc::new(Pending::new(Instant::now() + TIMEOUT));
                        let queued = Arc::clone(&pending);
                        if app
                            .run_on_main_thread(move || {
                                if queued.claim() {
                                    let _ = sender.send(dispatch(&handle, req));
                                }
                            })
                            .is_err()
                        {
                            json!({"id":id,"ok":false,"error":"fixture UI unavailable"})
                        } else {
                            receiver.recv_timeout(TIMEOUT).unwrap_or_else(|_| {
                                pending.cancel();
                                json!({"id":id,"ok":false,"error":"fixture UI timeout"})
                            })
                        }
                    }
                    _ => json!({"ok":false,"error":"fixture request rejected"}),
                };
                let _ = writeln!(stream, "{response}");
            }
        })
        .map_err(|_| "fixture control worker unavailable")?;
    Ok(())
}

fn close_all(app: &tauri::AppHandle) {
    for label in ["main", "selection", "input", "result", "screenshot_overlay"] {
        crate::clear_popup_state_internal(app, label);
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.hide();
        }
    }
}

fn dispatch(app: &tauri::AppHandle, req: Request) -> Value {
    let result: Result<Value, String> = (|| match req.command.as_str() {
        "state" => {
            let f = FIXTURE.get().ok_or("fixture inactive")?;
            let loaded = f
                .loaded
                .lock()
                .map_err(|_| "fixture readiness unavailable")?;
            let windows = app.webview_windows().into_iter().map(|(label,w)| {
                #[cfg(windows)] let hwnd = w.hwnd().map(|h| h.0 as usize as u64).unwrap_or(0);
                #[cfg(not(windows))] let hwnd = 0_u64;
                json!({"label":label,"visible":w.is_visible().unwrap_or(false),"focused":w.is_focused().unwrap_or(false),"loaded":loaded.contains(&label),"hwnd":hwnd})
            }).collect::<Vec<_>>();
            let settings = crate::app_settings::load_app_settings(app)?;
            use tauri_plugin_global_shortcut::GlobalShortcutExt;
            Ok(
                json!({"screenshotBlurBehavior":"hide","windows":windows,"appearanceRevision":f.appearance_revision.load(Ordering::Acquire),"screenshotRevision":f.screenshot_revision.load(Ordering::Acquire),"screenshotSelectionReady":f.screenshot_selection_ready.load(Ordering::Acquire),"hotkeys":{
                "settings":app.global_shortcut().is_registered(settings.settings_hotkey.as_str()),
                "selection":app.global_shortcut().is_registered(settings.selection_hotkey.as_str()),
                "screenshot":app.global_shortcut().is_registered(settings.screenshot_hotkey.as_str()),
                "input":app.global_shortcut().is_registered(settings.input_translate_hotkey.as_str())}}),
            )
        }
        "configure" => {
            let language = req
                .ui_language
                .as_deref()
                .ok_or("fixture language missing")?;
            let theme = req.theme_mode.as_deref().ok_or("fixture theme missing")?;
            if !matches!(language, "zh-Hans" | "en") || !matches!(theme, "light" | "dark") {
                return Err("unsupported fixture appearance".into());
            }
            close_all(app);
            let mut settings = crate::app_settings::load_app_settings(app)?;
            settings.ui_language = language.into();
            settings.theme_mode = theme.into();
            settings.autostart_enabled = false;
            settings.allow_clipboard_fallback = false;
            crate::save_app_settings(app.clone(), settings)?;
            let f = FIXTURE.get().ok_or("fixture inactive")?;
            let mut loaded = f
                .loaded
                .lock()
                .map_err(|_| "fixture readiness unavailable")?;
            let revision = f.appearance_revision.fetch_add(1, Ordering::AcqRel) + 1;
            loaded.clear();
            drop(loaded);
            f.screenshot_selection_ready.store(false, Ordering::Release);
            for window in app.webview_windows().into_values() {
                refresh_window(&window)?;
            }
            Ok(json!({"appearanceRevision":revision}))
        }
        "close_all" => {
            close_all(app);
            Ok(json!({}))
        }
        "show" => {
            close_all(app);
            match req.label.as_deref() {
                Some("main") => crate::show_settings_window(app),
                Some("selection") => show_selection(app),
                Some("input") => show_input(app),
                Some("result") => show_result(app),
                _ => return Err("unsupported fixture window".into()),
            }
            Ok(json!({}))
        }
        "quit" => {
            close_all(app);
            app.exit(0);
            Ok(json!({}))
        }
        _ => Err("unsupported fixture command".into()),
    })();
    match result {
        Ok(Value::Object(mut fields)) => {
            fields.insert("id".into(), json!(req.id));
            fields.insert("ok".into(), json!(true));
            Value::Object(fields)
        }
        Ok(_) => json!({"id":req.id,"ok":true}),
        Err(error) => json!({"id":req.id,"ok":false,"error":error}),
    }
}

pub fn show_selection(app: &tauri::AppHandle) {
    let snapshot = crate::selection::SelectionSnapshot {
        status: crate::selection::SelectionStatus::Captured,
        text: SAMPLE.into(),
        preview: SAMPLE.into(),
        char_count: SAMPLE.chars().count(),
        message: "Synthetic tool fixture".into(),
        source: "fixture".into(),
    };
    if let Ok(mut state) = app.state::<crate::AppState>().selection_snapshot.lock() {
        *state = snapshot;
    }
    let _ = crate::show_selection_webview(app);
}

pub fn show_input(app: &tauri::AppHandle) {
    let _ = crate::show_input_webview(app, "auto".into(), "zh-Hans".into(), "forward".into());
    let payload = crate::InputReadyPayload {
        status: "success".into(),
        content: RESULT.into(),
        detail: String::new(),
        source_language: "auto".into(),
        target_language: "zh-Hans".into(),
        direction: "forward".into(),
    };
    if let Ok(mut state) = app.state::<crate::AppState>().last_input_payload.lock() {
        *state = Some(payload.clone());
    }
    if let Ok(mut state) = app.state::<crate::AppState>().last_input_result.lock() {
        *state = Some(RESULT.into());
    }
    let _ = app.emit_to("input", "input-ready", payload);
    if let Some(window) = app.get_webview_window("input") {
        // React's setter/event pair updates only this fixture's fixed sample.
        let _ = window.eval(r#"(() => {const e=document.querySelector('textarea');if(e){Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype,'value').set.call(e,'QuickPick makes translation simple.');e.dispatchEvent(new Event('input',{bubbles:true}));}})()"#);
    }
}

pub fn show_result(app: &tauri::AppHandle) {
    let Ok(snapshot) = crate::ResultSnapshot::text_success(
        "translate",
        SAMPLE.into(),
        SAMPLE.chars().count(),
        RESULT.into(),
        "en".into(),
        "zh-Hans".into(),
        "forward".into(),
    ) else {
        return;
    };
    let _ = crate::set_app_result_snapshot(app, snapshot.clone());
    crate::show_webview_result_snapshot(app, &snapshot);
}

pub fn show_screenshot(app: &tauri::AppHandle) {
    if let Some(f) = FIXTURE.get() {
        if let Ok(mut loaded) = f.loaded.lock() {
            loaded.remove("screenshot_overlay");
            f.screenshot_revision.fetch_add(1, Ordering::AcqRel);
            f.screenshot_selection_ready.store(false, Ordering::Release);
        }
    }
    let state = app.state::<crate::AppState>();
    let Ok(mut store) = state.screenshot_store.lock() else {
        return;
    };
    let previous = store.active_id();
    let Ok(id) = store.begin() else {
        return;
    };
    drop(store);
    if let Some(id) = previous {
        crate::emit_screenshot_closed(app, id);
    }
    // Fixed dimensions/content, no display enumeration or desktop capture.
    let result = crate::screenshot::fixture_screenshot(id);
    crate::finish_screenshot_capture(app, id, result);
    if let Some(window) = app.get_webview_window("screenshot_overlay") {
        // A fresh tagged DOM discards old crops. The page reads the retained frame.
        let _ = refresh_window(&window);
    }
}
