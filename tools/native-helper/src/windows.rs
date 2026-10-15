//! Interactive Windows acceptance uses an explicitly requested, isolated dev-tools build.
#![cfg_attr(not(windows), allow(dead_code))]
//! No existing QuickPick process, user clipboard, desktop pixels or real configuration is used.
use std::path::PathBuf;

pub struct DriverError {
    pub code: i32,
    pub message: String,
}

impl DriverError {
    fn failed(message: impl Into<String>) -> Self {
        Self {
            code: 1,
            message: message.into(),
        }
    }
    fn pending(message: impl Into<String>) -> Self {
        Self {
            code: 2,
            message: format!("未执行：{}", message.into()),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Mode {
    Verify,
    Capture,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OverlayBlurBehavior {
    Hide,
    Keep,
}

fn overlay_blur_behavior(value: Option<&str>) -> Result<OverlayBlurBehavior, DriverError> {
    match value {
        Some("hide") => Ok(OverlayBlurBehavior::Hide),
        Some("keep") => Ok(OverlayBlurBehavior::Keep),
        _ => Err(DriverError::failed("fixture 缺少明确的截图失焦行为契约")),
    }
}

#[derive(Debug)]
struct Options {
    mode: Mode,
    fixture_exe: PathBuf,
    output_dir: PathBuf,
    themes: Vec<String>,
    languages: Vec<String>,
    include_screenshot: bool,
}

fn choices(value: &str, allowed: &[&str]) -> Result<Vec<String>, DriverError> {
    let mut result = Vec::new();
    for choice in value.split(',') {
        if !allowed.contains(&choice) || result.iter().any(|old| old == choice) {
            return Err(DriverError::failed("主题／语言参数不合法或重复"));
        }
        result.push(choice.to_owned());
    }
    Ok(result)
}

fn parse_options(args: &[String]) -> Result<Options, DriverError> {
    let mode = match args.first().map(String::as_str) {
        Some("verify-hotkeys") => Mode::Verify,
        Some("capture-readme") => Mode::Capture,
        _ => {
            return Err(DriverError::failed(
                "需要 verify-hotkeys 或 capture-readme 命令",
            ))
        }
    };
    let mut fixture_exe = None;
    let mut output_dir = None;
    let mut themes = None;
    let mut languages = None;
    let mut include_screenshot = false;
    let mut index = 1;
    while index < args.len() {
        let flag = &args[index];
        if flag == "--help" || flag == "-h" {
            return Err(DriverError::pending("用法：--fixture-exe <quickpick-fixture.exe> [--output-dir <目录>] [--themes light,dark] [--languages zh-Hans,en] [--include-screenshot]；只能使用独立 fixture 构建"));
        }
        if flag == "--include-screenshot" {
            if include_screenshot {
                return Err(DriverError::failed("重复的 --include-screenshot"));
            }
            include_screenshot = true;
            index += 1;
            continue;
        }
        let value = args
            .get(index + 1)
            .filter(|value| !value.starts_with("--"))
            .ok_or_else(|| DriverError::failed(format!("缺少参数值：{flag}")))?;
        match flag.as_str() {
            "--fixture-exe" if fixture_exe.is_none() => fixture_exe = Some(PathBuf::from(value)),
            "--output-dir" if output_dir.is_none() => output_dir = Some(PathBuf::from(value)),
            "--themes" if themes.is_none() => themes = Some(choices(value, &["light", "dark"])?),
            "--languages" if languages.is_none() => {
                languages = Some(choices(value, &["zh-Hans", "en"])?)
            }
            _ => return Err(DriverError::failed(format!("未知或重复参数：{flag}"))),
        }
        index += 2;
    }
    Ok(Options {
        mode,
        fixture_exe: fixture_exe.ok_or_else(|| {
            DriverError::pending("必须显式提供 --fixture-exe，不能使用正在运行的正式应用")
        })?,
        output_dir: output_dir.unwrap_or_else(|| PathBuf::from(".")),
        themes: themes.unwrap_or_else(|| vec!["light".into(), "dark".into()]),
        languages: languages.unwrap_or_else(|| vec!["zh-Hans".into(), "en".into()]),
        include_screenshot,
    })
}

fn screenshot_name(language: &str, theme: &str, label: &str) -> Result<PathBuf, DriverError> {
    let directory = match language {
        "zh-Hans" => "zh-cn",
        "en" => "en",
        _ => return Err(DriverError::failed("语言不合法")),
    };
    if !["light", "dark"].contains(&theme) {
        return Err(DriverError::failed("主题不合法"));
    }
    let name = match label {
        "main" => "settings",
        "selection" => "selection",
        "input" => "input",
        "screenshot_overlay" => "screenshot",
        _ => return Err(DriverError::failed("截图窗口不合法")),
    };
    Ok(PathBuf::from("screenshots")
        .join(directory)
        .join(format!("{name}-{theme}.png")))
}

// A successful PrintWindow return is insufficient: WebView/GPU windows can yield blank pixels.
fn validate_pixels(rgb: &[u8], width: u32, height: u32) -> Result<(), DriverError> {
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|value| value.checked_mul(3));
    if width < 60
        || height < 40
        || width > 8192
        || height > 8192
        || expected != Some(rgb.len())
        || rgb.len() > 128 * 1024 * 1024
    {
        return Err(DriverError::failed("窗口像素尺寸不合法"));
    }
    let count = rgb.len() / 3;
    let mut black = 0;
    let mut sum = 0u64;
    let mut square = 0u64;
    let mut colors = std::collections::HashSet::new();
    for pixel in rgb.chunks_exact(3) {
        let light = (u32::from(pixel[0]) + u32::from(pixel[1]) + u32::from(pixel[2])) / 3;
        black += usize::from(light <= 2);
        sum += u64::from(light);
        square += u64::from(light * light);
        if colors.len() < 32 {
            colors.insert([pixel[0], pixel[1], pixel[2]]);
        }
    }
    let variance = square as f64 / count as f64 - (sum as f64 / count as f64).powi(2);
    if black as f64 / count as f64 > 0.98 || colors.len() < 12 || variance < 2.0 {
        return Err(DriverError::pending("PrintWindow 未得到可辨认的应用内容；请使用支持软件渲染的交互式 Windows 桌面，未保存空白截图"));
    }
    Ok(())
}

const FIXTURE_MARKER: &str = "QUICKPICK_DEVTOOLS_FIXTURE_V1_6B8A5D3E";

// Scan bounded chunks, retaining the overlap so a marker spanning two reads is found.
// This runs before spawning, including when someone passes a production executable by mistake.
fn has_fixture_marker(mut executable: impl std::io::Read) -> std::io::Result<bool> {
    let needle = FIXTURE_MARKER.as_bytes();
    let mut chunk = [0u8; 64 * 1024];
    let mut overlap = Vec::new();
    let mut total = 0usize;
    loop {
        let count = executable.read(&mut chunk)?;
        if count == 0 {
            return Ok(false);
        }
        total += count;
        if total > 512 * 1024 * 1024 {
            return Ok(false);
        }
        overlap.extend_from_slice(&chunk[..count]);
        if overlap.windows(needle.len()).any(|window| window == needle) {
            return Ok(true);
        }
        if overlap.len() > needle.len() - 1 {
            overlap.drain(..overlap.len() - (needle.len() - 1));
        }
    }
}

pub fn run(args: &[String]) -> Result<(), DriverError> {
    let options = parse_options(args)?;
    #[cfg(windows)]
    {
        native::run(options)
    }
    #[cfg(not(windows))]
    {
        let _ = options;
        Err(DriverError::pending(
            "此验收入口需要交互式 Windows 桌面；跨平台编译不代表已经执行热键／截图验收",
        ))
    }
}

#[cfg(any(windows, test))]
trait KeyWriter {
    fn send(&mut self, key: u16, release: bool) -> Result<(), DriverError>;
}

#[cfg(any(windows, test))]
struct HeldKeys<'a> {
    keys: Vec<u16>,
    writer: &'a mut dyn KeyWriter,
}

#[cfg(any(windows, test))]
impl<'a> HeldKeys<'a> {
    fn new(writer: &'a mut dyn KeyWriter) -> Self {
        Self {
            keys: Vec::new(),
            writer,
        }
    }
    fn press(&mut self, key: u16) -> Result<(), DriverError> {
        self.writer.send(key, false)?;
        self.keys.push(key);
        Ok(())
    }
    fn release(&mut self, key: u16) -> Result<(), DriverError> {
        self.writer.send(key, true)?;
        self.keys.retain(|held| *held != key);
        Ok(())
    }
}

#[cfg(any(windows, test))]
impl Drop for HeldKeys<'_> {
    fn drop(&mut self) {
        for key in self.keys.iter().rev() {
            let _ = self.writer.send(*key, true);
        }
    }
}

#[cfg(windows)]
mod native {
    use super::*;
    use image::{imageops::FilterType, RgbImage};
    use serde_json::{json, Value};
    use std::{
        fs,
        io::{BufRead, BufReader, Read, Write},
        mem::{size_of, zeroed},
        net::{Ipv4Addr, SocketAddr, TcpStream},
        os::windows::io::AsRawHandle,
        path::Path,
        process::{Child, Command, Stdio},
        ptr::{null, null_mut},
        sync::mpsc,
        thread,
        time::{Duration, Instant},
    };
    use windows_sys::Win32::{
        Foundation::{CloseHandle, HANDLE, HWND, RECT},
        Graphics::Gdi::{
            ClientToScreen, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject,
            SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
        },
        Security::Cryptography::{BCryptGenRandom, BCRYPT_USE_SYSTEM_PREFERRED_RNG},
        Storage::Xps::{PrintWindow, PW_CLIENTONLY},
        System::{
            JobObjects::{
                AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
                SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
                JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            },
            StationsAndDesktops::{
                CloseDesktop, GetProcessWindowStation, GetUserObjectInformationW, OpenInputDesktop,
                DESKTOP_READOBJECTS, DESKTOP_SWITCHDESKTOP, DESKTOP_WRITEOBJECTS, UOI_FLAGS,
                UOI_NAME, USEROBJECTFLAGS,
            },
        },
        UI::{
            HiDpi::{
                AreDpiAwarenessContextsEqual, GetThreadDpiAwarenessContext,
                SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
            },
            Input::KeyboardAndMouse::{
                GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE,
                KEYBDINPUT, KEYEVENTF_KEYUP, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN,
                MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_VIRTUALDESK, MOUSEINPUT,
                VK_CONTROL, VK_ESCAPE, VK_LBUTTON, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
            },
            WindowsAndMessaging::{
                CreateWindowExW, DestroyWindow, DispatchMessageW, GetClientRect,
                GetForegroundWindow, GetSystemMetrics, GetWindowThreadProcessId, IsWindow,
                IsWindowVisible, PeekMessageW, SetForegroundWindow, ShowWindow, TranslateMessage,
                MSG, PM_REMOVE, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
                SM_YVIRTUALSCREEN, SW_SHOW, WSF_VISIBLE, WS_OVERLAPPEDWINDOW,
            },
        },
    };

    const RPC_LIMIT: u64 = 64 * 1024;
    const OP_TIMEOUT: Duration = Duration::from_secs(8);

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(Some(0)).collect()
    }

    fn pump() {
        unsafe {
            let mut message: MSG = zeroed();
            while PeekMessageW(&mut message, null_mut(), 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
    }

    fn pause(duration: Duration) {
        pump();
        thread::sleep(duration);
        pump();
    }

    fn interactive_desktop() -> Result<(), DriverError> {
        unsafe {
            let desktop = OpenInputDesktop(
                0,
                0,
                DESKTOP_READOBJECTS | DESKTOP_WRITEOBJECTS | DESKTOP_SWITCHDESKTOP,
            );
            if desktop.is_null() {
                return Err(DriverError::pending(
                    "无法打开输入桌面（锁屏、服务或无交互 runner）",
                ));
            }
            let mut name = [0u16; 256];
            let mut required = 0u32;
            let result = GetUserObjectInformationW(
                desktop,
                UOI_NAME,
                name.as_mut_ptr().cast(),
                size_of::<[u16; 256]>() as u32,
                &mut required,
            );
            CloseDesktop(desktop);
            let length = name
                .iter()
                .position(|value| *value == 0)
                .unwrap_or(name.len());
            if result == 0
                || !String::from_utf16_lossy(&name[..length]).eq_ignore_ascii_case("Default")
            {
                return Err(DriverError::pending("当前输入桌面不是可用的 Default 桌面"));
            }
            let mut flags: USEROBJECTFLAGS = zeroed();
            if GetUserObjectInformationW(
                GetProcessWindowStation(),
                UOI_FLAGS,
                (&mut flags as *mut USEROBJECTFLAGS).cast(),
                size_of::<USEROBJECTFLAGS>() as u32,
                &mut required,
            ) == 0
                || flags.dwFlags & WSF_VISIBLE as u32 == 0
                || GetForegroundWindow().is_null()
            {
                return Err(DriverError::pending("没有可交互窗口站或前台窗口"));
            }
            SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
            if AreDpiAwarenessContextsEqual(
                GetThreadDpiAwarenessContext(),
                DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
            ) == 0
            {
                return Err(DriverError::pending(
                    "无法启用物理像素 DPI 坐标，不能可靠进行 owned 窗口拖选",
                ));
            }
        }
        Ok(())
    }

    fn random_token() -> Result<String, DriverError> {
        let mut bytes = [0u8; 32];
        if unsafe {
            BCryptGenRandom(
                null_mut(),
                bytes.as_mut_ptr(),
                bytes.len() as u32,
                BCRYPT_USE_SYSTEM_PREFERRED_RNG,
            )
        } < 0
        {
            return Err(DriverError::failed("无法生成独立验收令牌"));
        }
        Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
    }

    struct OwnedJob(HANDLE);
    impl OwnedJob {
        fn create() -> Result<Self, DriverError> {
            let handle = unsafe { CreateJobObjectW(null(), null()) };
            if handle.is_null() {
                return Err(DriverError::pending("无法建立独立 fixture 进程作业"));
            }
            let job = Self(handle);
            let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
            limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            if unsafe {
                SetInformationJobObject(
                    handle,
                    JobObjectExtendedLimitInformation,
                    (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                    size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                )
            } == 0
            {
                return Err(DriverError::pending(
                    "系统拒绝 fixture 作业的自动退出清理约束",
                ));
            }
            Ok(job)
        }
    }
    impl Drop for OwnedJob {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }

    struct Fixture {
        child: Child,
        job: Option<OwnedJob>,
        root: PathBuf,
        token: String,
        port: u16,
        next_id: u64,
        appearance_revision: Option<u64>,
    }

    impl Fixture {
        fn start(executable: &Path) -> Result<Self, DriverError> {
            let executable = executable.canonicalize().map_err(|_| {
                DriverError::pending(
                    "指定 fixture 可执行文件不存在；请先构建 --features dev-tools 的独立应用",
                )
            })?;
            if !executable.is_file() {
                return Err(DriverError::failed("fixture 可执行文件不是普通文件"));
            }
            let binary = fs::File::open(&executable)
                .map_err(|_| DriverError::failed("无法读取 fixture 可执行文件"))?;
            if binary
                .metadata()
                .map_err(|_| DriverError::failed("无法检查 fixture 文件尺寸"))?
                .len()
                > 512 * 1024 * 1024
                || !has_fixture_marker(binary)
                    .map_err(|_| DriverError::failed("fixture marker 检查失败"))?
            {
                return Err(DriverError::pending(
                    "指定文件缺少独立 dev-tools fixture 标记，未启动；普通发行程序不能用于此工具",
                ));
            }
            let job = OwnedJob::create()?;
            let token = random_token()?;
            let root = std::env::temp_dir().join(format!("quickpick-fixture-{token}"));
            fs::create_dir(&root).map_err(|_| DriverError::failed("无法创建独占临时验收目录"))?;
            let child = Command::new(executable)
                .arg("--quickpick-fixture")
                .arg("--fixture-root")
                .arg(&root)
                .arg("--fixture-token")
                .arg(&token)
                .env(
                    "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
                    "--disable-gpu --disable-gpu-compositing",
                )
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
            let child = match child {
                Ok(child) => child,
                Err(_) => {
                    let _ = fs::remove_dir(&root);
                    return Err(DriverError::failed("无法启动独立 fixture 进程"));
                }
            };
            let mut fixture = Self {
                child,
                job: Some(job),
                root,
                token,
                port: 0,
                next_id: 0,
                appearance_revision: None,
            };
            if unsafe {
                AssignProcessToJobObject(
                    fixture.job.as_ref().unwrap().0,
                    fixture.child.as_raw_handle(),
                )
            } == 0
            {
                return Err(DriverError::pending(
                    "不能将本次启动的 fixture 纳入独立退出作业；已停止 owned 进程",
                ));
            }
            let deadline = Instant::now() + Duration::from_secs(40);
            loop {
                if fixture
                    .child
                    .try_wait()
                    .map_err(|_| DriverError::failed("无法查询 owned fixture 进程"))?
                    .is_some()
                {
                    return Err(DriverError::failed(
                        "fixture 在握手前退出；只支持 dev-tools 构建",
                    ));
                }
                if let Ok(file) = fs::File::open(fixture.root.join("ready.json")) {
                    let mut bytes = Vec::new();
                    file.take(16 * 1024 + 1)
                        .read_to_end(&mut bytes)
                        .map_err(|_| DriverError::failed("读取 fixture 握手失败"))?;
                    if bytes.len() > 16 * 1024 {
                        return Err(DriverError::failed("fixture 握手过大"));
                    }
                    if let Ok(ready) = serde_json::from_slice::<Value>(&bytes) {
                        let port = ready["port"]
                            .as_u64()
                            .filter(|value| *value > 0 && *value <= u16::MAX as u64);
                        if ready["protocol"] != 1
                            || ready["pid"].as_u64() != Some(u64::from(fixture.child.id()))
                            || ready["token"].as_str() != Some(fixture.token.as_str())
                            || ready["buildMarker"].as_str() != Some(FIXTURE_MARKER)
                            || port.is_none()
                        {
                            return Err(DriverError::failed("fixture 握手的协议／PID／令牌不匹配"));
                        }
                        fixture.port = port.unwrap() as u16;
                        return Ok(fixture);
                    }
                }
                if Instant::now() >= deadline {
                    return Err(DriverError::pending("fixture WebView 启动或握手超时"));
                }
                pause(Duration::from_millis(100));
            }
        }

        fn rpc(&mut self, command: &str, fields: Value) -> Result<Value, DriverError> {
            if self
                .child
                .try_wait()
                .map_err(|_| DriverError::failed("无法查询 fixture 状态"))?
                .is_some()
            {
                return Err(DriverError::failed("owned fixture 已退出"));
            }
            self.next_id += 1;
            let mut request = fields;
            let object = request
                .as_object_mut()
                .ok_or_else(|| DriverError::failed("内部协议字段不是对象"))?;
            object.insert("token".into(), json!(self.token));
            object.insert("id".into(), json!(self.next_id));
            object.insert("command".into(), json!(command));
            let address = SocketAddr::from((Ipv4Addr::LOCALHOST, self.port));
            let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(3))
                .map_err(|_| DriverError::failed("fixture 控制连接失败"))?;
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .map_err(|_| DriverError::failed("无法设置协议时限"))?;
            stream
                .set_write_timeout(Some(Duration::from_secs(3)))
                .map_err(|_| DriverError::failed("无法设置协议时限"))?;
            let mut bytes = serde_json::to_vec(&request)
                .map_err(|_| DriverError::failed("fixture 请求编码失败"))?;
            if bytes.len() as u64 >= RPC_LIMIT {
                return Err(DriverError::failed("fixture 请求过大"));
            }
            bytes.push(b'\n');
            stream
                .write_all(&bytes)
                .map_err(|_| DriverError::failed("fixture 请求发送失败"))?;
            let mut response = Vec::new();
            BufReader::new(stream.take(RPC_LIMIT + 1))
                .read_until(b'\n', &mut response)
                .map_err(|_| DriverError::failed("fixture 响应超时或读取失败"))?;
            if response.len() as u64 > RPC_LIMIT || response.last() != Some(&b'\n') {
                return Err(DriverError::failed("fixture 响应格式／大小不合法"));
            }
            let value: Value = serde_json::from_slice(&response)
                .map_err(|_| DriverError::failed("fixture 响应不是 JSON"))?;
            if value["id"].as_u64() != Some(self.next_id) || value["ok"] != true {
                return Err(DriverError::failed(format!(
                    "fixture 命令未完成：{command}"
                )));
            }
            Ok(value)
        }

        fn windows(&mut self) -> Result<Vec<WindowState>, DriverError> {
            let state = self.rpc("state", json!({}))?;
            if state["appearanceRevision"].as_u64() != self.appearance_revision {
                return Err(DriverError::failed("fixture 外观版本与本轮配置不符"));
            }
            let windows = state["windows"]
                .as_array()
                .ok_or_else(|| DriverError::failed("缺少窗口状态"))?;
            let mut result = Vec::new();
            for row in windows {
                let label = row["label"]
                    .as_str()
                    .ok_or_else(|| DriverError::failed("窗口 label 不合法"))?
                    .to_owned();
                if !["main", "selection", "input", "result", "screenshot_overlay"]
                    .contains(&label.as_str())
                    || result
                        .iter()
                        .any(|window: &WindowState| window.label == label)
                {
                    return Err(DriverError::failed("窗口 label 重复或超出 fixture 范围"));
                }
                let hwnd = row["hwnd"]
                    .as_u64()
                    .filter(|value| *value > 0 && *value <= usize::MAX as u64)
                    .ok_or_else(|| DriverError::failed("窗口 HWND 不合法"))?
                    as usize;
                owned_window(self.child.id(), hwnd)?;
                result.push(WindowState {
                    label,
                    hwnd,
                    visible: row["visible"]
                        .as_bool()
                        .ok_or_else(|| DriverError::failed("visible 状态不合法"))?,
                    focused: row["focused"]
                        .as_bool()
                        .ok_or_else(|| DriverError::failed("focused 状态不合法"))?,
                    loaded: row["loaded"]
                        .as_bool()
                        .ok_or_else(|| DriverError::failed("loaded 状态不合法"))?,
                });
            }
            Ok(result)
        }

        fn registered(&mut self) -> Result<(), DriverError> {
            let state = self.rpc("state", json!({}))?;
            self.appearance_revision = Some(
                state["appearanceRevision"]
                    .as_u64()
                    .filter(|value| *value > 0)
                    .ok_or_else(|| DriverError::failed("fixture 缺少外观版本"))?,
            );
            for name in ["settings", "selection", "screenshot", "input"] {
                if state["hotkeys"][name] != true {
                    return Err(DriverError::failed(format!(
                        "fixture 快捷键未注册或被占用：{name}；不能通过伪造热键消息验收"
                    )));
                }
            }
            Ok(())
        }

        fn configure(&mut self, language: &str, theme: &str) -> Result<(), DriverError> {
            let response = self.rpc(
                "configure",
                json!({"uiLanguage": language, "themeMode": theme}),
            )?;
            let revision = response["appearanceRevision"]
                .as_u64()
                .filter(|revision| {
                    self.appearance_revision
                        .map(|old| *revision > old)
                        .unwrap_or(false)
                })
                .ok_or_else(|| DriverError::failed("fixture 外观配置没有返回新的版本"))?;
            self.appearance_revision = Some(revision);
            Ok(())
        }

        fn close_all(&mut self) -> Result<(), DriverError> {
            self.rpc("close_all", json!({}))?;
            let deadline = Instant::now() + OP_TIMEOUT;
            loop {
                if self.windows()?.iter().all(|window| !window.visible) {
                    return Ok(());
                }
                if Instant::now() >= deadline {
                    return Err(DriverError::failed("fixture 窗口未全部隐藏"));
                }
                pause(Duration::from_millis(80));
            }
        }

        fn wait_window(&mut self, label: &str) -> Result<WindowState, DriverError> {
            let deadline = Instant::now() + OP_TIMEOUT;
            loop {
                let windows = self.windows()?;
                if let Some(window) = windows.into_iter().find(|window| {
                    window.label == label && window.visible && window.focused && window.loaded
                }) {
                    if unsafe {
                        IsWindowVisible(window.hwnd as HWND) != 0
                            && GetForegroundWindow() == window.hwnd as HWND
                    } {
                        return Ok(window);
                    }
                }
                if Instant::now() >= deadline {
                    return Err(DriverError::failed(format!(
                        "真实热键未使指定窗口可见、就绪且获得焦点：{label}"
                    )));
                }
                pause(Duration::from_millis(80));
            }
        }

        fn wait_hidden(&mut self, label: &str) -> Result<(), DriverError> {
            let deadline = Instant::now() + OP_TIMEOUT;
            loop {
                if let Some(window) = self
                    .windows()?
                    .into_iter()
                    .find(|window| window.label == label)
                {
                    if !window.visible && unsafe { IsWindowVisible(window.hwnd as HWND) == 0 } {
                        return Ok(());
                    }
                }
                if Instant::now() >= deadline {
                    return Err(DriverError::failed(format!(
                        "失焦／Esc 后指定窗口未隐藏：{label}"
                    )));
                }
                pause(Duration::from_millis(80));
            }
        }

        fn wait_retained_after_blur(&mut self, label: &str) -> Result<WindowState, DriverError> {
            let deadline = Instant::now() + OP_TIMEOUT;
            let mut unfocused_since = None;
            loop {
                let window = self
                    .windows()?
                    .into_iter()
                    .find(|window| window.label == label)
                    .ok_or_else(|| DriverError::failed("失焦后截图窗口身份丢失"))?;
                if !window.visible || unsafe { IsWindowVisible(window.hwnd as HWND) == 0 } {
                    return Err(DriverError::failed(
                        "主线截图窗口失焦后没有保留既有可见状态",
                    ));
                }
                if !window.focused
                    && window.loaded
                    && unsafe { GetForegroundWindow() != window.hwnd as HWND }
                {
                    let since = unfocused_since.get_or_insert_with(Instant::now);
                    if since.elapsed() >= Duration::from_millis(500) {
                        return Ok(window);
                    }
                } else {
                    unfocused_since = None;
                }
                if Instant::now() >= deadline {
                    return Err(DriverError::failed("截图窗口没有稳定失去焦点"));
                }
                pause(Duration::from_millis(80));
            }
        }

        fn wait_screenshot_selection(&mut self) -> Result<(), DriverError> {
            let deadline = Instant::now() + OP_TIMEOUT;
            loop {
                let state = self.rpc("state", json!({}))?;
                if state["appearanceRevision"].as_u64() != self.appearance_revision {
                    return Err(DriverError::failed("截图拖选属于旧外观版本"));
                }
                if state["screenshotSelectionReady"] == true {
                    return Ok(());
                }
                if Instant::now() >= deadline {
                    return Err(DriverError::failed(
                        "真实合成拖选后未渲染截图功能条，未生成 README 截图",
                    ));
                }
                pause(Duration::from_millis(80));
            }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            if self.port != 0 {
                let _ = self.rpc("quit", json!({}));
            }
            let deadline = Instant::now() + Duration::from_secs(2);
            while Instant::now() < deadline {
                if self.child.try_wait().ok().flatten().is_some() {
                    break;
                }
                thread::sleep(Duration::from_millis(50));
            }
            // Closing this private job stops only our child/tree, including its WebView workers.
            // Every wait is bounded; Child::wait is never called on a possibly live process.
            drop(self.job.take());
            if self.child.try_wait().ok().flatten().is_none() {
                let _ = self.child.kill();
            }
            let deadline = Instant::now() + Duration::from_secs(2);
            while Instant::now() < deadline {
                if self.child.try_wait().ok().flatten().is_some() {
                    break;
                }
                thread::sleep(Duration::from_millis(50));
            }
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    struct WindowState {
        label: String,
        hwnd: usize,
        visible: bool,
        focused: bool,
        loaded: bool,
    }

    fn owned_window(pid: u32, hwnd: usize) -> Result<(), DriverError> {
        unsafe {
            let mut actual_pid = 0;
            GetWindowThreadProcessId(hwnd as HWND, &mut actual_pid);
            if IsWindow(hwnd as HWND) == 0 || actual_pid != pid {
                return Err(DriverError::failed("窗口不属于本次启动的 fixture PID"));
            }
        }
        Ok(())
    }

    struct SourceWindow(HWND);
    impl SourceWindow {
        fn create() -> Result<Self, DriverError> {
            let class = wide("STATIC");
            let title = wide("QuickPick isolated acceptance source — synthetic text only");
            let hwnd = unsafe {
                CreateWindowExW(
                    0,
                    class.as_ptr(),
                    title.as_ptr(),
                    WS_OVERLAPPEDWINDOW,
                    120,
                    120,
                    640,
                    360,
                    null_mut(),
                    null_mut(),
                    null_mut(),
                    null(),
                )
            };
            if hwnd.is_null() {
                return Err(DriverError::pending("无法创建独立合成输入窗口"));
            }
            Ok(Self(hwnd))
        }
        fn focus(&self) -> Result<(), DriverError> {
            unsafe {
                ShowWindow(self.0, SW_SHOW);
                SetForegroundWindow(self.0);
            }
            pause(Duration::from_millis(150));
            if unsafe { GetForegroundWindow() } != self.0 {
                return Err(DriverError::pending(
                    "系统拒绝前台切换，不能可靠发送真实快捷键",
                ));
            }
            Ok(())
        }
    }
    impl Drop for SourceWindow {
        fn drop(&mut self) {
            unsafe {
                DestroyWindow(self.0);
            }
        }
    }

    fn idle_keys(extra: u16) -> Result<(), DriverError> {
        for key in [VK_MENU, VK_CONTROL, VK_SHIFT, VK_LWIN, VK_RWIN, extra] {
            if unsafe { GetAsyncKeyState(i32::from(key)) } < 0 {
                return Err(DriverError::pending("检测到用户正按住组合键，请松开后重试"));
            }
        }
        Ok(())
    }

    fn key_input(key: u16, release: bool) -> Result<(), DriverError> {
        let input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: key,
                    wScan: 0,
                    dwFlags: if release { KEYEVENTF_KEYUP } else { 0 },
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        if unsafe { SendInput(1, &input, size_of::<INPUT>() as i32) } != 1 {
            return Err(DriverError::pending(
                "SendInput 被桌面／权限边界阻止；未验收热键注册",
            ));
        }
        Ok(())
    }

    struct NativeKeyWriter;
    impl KeyWriter for NativeKeyWriter {
        fn send(&mut self, key: u16, release: bool) -> Result<(), DriverError> {
            key_input(key, release)
        }
    }

    fn chord(source: &SourceWindow, digit: u8) -> Result<(), DriverError> {
        source.focus()?;
        let digit = u16::from(digit);
        idle_keys(digit)?;
        let mut writer = NativeKeyWriter;
        let mut held = HeldKeys::new(&mut writer);
        held.press(VK_MENU)?;
        held.press(digit)?;
        pause(Duration::from_millis(60));
        // Both releases are attempted even if the first one fails; Drop retries owned keys.
        let digit_release = held.release(digit);
        let alt_release = held.release(VK_MENU);
        digit_release?;
        alt_release?;
        Ok(())
    }

    fn escape(window: &WindowState, pid: u32) -> Result<(), DriverError> {
        owned_window(pid, window.hwnd)?;
        if unsafe { GetForegroundWindow() } != window.hwnd as HWND {
            return Err(DriverError::pending("Esc 目标窗口已失去焦点"));
        }
        idle_keys(VK_ESCAPE)?;
        let mut writer = NativeKeyWriter;
        let mut held = HeldKeys::new(&mut writer);
        held.press(VK_ESCAPE)?;
        held.release(VK_ESCAPE)
    }

    fn client_rect(pid: u32, hwnd: usize) -> Result<(u32, u32), DriverError> {
        owned_window(pid, hwnd)?;
        let mut rect: RECT = unsafe { zeroed() };
        if unsafe { GetClientRect(hwnd as HWND, &mut rect) } == 0 {
            return Err(DriverError::failed("读取 owned 窗口尺寸失败"));
        }
        let width = u32::try_from(rect.right - rect.left)
            .map_err(|_| DriverError::failed("窗口宽度不合法"))?;
        let height = u32::try_from(rect.bottom - rect.top)
            .map_err(|_| DriverError::failed("窗口高度不合法"))?;
        if width == 0
            || height == 0
            || width > 8192
            || height > 8192
            || u64::from(width) * u64::from(height) * 4 > 128 * 1024 * 1024
        {
            return Err(DriverError::failed("窗口尺寸超过捕获预算"));
        }
        Ok((width, height))
    }

    fn mouse_input(x: i32, y: i32, flags: u32) -> Result<(), DriverError> {
        let left = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
        let top = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };
        let width = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
        let height = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };
        if width <= 1
            || height <= 1
            || x < left
            || y < top
            || x >= left + width
            || y >= top + height
        {
            return Err(DriverError::pending("owned 窗口拖选位置不在交互桌面边界内"));
        }
        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: ((i64::from(x - left) * 65535) / i64::from(width - 1)) as i32,
                    dy: ((i64::from(y - top) * 65535) / i64::from(height - 1)) as i32,
                    mouseData: 0,
                    dwFlags: MOUSEEVENTF_MOVE
                        | MOUSEEVENTF_ABSOLUTE
                        | MOUSEEVENTF_VIRTUALDESK
                        | flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        if unsafe { SendInput(1, &input, size_of::<INPUT>() as i32) } != 1 {
            return Err(DriverError::pending("合成选区的鼠标 SendInput 被阻止"));
        }
        Ok(())
    }

    struct MouseRelease {
        x: i32,
        y: i32,
        pressed: bool,
    }

    fn release_mouse_button() -> Result<(), DriverError> {
        // Release does not depend on monitor geometry: the desktop may change during a drag.
        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: 0,
                    dwFlags: MOUSEEVENTF_LEFTUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        if unsafe { SendInput(1, &input, size_of::<INPUT>() as i32) } != 1 {
            return Err(DriverError::pending("鼠标释放 SendInput 被阻止"));
        }
        Ok(())
    }

    impl Drop for MouseRelease {
        fn drop(&mut self) {
            if self.pressed {
                let _ = release_mouse_button();
            }
        }
    }

    fn draw_synthetic_selection(window: &WindowState, pid: u32) -> Result<(), DriverError> {
        owned_window(pid, window.hwnd)?;
        if unsafe { GetAsyncKeyState(i32::from(VK_LBUTTON)) } < 0 {
            return Err(DriverError::pending("用户鼠标左键仍按住"));
        }
        let (width, height) = client_rect(pid, window.hwnd)?;
        let mut point = windows_sys::Win32::Foundation::POINT { x: 0, y: 0 };
        if unsafe { ClientToScreen(window.hwnd as HWND, &mut point) } == 0 {
            return Err(DriverError::failed("读取 owned 遮罩位置失败"));
        }
        let start = (point.x + (width / 5) as i32, point.y + (height / 4) as i32);
        let end = (
            point.x + (width * 7 / 10) as i32,
            point.y + (height * 3 / 5) as i32,
        );
        let mut held = MouseRelease {
            x: start.0,
            y: start.1,
            pressed: false,
        };
        if unsafe { GetForegroundWindow() } != window.hwnd as HWND {
            return Err(DriverError::pending("合成遮罩失去焦点"));
        }
        mouse_input(start.0, start.1, 0)?;
        mouse_input(start.0, start.1, MOUSEEVENTF_LEFTDOWN)?;
        held.pressed = true;
        for step in 1..=12 {
            if unsafe { GetForegroundWindow() } != window.hwnd as HWND {
                return Err(DriverError::pending("合成拖选时用户切换了窗口"));
            }
            held.x = start.0 + (end.0 - start.0) * step / 12;
            held.y = start.1 + (end.1 - start.1) * step / 12;
            mouse_input(held.x, held.y, 0)?;
            pause(Duration::from_millis(25));
        }
        release_mouse_button()?;
        held.pressed = false;
        pause(Duration::from_millis(500));
        Ok(())
    }

    fn print_owned_window(pid: u32, hwnd: usize) -> Result<RgbImage, DriverError> {
        let (width, height) = client_rect(pid, hwnd)?;
        unsafe {
            let dc = CreateCompatibleDC(null_mut());
            if dc.is_null() {
                return Err(DriverError::failed("创建内存绘图上下文失败"));
            }
            struct MemoryDc(windows_sys::Win32::Graphics::Gdi::HDC);
            impl Drop for MemoryDc {
                fn drop(&mut self) {
                    unsafe {
                        DeleteDC(self.0);
                    }
                }
            }
            let _dc = MemoryDc(dc);
            let info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width as i32,
                    biHeight: -(height as i32),
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB,
                    ..zeroed()
                },
                ..zeroed()
            };
            let mut pixels = null_mut();
            let bitmap = CreateDIBSection(dc, &info, DIB_RGB_COLORS, &mut pixels, null_mut(), 0);
            if bitmap.is_null() || pixels.is_null() {
                if !bitmap.is_null() {
                    DeleteObject(bitmap);
                }
                return Err(DriverError::failed("创建 owned 窗口位图失败"));
            }
            struct Bitmap(windows_sys::Win32::Graphics::Gdi::HBITMAP);
            impl Drop for Bitmap {
                fn drop(&mut self) {
                    unsafe {
                        DeleteObject(self.0);
                    }
                }
            }
            let _bitmap = Bitmap(bitmap);
            let old = SelectObject(dc, bitmap);
            struct Selected {
                dc: windows_sys::Win32::Graphics::Gdi::HDC,
                old: windows_sys::Win32::Graphics::Gdi::HGDIOBJ,
            }
            impl Drop for Selected {
                fn drop(&mut self) {
                    unsafe {
                        SelectObject(self.dc, self.old);
                    }
                }
            }
            let _selected = Selected { dc, old };
            // PrintWindow targets only this HWND. There is deliberately no desktop/screen BitBlt fallback.
            let size = width as usize * height as usize * 4;
            std::ptr::write_bytes(pixels, 0, size);
            if PrintWindow(
                hwnd as HWND,
                dc,
                PW_CLIENTONLY | 2, /* PW_RENDERFULLCONTENT */
            ) == 0
            {
                return Err(DriverError::pending(
                    "owned WebView 不支持 PrintWindow；未读取桌面作替代",
                ));
            }
            owned_window(pid, hwnd)?;
            let bgra = std::slice::from_raw_parts(pixels.cast::<u8>(), size);
            let mut rgb = Vec::with_capacity(width as usize * height as usize * 3);
            for pixel in bgra.chunks_exact(4) {
                rgb.extend_from_slice(&[pixel[2], pixel[1], pixel[0]]);
            }
            validate_pixels(&rgb, width, height)?;
            RgbImage::from_raw(width, height, rgb)
                .ok_or_else(|| DriverError::failed("owned 窗口像素编码失败"))
        }
    }

    fn capture(window: &WindowState, pid: u32) -> Result<RgbImage, DriverError> {
        let hwnd = window.hwnd;
        let (send, receive) = mpsc::sync_channel(1);
        thread::Builder::new()
            .name("owned-window-print".into())
            .spawn(move || {
                let _ = send.send(print_owned_window(pid, hwnd));
            })
            .map_err(|_| DriverError::failed("无法创建窗口打印线程"))?;
        let image = receive
            .recv_timeout(Duration::from_secs(5))
            .map_err(|_| DriverError::pending("PrintWindow 超时；未保存截图"))??;
        if image.width() <= 1280 {
            return Ok(image);
        }
        let height = ((u64::from(image.height()) * 1280) / u64::from(image.width())).max(1) as u32;
        Ok(image::imageops::resize(
            &image,
            1280,
            height,
            FilterType::Lanczos3,
        ))
    }

    fn verify(fixture: &mut Fixture, source: &SourceWindow) -> Result<(), DriverError> {
        fixture.registered()?;
        let state = fixture.rpc("state", json!({}))?;
        let overlay_blur = overlay_blur_behavior(state["screenshotBlurBehavior"].as_str())?;
        for (digit, label) in [
            (b'0', "main"),
            (b'2', "selection"),
            (b'4', "input"),
            (b'3', "screenshot_overlay"),
        ] {
            fixture.close_all()?;
            chord(source, digit)?;
            let window = fixture.wait_window(label)?;
            if fixture
                .windows()?
                .iter()
                .any(|other| other.visible && other.label != label)
            {
                return Err(DriverError::failed(format!(
                    "Alt+{} 还打开了非目标窗口",
                    char::from(digit)
                )));
            }
            if label == "screenshot_overlay" && overlay_blur == OverlayBlurBehavior::Keep {
                source.focus()?;
                let retained = fixture.wait_retained_after_blur(label)?;
                owned_window(fixture.child.id(), retained.hwnd)?;
                unsafe {
                    SetForegroundWindow(retained.hwnd as HWND);
                }
                pause(Duration::from_millis(160));
                if unsafe { GetForegroundWindow() } != retained.hwnd as HWND {
                    return Err(DriverError::pending(
                        "无法重新激活本次保留的截图窗口，Esc 验收未执行",
                    ));
                }
                let refocused = fixture.wait_window(label)?;
                escape(&refocused, fixture.child.id())?;
                fixture.wait_hidden(label)?;
            } else if ["selection", "screenshot_overlay"].contains(&label) {
                source.focus()?;
                fixture.wait_hidden(label)?;
                // Re-open and separately verify Escape cancellation, rather than just forcing hide.
                chord(source, digit)?;
                let reopened = fixture.wait_window(label)?;
                escape(&reopened, fixture.child.id())?;
                fixture.wait_hidden(label)?;
            } else if label == "input" {
                escape(&window, fixture.child.id())?;
                fixture.wait_hidden(label)?;
            } else {
                source.focus()?;
                if !fixture
                    .windows()?
                    .iter()
                    .any(|window| window.label == "main" && window.visible)
                {
                    return Err(DriverError::failed("设置页失焦后被错误隐藏"));
                }
            }
            println!(
                "PASS Alt+{} → owned {label}；真实 SendInput／PID／窗口状态已验证",
                char::from(digit)
            );
        }
        fixture.close_all()?;
        println!("热键验收通过；选择文本与截图为固定合成 fixture，不代表 UIA／真实截图权限验收。");
        Ok(())
    }

    fn capture_readme(
        fixture: &mut Fixture,
        source: &SourceWindow,
        options: &Options,
    ) -> Result<(), DriverError> {
        fixture.registered()?;
        let mut staged = Vec::new();
        for language in &options.languages {
            for theme in &options.themes {
                fixture.close_all()?;
                fixture.configure(language, theme)?;
                let mut windows = vec![(b'0', "main"), (b'2', "selection"), (b'4', "input")];
                if options.include_screenshot {
                    windows.push((b'3', "screenshot_overlay"));
                }
                for (digit, label) in windows {
                    fixture.close_all()?;
                    chord(source, digit)?;
                    let window = fixture.wait_window(label)?;
                    if label == "screenshot_overlay" {
                        draw_synthetic_selection(&window, fixture.child.id())?;
                        fixture.wait_screenshot_selection()?;
                    }
                    pause(Duration::from_millis(600));
                    let image = capture(&window, fixture.child.id())?;
                    let relative = screenshot_name(language, theme, label)?;
                    let staging = fixture.root.join("captures").join(&relative);
                    fs::create_dir_all(staging.parent().unwrap())
                        .map_err(|_| DriverError::failed("无法创建独立截图暂存目录"))?;
                    image
                        .save_with_format(&staging, image::ImageFormat::Png)
                        .map_err(|_| DriverError::failed("owned 窗口 PNG 编码失败"))?;
                    staged.push((staging, relative));
                }
            }
        }
        fixture.close_all()?;
        // Publish only after every requested language/theme/window yielded nonblank content.
        for (staging, relative) in staged {
            let destination = options.output_dir.join(relative);
            let parent = destination.parent().unwrap();
            fs::create_dir_all(parent).map_err(|_| DriverError::failed("无法创建输出目录"))?;
            let temporary = destination.with_extension(format!("{}.tmp", fixture.token));
            struct TemporaryOutput(PathBuf);
            impl Drop for TemporaryOutput {
                fn drop(&mut self) {
                    let _ = fs::remove_file(&self.0);
                }
            }
            let mut output = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)
                .map_err(|_| DriverError::failed("无法创建截图发布暂存文件"))?;
            let _cleanup = TemporaryOutput(temporary.clone());
            let copied = std::io::copy(
                &mut fs::File::open(&staging)
                    .map_err(|_| DriverError::failed("截图暂存文件不存在"))?,
                &mut output,
            );
            let synced = output.sync_all();
            drop(output);
            if copied.is_err() || synced.is_err() {
                let _ = fs::remove_file(&temporary);
                return Err(DriverError::failed("截图发布写入失败"));
            }
            if fs::rename(&temporary, &destination).is_err() {
                let _ = fs::remove_file(&temporary);
                return Err(DriverError::failed("截图发布重命名失败"));
            }
        }
        println!("README 截图已生成：仅 owned fixture 窗口，固定合成内容，最大宽度 1280；未读取用户桌面。");
        Ok(())
    }

    pub(super) fn run(options: Options) -> Result<(), DriverError> {
        interactive_desktop()?;
        let source = SourceWindow::create()?;
        source.focus()?;
        let mut fixture = Fixture::start(&options.fixture_exe)?;
        match options.mode {
            Mode::Verify => verify(&mut fixture, &source),
            Mode::Capture => capture_readme(&mut fixture, &source, &options),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_blur_contract_requires_an_explicit_supported_behavior() {
        assert_eq!(
            overlay_blur_behavior(Some("hide")).ok().unwrap(),
            OverlayBlurBehavior::Hide
        );
        assert_eq!(
            overlay_blur_behavior(Some("keep")).ok().unwrap(),
            OverlayBlurBehavior::Keep
        );
        for unknown in [None, Some(""), Some("KEEP"), Some("ignored")] {
            assert!(overlay_blur_behavior(unknown).is_err());
        }
    }
    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }
    #[test]
    fn missing_fixture_never_attaches_to_existing_process() {
        assert_eq!(
            parse_options(&args(&["verify-hotkeys"]))
                .err()
                .unwrap()
                .code,
            2
        );
    }
    #[test]
    fn rejects_unknown_duplicate_and_path_traversal_values() {
        for values in [
            vec![
                "capture-readme",
                "--fixture-exe",
                "fixture.exe",
                "--themes",
                "light,light",
            ],
            vec![
                "capture-readme",
                "--fixture-exe",
                "fixture.exe",
                "--languages",
                "../../private",
            ],
            vec!["verify-hotkeys", "--fixture-exe", "a", "--fixture-exe", "b"],
            vec!["verify-hotkeys", "--exe", "a"],
        ] {
            assert_eq!(parse_options(&args(&values)).err().unwrap().code, 1);
        }
    }
    #[test]
    fn preserves_spaces_in_executable_path_and_default_coverage() {
        let parsed = parse_options(&args(&[
            "capture-readme",
            "--fixture-exe",
            "C:/fixture path/quickpick.exe",
        ]))
        .unwrap_or_else(|_| panic!("valid arguments"));
        assert_eq!(
            parsed.fixture_exe,
            PathBuf::from("C:/fixture path/quickpick.exe")
        );
        assert_eq!(parsed.themes, ["light", "dark"]);
        assert_eq!(parsed.languages, ["zh-Hans", "en"]);
    }
    #[test]
    fn screenshot_paths_preserve_existing_readme_filenames() {
        assert_eq!(
            screenshot_name("zh-Hans", "dark", "main").unwrap_or_else(|_| panic!("valid name")),
            PathBuf::from("screenshots/zh-cn/settings-dark.png")
        );
        assert_eq!(
            screenshot_name("en", "light", "screenshot_overlay")
                .unwrap_or_else(|_| panic!("valid name")),
            PathBuf::from("screenshots/en/screenshot-light.png")
        );
        assert!(screenshot_name("en", "../private", "main").is_err());
        assert!(screenshot_name("en", "light", "other-pid").is_err());
    }
    #[test]
    fn blank_black_uniform_and_incomplete_capture_cannot_pass() {
        for color in [[0, 0, 0], [255, 255, 255], [35, 40, 50]] {
            assert_eq!(
                validate_pixels(&color.repeat(60 * 40), 60, 40)
                    .err()
                    .unwrap()
                    .code,
                2
            );
        }
        assert_eq!(validate_pixels(&[0; 3], 60, 40).err().unwrap().code, 1);
    }
    #[test]
    fn nonblank_synthetic_content_is_accepted() {
        let data: Vec<u8> = (0..60 * 40)
            .flat_map(|index| {
                [
                    ((index * 13) % 256) as u8,
                    ((index * 7) % 256) as u8,
                    ((index * 23) % 256) as u8,
                ]
            })
            .collect();
        assert!(validate_pixels(&data, 60, 40).is_ok());
    }
    #[test]
    fn production_executable_without_marker_is_rejected() {
        assert!(!has_fixture_marker(std::io::Cursor::new(
            b"MZ production application".as_slice()
        ))
        .unwrap());
    }
    #[test]
    fn marker_split_across_read_chunks_is_recognized() {
        let mut binary = vec![0; 64 * 1024 - 12];
        binary.extend_from_slice(FIXTURE_MARKER.as_bytes());
        binary.extend_from_slice(&[0; 30]);
        assert!(has_fixture_marker(std::io::Cursor::new(binary)).unwrap());
    }

    struct MockKeys {
        calls: Vec<(u16, bool)>,
        fail: Option<(u16, bool)>,
    }
    impl KeyWriter for MockKeys {
        fn send(&mut self, key: u16, release: bool) -> Result<(), DriverError> {
            self.calls.push((key, release));
            if self.fail == Some((key, release)) {
                self.fail = None;
                return Err(DriverError::pending("synthetic partial SendInput"));
            }
            Ok(())
        }
    }
    #[test]
    fn partial_chord_press_releases_only_owned_alt() {
        let mut keys = MockKeys {
            calls: Vec::new(),
            fail: Some((0x32, false)),
        };
        {
            let mut held = HeldKeys::new(&mut keys);
            assert!(held.press(0x12).is_ok());
            assert!(held.press(0x32).is_err());
        }
        assert_eq!(keys.calls, [(0x12, false), (0x32, false), (0x12, true)]);
    }
    #[test]
    fn failed_digit_release_still_releases_alt_and_retries_digit() {
        let mut keys = MockKeys {
            calls: Vec::new(),
            fail: Some((0x32, true)),
        };
        {
            let mut held = HeldKeys::new(&mut keys);
            assert!(held.press(0x12).is_ok());
            assert!(held.press(0x32).is_ok());
            let digit = held.release(0x32);
            let alt = held.release(0x12);
            assert!(digit.is_err());
            assert!(alt.is_ok());
        }
        assert_eq!(
            keys.calls,
            [
                (0x12, false),
                (0x32, false),
                (0x32, true),
                (0x12, true),
                (0x32, true)
            ]
        );
    }
    #[test]
    fn all_owned_releases_are_attempted_even_if_one_retry_fails() {
        let mut keys = MockKeys {
            calls: Vec::new(),
            fail: Some((0x32, true)),
        };
        {
            let mut held = HeldKeys::new(&mut keys);
            assert!(held.press(0x12).is_ok());
            assert!(held.press(0x32).is_ok());
        }
        assert_eq!(
            keys.calls,
            [(0x12, false), (0x32, false), (0x32, true), (0x12, true)]
        );
    }
}
