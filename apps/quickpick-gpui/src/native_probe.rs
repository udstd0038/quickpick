//! 原生托盘、菜单和热键均在 GPUI UI 线程创建，回调仅投递类型化消息。
use async_channel::Sender;
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};
use muda::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, TrayIconEvent};

pub enum NativeEvent {
    Hotkey,
    Tray,
    Menu,
    Quit,
}

pub struct NativeProbe {
    manager: GlobalHotKeyManager,
    hotkey: HotKey,
    _tray: TrayIcon,
}

impl NativeProbe {
    /// 此样机每进程只启用一次；避免改变事件库的进程级 handler。
    pub fn new(sender: Sender<NativeEvent>) -> Result<Self, &'static str> {
        let menu = Menu::new();
        let activate = MenuItem::with_id("probe-show", "显示验证窗口", true, None);
        let quit = MenuItem::with_id("probe-quit", "退出验证程序", true, None);
        menu.append_items(&[&activate, &quit])
            .map_err(|_| "创建原生验证菜单失败")?;
        let image = image::load_from_memory(include_bytes!("../assets/tray-icon.png"))
            .map_err(|_| "读取托盘资源失败")?
            .into_rgba8();
        let icon = Icon::from_rgba(image.as_raw().clone(), image.width(), image.height())
            .map_err(|_| "创建托盘图标失败")?;
        let tray_sender = sender.clone();
        TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
            if matches!(event, TrayIconEvent::DoubleClick { .. }) {
                let _ = tray_sender.try_send(NativeEvent::Tray);
            }
        }));
        let tray = TrayIconBuilder::new()
            .with_tooltip("QuickPick · GPUI 验证程序")
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .build()
            .map_err(|_| "创建托盘验证图标失败")?;
        let manager = GlobalHotKeyManager::new().map_err(|_| "初始化热键验证失败")?;
        let hotkey = HotKey::new(Some(Modifiers::ALT | Modifiers::SHIFT), Code::F12);
        manager
            .register(hotkey)
            .map_err(|_| "Alt+Shift+F12 已被占用，未注册测试热键")?;

        let hotkey_sender = sender.clone();
        GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
            if event.id == hotkey.id() && event.state == HotKeyState::Pressed {
                let _ = hotkey_sender.try_send(NativeEvent::Hotkey);
            }
        }));
        let activate_id = activate.id().clone();
        let quit_id = quit.id().clone();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            if event.id == activate_id {
                let _ = sender.try_send(NativeEvent::Menu);
            } else if event.id == quit_id {
                let _ = sender.try_send(NativeEvent::Quit);
            }
        }));
        Ok(Self {
            manager,
            hotkey,
            _tray: tray,
        })
    }
}

impl Drop for NativeProbe {
    fn drop(&mut self) {
        let _ = self.manager.unregister(self.hotkey);
    }
}
