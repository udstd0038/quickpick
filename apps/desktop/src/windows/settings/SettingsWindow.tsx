import {
  type CSSProperties,
  type KeyboardEvent as ReactKeyboardEvent,
  type PointerEvent,
  useEffect,
  useRef,
  useState,
} from "react";
import { createPortal } from "react-dom";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  applyDocumentTranslation,
  targetLanguageForUiLanguage,
  translationLanguageOptions,
  uiLanguageOptions,
  type UiLanguage,
} from "../../lib/i18n";
import { useSettingsStore } from "../../stores/settingsStore";

type WindowKind =
  | "main"
  | "settings"
  | "selection_bar"
  | "result"
  | "screenshot_overlay";
type SelectionStatus = "captured" | "empty" | "unsupported" | "error";

type SelectionSnapshot = {
  status: SelectionStatus;
  text: string;
  preview: string;
  charCount: number;
  message: string;
  source: string;
};

type SelectionActionResult = {
  message: string;
};

type OperationState = {
  kind: "idle" | "pending" | "success" | "error";
  message: string;
};

type ScreenshotStatusEvent = {
  kind: "success" | "error";
  message: string;
};

type DiagnosticStatus = "ok" | "paused" | "blocked" | "info";

type DiagnosticItem = {
  label: string;
  value: string;
  status: DiagnosticStatus;
};

type DiagnosticSection = {
  title: string;
  detail: string;
  items: DiagnosticItem[];
};

type MainDiagnostics = {
  summary: string;
  sections: DiagnosticSection[];
};

type ResultSnapshot = {
  status: "empty" | "placeholder" | "loading" | "success" | "error";
  kind: string;
  title: string;
  content: string;
  detail: string;
  sourcePreview: string;
  sourceCharCount: number;
  canCopy: boolean;
  canRetry: boolean;
  sourceLanguage: string;
  targetLanguage: string;
  translationDirection: string;
  canSwitchLanguage: boolean;
};

type AppSettings = {
  autostartEnabled: boolean;
  settingsHotkey: string;
  selectionHotkey: string;
  screenshotHotkey: string;
  inputTranslateHotkey: string;
  uiLanguage: UiLanguage;
  themeMode: "system" | "light" | "dark";
  windowEffect: "acrylic" | "mica";
  panelOpacity: number;
  textAiProvider: string;
  textAiBaseUrl: string;
  textAiModel: string;
  visionAiProvider: string;
  visionAiBaseUrl: string;
  visionAiModel: string;
  inputAiProvider: string;
  inputAiBaseUrl: string;
  inputAiModel: string;
  inputTranslateSourceLanguage: string;
  inputTranslateTargetLanguage: string;
  translationTargetLanguage: string;
  aiProvider: string;
  aiBaseUrl: string;
  aiTextModel: string;
  aiVisionModel: string;
  aiTimeoutSeconds: number;
};

type HotkeySettingKey =
  | "settingsHotkey"
  | "selectionHotkey"
  | "screenshotHotkey"
  | "inputTranslateHotkey";

type ApiKeyStatus = {
  configured: boolean;
};

const modifierKeyNames = new Set([
  "Alt",
  "AltGraph",
  "Control",
  "Meta",
  "OS",
  "Shift",
]);

function keyNameFromKeyboardEvent(
  event: ReactKeyboardEvent<HTMLElement>,
): string | null {
  const code = event.code;

  if (/^Key[A-Z]$/.test(code)) {
    return code.slice(3);
  }
  if (/^Digit[0-9]$/.test(code)) {
    return code.slice(5);
  }
  if (/^F([1-9]|1[0-9]|2[0-4])$/.test(code)) {
    return code;
  }

  const codeMap: Record<string, string> = {
    ArrowDown: "ArrowDown",
    ArrowLeft: "ArrowLeft",
    ArrowRight: "ArrowRight",
    ArrowUp: "ArrowUp",
    Backspace: "Backspace",
    Delete: "Delete",
    End: "End",
    Enter: "Enter",
    Home: "Home",
    Insert: "Insert",
    PageDown: "PageDown",
    PageUp: "PageUp",
    PrintScreen: "PrintScreen",
    Space: "Space",
    Tab: "Tab",
  };

  return codeMap[code] || null;
}

function shortcutFromKeyboardEvent(
  event: ReactKeyboardEvent<HTMLElement>,
): string | null {
  const keyName = keyNameFromKeyboardEvent(event);
  if (!keyName) {
    return null;
  }

  const parts: string[] = [];
  if (event.ctrlKey) {
    parts.push("Ctrl");
  }
  if (event.shiftKey) {
    parts.push("Shift");
  }
  if (event.altKey) {
    parts.push("Alt");
  }
  parts.push(keyName);

  return parts.join("+");
}

function sameShortcut(left: string, right: string): boolean {
  return left.replace(/\s+/g, "").toLowerCase() ===
    right.replace(/\s+/g, "").toLowerCase();
}

type SettingsStatus = {
  kind: "idle" | "loading" | "saving" | "success" | "error";
  message: string;
};

type ScreenshotPoint = {
  x: number;
  y: number;
};

type ScreenshotRect = {
  left: number;
  top: number;
  width: number;
  height: number;
};

const minScreenshotSelectionSize = 8;

const defaultAppSettings: AppSettings = {
  autostartEnabled: true,
  settingsHotkey: "Alt+1",
  selectionHotkey: "Alt+2",
  screenshotHotkey: "Alt+3",
  inputTranslateHotkey: "Alt+4",
  uiLanguage: "system",
  themeMode: "system",
  windowEffect: "acrylic",
  panelOpacity: 70,
  textAiProvider: "deepseek",
  textAiBaseUrl: "",
  textAiModel: "",
  visionAiProvider: "xiaomi_mimo",
  visionAiBaseUrl: "",
  visionAiModel: "",
  inputAiProvider: "deepseek",
  inputAiBaseUrl: "",
  inputAiModel: "",
  inputTranslateSourceLanguage: "auto",
  inputTranslateTargetLanguage: "zh-Hans",
  translationTargetLanguage: "zh-Hans",
  aiProvider: "",
  aiBaseUrl: "",
  aiTextModel: "",
  aiVisionModel: "",
  aiTimeoutSeconds: 30,
};

const textAiProviderOptions = [
  {
    id: "openai_compatible",
    label: "通用 OpenAI 兼容",
    baseUrl: "",
    textModel: "",
    visionModel: "",
  },
  {
    id: "deepseek",
    label: "DeepSeek",
    baseUrl: "https://api.deepseek.com",
    textModel: "deepseek-v4-flash",
    visionModel: "deepseek-v4-flash",
  },
  {
    id: "xiaomi_mimo",
    label: "小米 MiMo",
    baseUrl: "https://api.xiaomimimo.com/v1",
    textModel: "mimo-v2.5",
    visionModel: "mimo-v2.5",
  },
  {
    id: "kimi",
    label: "Kimi",
    baseUrl: "https://api.moonshot.cn/v1",
    textModel: "kimi-k2.6",
    visionModel: "kimi-k2.6",
  },
  {
    id: "glm",
    label: "GLM",
    baseUrl: "https://open.bigmodel.cn/api/paas/v4",
    textModel: "glm-5.2",
    visionModel: "glm-4.5v",
  },
  {
    id: "minimax",
    label: "MiniMax",
    baseUrl: "https://api.minimaxi.com/v1",
    textModel: "MiniMax-M2.7",
    visionModel: "MiniMax-VL-01",
  },
  {
    id: "qwen",
    label: "Qwen",
    baseUrl: "https://dashscope.aliyuncs.com/compatible-mode/v1",
    textModel: "qwen-plus",
    visionModel: "qwen3-vl-plus",
  },
];

const visionAiProviderOptions = textAiProviderOptions.filter(
  (option) => option.id !== "deepseek",
);
const aiProviderOptions = textAiProviderOptions;
const inputAiProviderOptions = textAiProviderOptions;

const themeModeOptions = [
  { id: "system", label: "跟随系统" },
  { id: "light", label: "浅色" },
  { id: "dark", label: "深色" },
] as const;

const windowEffectOptions = [
  { id: "acrylic", label: "Acrylic" },
  { id: "mica", label: "Mica" },
] as const;

type GlassSelectOption<T extends string = string> = {
  id: T;
  label: string;
};

const settingsNavItems = [
  { id: "general", label: "通用" },
  { id: "hotkeys", label: "快捷键" },
  { id: "ai-selection", label: "划词模型" },
  { id: "ai-screenshot", label: "截图模型" },
  { id: "ai-input", label: "输入模型" },
  { id: "appearance", label: "外观" },
  { id: "privacy", label: "隐私" },
] as const;

type SettingsSectionId = (typeof settingsNavItems)[number]["id"];

function currentSystemTheme(): "light" | "dark" {
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

const appearanceAlphaVariables = [
  "--qp-shell-alpha",
  "--qp-panel-alpha",
  "--qp-panel-strong-alpha",
  "--qp-panel-soft-alpha",
  "--qp-control-alpha",
  "--qp-input-alpha",
  "--qp-footer-alpha",
];

function applyDocumentAppearance(settings: AppSettings) {
  const root = document.documentElement;
  applyDocumentTranslation(settings.uiLanguage);
  const themeMode =
    settings.themeMode === "system" ? currentSystemTheme() : settings.themeMode;
  const effectiveTheme = themeMode === "dark" ? "workbench" : themeMode;

  root.dataset.theme = effectiveTheme;
  root.dataset.themePreference = settings.themeMode;
  root.style.colorScheme = effectiveTheme === "light" ? "light" : "dark";
  root.dataset.windowEffect = settings.windowEffect;
  if (settings.windowEffect === "mica") {
    for (const name of appearanceAlphaVariables) {
      root.style.removeProperty(name);
    }
    return;
  }

  const panelOpacity =
    Math.min(100, Math.max(30, Number(settings.panelOpacity) || 70)) / 100;
  for (const name of appearanceAlphaVariables) {
    root.style.setProperty(name, String(panelOpacity));
  }
}

function applyTextAiProviderDefaults(
  current: AppSettings,
  providerId: string,
): AppSettings {
  const provider =
    textAiProviderOptions.find((option) => option.id === providerId) ||
    textAiProviderOptions[0];

  return {
    ...current,
    textAiProvider: provider.id,
    textAiBaseUrl: provider.baseUrl || current.textAiBaseUrl,
    textAiModel: provider.textModel || current.textAiModel,
  };
}

function applyAiProviderDefaults(
  current: AppSettings,
  providerId: string,
): AppSettings {
  return applyTextAiProviderDefaults(current, providerId);
}

function applyVisionAiProviderDefaults(
  current: AppSettings,
  providerId: string,
): AppSettings {
  const provider =
    visionAiProviderOptions.find((option) => option.id === providerId) ||
    visionAiProviderOptions[0];

  return {
    ...current,
    visionAiProvider: provider.id,
    visionAiBaseUrl: provider.baseUrl || current.visionAiBaseUrl,
    visionAiModel:
      provider.visionModel ||
      (provider.id === "openai_compatible" ? current.visionAiModel : ""),
  };
}

function applyInputAiProviderDefaults(
  current: AppSettings,
  providerId: string,
): AppSettings {
  const provider =
    inputAiProviderOptions.find((option) => option.id === providerId) ||
    inputAiProviderOptions[0];

  return {
    ...current,
    inputAiProvider: provider.id,
    inputAiBaseUrl: provider.baseUrl || current.inputAiBaseUrl,
    inputAiModel:
      provider.textModel ||
      (provider.id === "openai_compatible" ? current.inputAiModel : ""),
  };
}

function normalizeAppSettings(current: AppSettings): AppSettings {
  const textProvider =
    textAiProviderOptions.find((option) => option.id === current.textAiProvider) ||
    textAiProviderOptions[1];
  const visionProvider =
    visionAiProviderOptions.find((option) => option.id === current.visionAiProvider) ||
    visionAiProviderOptions[0];
  const inputProvider =
    inputAiProviderOptions.find((option) => option.id === current.inputAiProvider) ||
    inputAiProviderOptions[1];
  const rawThemeMode = String(current.themeMode).trim();
  const themeMode = rawThemeMode === "workbench"
    ? "dark"
    : themeModeOptions.some((option) => option.id === rawThemeMode)
      ? (rawThemeMode as AppSettings["themeMode"])
      : defaultAppSettings.themeMode;
  const uiLanguage: UiLanguage = uiLanguageOptions.some(
    (option) => option.id === current.uiLanguage,
  )
    ? (current.uiLanguage as UiLanguage)
    : defaultAppSettings.uiLanguage;
  const defaultTargetLanguage = targetLanguageForUiLanguage(uiLanguage);
  const translationTargetLanguage =
    uiLanguage === "system" || !translationLanguageOptions.some(
      (option) => option.id === current.translationTargetLanguage,
    )
      ? defaultTargetLanguage
      : current.translationTargetLanguage;
  const inputTranslateSourceLanguage =
    current.inputTranslateSourceLanguage === "auto" ||
    translationLanguageOptions.some(
      (option) => option.id === current.inputTranslateSourceLanguage,
    )
      ? current.inputTranslateSourceLanguage
      : defaultAppSettings.inputTranslateSourceLanguage;
  const inputTranslateTargetLanguage =
    uiLanguage === "system" || !translationLanguageOptions.some(
      (option) => option.id === current.inputTranslateTargetLanguage,
    )
      ? defaultTargetLanguage
      : current.inputTranslateTargetLanguage;
  const timeout = Number(current.aiTimeoutSeconds);

  return {
    autostartEnabled: current.autostartEnabled,
    settingsHotkey:
      current.settingsHotkey.trim() || defaultAppSettings.settingsHotkey,
    selectionHotkey: current.selectionHotkey.trim() || defaultAppSettings.selectionHotkey,
    screenshotHotkey:
      current.screenshotHotkey.trim() || defaultAppSettings.screenshotHotkey,
    inputTranslateHotkey:
      current.inputTranslateHotkey.trim() ||
      defaultAppSettings.inputTranslateHotkey,
    uiLanguage,
    themeMode,
    windowEffect: current.windowEffect === "mica" ? "mica" : "acrylic",
    panelOpacity:
      Number.isFinite(Number(current.panelOpacity)) &&
      Number(current.panelOpacity) >= 30 &&
      Number(current.panelOpacity) <= 100
        ? Math.round(Number(current.panelOpacity))
        : defaultAppSettings.panelOpacity,
    textAiProvider: textProvider.id,
    textAiBaseUrl: current.textAiBaseUrl.trim() || textProvider.baseUrl,
    textAiModel: current.textAiModel.trim() || textProvider.textModel,
    visionAiProvider: visionProvider.id,
    visionAiBaseUrl: current.visionAiBaseUrl.trim() || visionProvider.baseUrl,
    visionAiModel:
      current.visionAiModel.trim() ||
      (visionProvider.id === "openai_compatible" ? "" : visionProvider.visionModel),
    inputAiProvider: inputProvider.id,
    inputAiBaseUrl: current.inputAiBaseUrl.trim() || inputProvider.baseUrl,
    inputAiModel:
      current.inputAiModel.trim() ||
      (inputProvider.id === "openai_compatible"
        ? ""
        : inputProvider.textModel),
    inputTranslateSourceLanguage,
    inputTranslateTargetLanguage,
    translationTargetLanguage,
    aiProvider: "",
    aiBaseUrl: "",
    aiTextModel: "",
    aiVisionModel: "",
    aiTimeoutSeconds:
      Number.isFinite(timeout) && timeout >= 5 && timeout <= 120
        ? timeout
        : defaultAppSettings.aiTimeoutSeconds,
  };
}

const defaultResultSnapshot: ResultSnapshot = {
  status: "empty",
  kind: "empty",
  title: "QuickPick 结果",
  content: "暂无结果",
  detail: "从划词弹窗或截图菜单点击 AI 功能后，结果会通过独立原生弹窗显示。",
  sourcePreview: "",
  sourceCharCount: 0,
  canCopy: false,
  canRetry: false,
  sourceLanguage: "auto",
  targetLanguage: "zh-Hans",
  translationDirection: "right",
  canSwitchLanguage: false,
};

const resultStatusLabel: Record<ResultSnapshot["status"], string> = {
  empty: "就绪",
  placeholder: "待处理",
  loading: "处理中",
  success: "完成",
  error: "需处理",
};

const selectionStatusLabel: Record<SelectionStatus, string> = {
  captured: "已读取",
  empty: "未选中",
  unsupported: "不支持",
  error: "失败",
};

const defaultSelectionSnapshot: SelectionSnapshot = {
  status: "empty",
  text: "",
  preview: "正在读取选中文本",
  charCount: 0,
  message: "正在读取选中文本",
  source: "uia",
};

const roadmap = [
  { label: "当前", value: "原生无感弹窗" },
  { label: "下步", value: "弹窗关闭固定验收" },
  { label: "可用热键", value: "Alt+2 划词 / Alt+3 截图" },
];

function App() {
  const [windowKind, setWindowKind] = useState<WindowKind>("settings");
  const [coreStatus, setCoreStatus] = useState("正在检查核心连接");

  const checkCore = () => {
    invoke<string>("ping")
      .then(setCoreStatus)
      .catch(() => setCoreStatus("前端预览模式"));
  };

  useEffect(() => {
    try {
      const label = getCurrentWindow().label;
      if (
        label === "settings" ||
        label === "selection_bar" ||
        label === "result" ||
        label === "screenshot_overlay"
      ) {
        setWindowKind(label);
      } else {
        setWindowKind("settings");
      }
    } catch {
      setWindowKind("settings");
    }

    checkCore();
  }, []);

  if (windowKind === "selection_bar") {
    return <SelectionBar />;
  }

  if (windowKind === "result") {
    return <ResultWindow />;
  }

  if (windowKind === "screenshot_overlay") {
    return <ScreenshotOverlay />;
  }

  return <SettingsWindow coreStatus={coreStatus} />;
}

function ScreenshotOverlay() {
  const [dragStart, setDragStart] = useState<ScreenshotPoint | null>(null);
  const [dragCurrent, setDragCurrent] = useState<ScreenshotPoint | null>(null);
  const [selectedRect, setSelectedRect] = useState<ScreenshotRect | null>(null);
  const isDragging = dragStart !== null && dragCurrent !== null;
  const activeRect = isDragging
    ? normalizeScreenshotRect(dragStart, dragCurrent)
    : selectedRect;
  const hasSelection =
    activeRect !== null &&
    activeRect.width >= minScreenshotSelectionSize &&
    activeRect.height >= minScreenshotSelectionSize;
  const sizeText = activeRect
    ? `${Math.round(activeRect.width)} × ${Math.round(activeRect.height)}`
    : "拖拽框选";
  const statusTitle = isDragging
    ? "正在框选"
    : hasSelection
      ? "已选择区域"
      : "区域截图";
  const statusText = hasSelection || isDragging ? sizeText : "拖拽框选";

  const closeOverlay = () => {
    setDragStart(null);
    setDragCurrent(null);
    setSelectedRect(null);
    getCurrentWindow().hide();
  };

  const startSelection = (event: PointerEvent<HTMLElement>) => {
    if (event.button !== 0 || isInteractiveTarget(event.target)) {
      return;
    }

    const point = pointFromPointer(event);
    setDragStart(point);
    setDragCurrent(point);
    setSelectedRect(null);
    event.currentTarget.setPointerCapture(event.pointerId);
    event.preventDefault();
  };

  const updateSelection = (event: PointerEvent<HTMLElement>) => {
    if (dragStart === null) {
      return;
    }

    setDragCurrent(pointFromPointer(event));
    event.preventDefault();
  };

  const finishSelection = (event: PointerEvent<HTMLElement>) => {
    if (dragStart === null || dragCurrent === null) {
      return;
    }

    const rect = normalizeScreenshotRect(dragStart, pointFromPointer(event));
    setSelectedRect(
      rect.width >= minScreenshotSelectionSize && rect.height >= minScreenshotSelectionSize
        ? rect
        : null,
    );
    setDragStart(null);
    setDragCurrent(null);

    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    event.preventDefault();
  };

  const cancelSelection = (event: PointerEvent<HTMLElement>) => {
    setDragStart(null);
    setDragCurrent(null);
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
  };

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        closeOverlay();
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
    };
  }, []);

  return (
    <main
      className="screenshot-overlay"
      aria-label="区域截图"
      onContextMenu={(event) => event.preventDefault()}
      onPointerCancel={cancelSelection}
      onPointerDown={startSelection}
      onPointerMove={updateSelection}
      onPointerUp={finishSelection}
    >
      {activeRect && (
        <div
          className={`screenshot-frame ${hasSelection ? "screenshot-frame-ready" : ""}`}
          style={{
            left: activeRect.left,
            top: activeRect.top,
            width: activeRect.width,
            height: activeRect.height,
          }}
          aria-hidden="true"
        />
      )}
      <section className="screenshot-hud" aria-live="polite">
        <span className="screenshot-reticle" aria-hidden="true" />
        <div>
          <strong>{statusTitle}</strong>
          <span>{statusText}</span>
        </div>
        <button className="screenshot-cancel" type="button" onClick={closeOverlay}>
          取消
        </button>
      </section>
    </main>
  );
}

function pointFromPointer(event: PointerEvent<HTMLElement>): ScreenshotPoint {
  return {
    x: event.clientX,
    y: event.clientY,
  };
}

function normalizeScreenshotRect(
  start: ScreenshotPoint,
  end: ScreenshotPoint,
): ScreenshotRect {
  const left = Math.min(start.x, end.x);
  const top = Math.min(start.y, end.y);

  return {
    left,
    top,
    width: Math.abs(end.x - start.x),
    height: Math.abs(end.y - start.y),
  };
}

function isInteractiveTarget(target: EventTarget): boolean {
  return target instanceof HTMLElement && target.closest("button") !== null;
}

function SelectionBar() {
  const [snapshot, setSnapshot] = useState<SelectionSnapshot>(
    defaultSelectionSnapshot,
  );
  const [operation, setOperation] = useState<OperationState>({
    kind: "idle",
    message: "",
  });

  const canUseText = snapshot.status === "captured" && snapshot.text.trim().length > 0;
  const isPending = operation.kind === "pending";
  const actions: Array<
    | {
        label: string;
        title: string;
        command: "copy_selection_text" | "search_selection_text";
        enabled: true;
      }
    | {
        label: string;
        title: string;
        resultAction: "translate" | "summarize";
        enabled: true;
      }
    | { label: string; title: string; enabled: false }
  > = [
    { label: "复制", title: "复制选中文本", command: "copy_selection_text", enabled: true },
    { label: "翻译", title: "执行文本翻译", resultAction: "translate", enabled: true },
    { label: "总结", title: "执行文本总结", resultAction: "summarize", enabled: true },
    { label: "搜索", title: "用默认浏览器搜索", command: "search_selection_text", enabled: true },
  ];
  const statusLabel: Record<SelectionStatus, string> = {
    captured: "已读取",
    empty: "未选中",
    unsupported: "不支持",
    error: "失败",
  };
  const metaText =
    operation.message ||
    (snapshot.status === "captured"
      ? `${snapshot.charCount} 字符`
      : snapshot.message);

  const runSelectionAction = (
    command: "copy_selection_text" | "search_selection_text",
  ) => {
    if (!canUseText || isPending) {
      return;
    }

    setOperation({ kind: "pending", message: "处理中" });
    invoke<SelectionActionResult>(command)
      .then((result) => {
        setOperation({ kind: "success", message: result.message });
        window.setTimeout(() => {
          getCurrentWindow().hide();
        }, 650);
      })
      .catch((error) => {
        setOperation({
          kind: "error",
          message: typeof error === "string" ? error : "操作失败，请重新选择后再试",
        });
      });
  };

  const runTextAiAction = (action: "translate" | "summarize") => {
    if (!canUseText || isPending) {
      return;
    }

    setOperation({ kind: "pending", message: "正在请求 AI" });
    invoke<SelectionActionResult>("run_text_ai_action", { action })
      .then((result) => {
        setOperation({ kind: "success", message: result.message });
      })
      .catch((error) => {
        setOperation({
          kind: "error",
          message: typeof error === "string" ? error : "文本 AI 请求失败，请重试",
        });
      });
  };

  useEffect(() => {
    const currentWindow = getCurrentWindow();
    let isMounted = true;
    let unlisten: (() => void) | undefined;

    const loadSnapshot = () => {
      invoke<SelectionSnapshot>("get_selection_snapshot")
        .then((value) => {
          if (isMounted) {
            setSnapshot(value);
            setOperation({ kind: "idle", message: "" });
          }
        })
        .catch(() => {
          if (isMounted) {
            setSnapshot({
              status: "error",
              text: "",
              preview: "无法读取取词状态",
              charCount: 0,
              message: "无法连接 QuickPick 核心，请重新打开应用",
              source: "ui",
            });
          }
        });
    };

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        currentWindow.hide();
      }
    };

    loadSnapshot();
    listen("selection-updated", loadSnapshot).then((dispose) => {
      unlisten = dispose;
    });
    window.addEventListener("keydown", onKeyDown);

    return () => {
      isMounted = false;
      unlisten?.();
      window.removeEventListener("keydown", onKeyDown);
    };
  }, []);

  return (
    <main className="selection-shell" aria-label="划词菜单">
      <div className="selection-bar">
        <section className="selection-status" aria-live="polite">
          <div className="selection-status-line">
            <span
              className={`selection-dot selection-dot-${snapshot.status}`}
              aria-hidden="true"
            />
            <strong>{statusLabel[snapshot.status]}</strong>
            <span>{metaText}</span>
          </div>
          <p className="selection-preview" title={snapshot.preview}>
            {snapshot.preview}
          </p>
        </section>
        {actions.map((action) => (
          <button
            className={`selection-action ${
              action.enabled && canUseText ? "selection-action-active" : ""
            }`}
            type="button"
            title={
              action.enabled && !canUseText
                ? "读取到选中文本后可用"
                : action.title
            }
            aria-label={action.title}
            disabled={!action.enabled || !canUseText || isPending}
            onClick={() => {
              if (!action.enabled) {
                return;
              }

              if ("command" in action) {
                runSelectionAction(action.command);
              } else {
                runTextAiAction(action.resultAction);
              }
            }}
            key={action.label}
          >
            {action.label}
          </button>
        ))}
      </div>
    </main>
  );
}

function ResultWindow() {
  const [snapshot, setSnapshot] = useState<ResultSnapshot>(defaultResultSnapshot);
  const [copyOperation, setCopyOperation] = useState<OperationState>({
    kind: "idle",
    message: "",
  });
  const [retryOperation, setRetryOperation] = useState<OperationState>({
    kind: "idle",
    message: "",
  });
  const retryAction =
    snapshot.kind === "text_translate"
      ? "translate"
      : snapshot.kind === "text_summarize"
        ? "summarize"
        : null;
  const isRetrying = retryOperation.kind === "pending";
  const canRetry = snapshot.canRetry && retryAction !== null && !isRetrying;
  const footerFeedback = retryOperation.message
    ? retryOperation
    : copyOperation.message
      ? copyOperation
      : null;

  const closeWindow = () => {
    invoke<SelectionActionResult>("clear_result_snapshot").finally(() => {
      getCurrentWindow().hide();
    });
  };

  const copyResult = () => {
    if (!snapshot.canCopy || copyOperation.kind === "pending") {
      return;
    }

    setCopyOperation({ kind: "pending", message: "正在复制" });
    invoke<SelectionActionResult>("copy_result_content")
      .then((result) => {
        setCopyOperation({ kind: "success", message: result.message });
      })
      .catch((error) => {
        setCopyOperation({
          kind: "error",
          message: typeof error === "string" ? error : "复制失败，请稍后再试",
        });
      });
  };

  const retryResult = () => {
    if (!canRetry || retryAction === null) {
      return;
    }

    setCopyOperation({ kind: "idle", message: "" });
    setRetryOperation({ kind: "pending", message: "正在重试" });
    invoke<SelectionActionResult>("run_text_ai_action", { action: retryAction })
      .then((result) => {
        setRetryOperation({ kind: "success", message: result.message });
      })
      .catch((error) => {
        setRetryOperation({
          kind: "error",
          message: typeof error === "string" ? error : "重试失败，请重新划词后再试",
        });
      });
  };

  useEffect(() => {
    let isMounted = true;
    let unlisten: (() => void) | undefined;

    const loadSnapshot = () => {
      invoke<ResultSnapshot>("get_result_snapshot")
        .then((value) => {
          if (isMounted) {
            setSnapshot(value);
            setCopyOperation({ kind: "idle", message: "" });
            setRetryOperation({ kind: "idle", message: "" });
          }
        })
        .catch(() => {
          if (isMounted) {
            setSnapshot({
              status: "error",
              kind: "error",
              title: "结果弹窗",
              content: "无法读取结果状态",
              detail: "请关闭后重新从划词弹窗触发。",
              sourcePreview: "",
              sourceCharCount: 0,
              canCopy: false,
              canRetry: false,
              sourceLanguage: "auto",
              targetLanguage: "zh-Hans",
              translationDirection: "right",
              canSwitchLanguage: false,
            });
          }
        });
    };

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        closeWindow();
      }
    };

    loadSnapshot();
    listen("result-updated", loadSnapshot).then((dispose) => {
      unlisten = dispose;
    });
    window.addEventListener("keydown", onKeyDown);

    return () => {
      isMounted = false;
      unlisten?.();
      window.removeEventListener("keydown", onKeyDown);
    };
  }, []);

  return (
    <main className="app-shell result-window">
      <header className="result-header">
        <div>
          <p className="eyebrow">结果</p>
          <h1>{snapshot.title}</h1>
        </div>
        <span className={`result-status result-status-${snapshot.status}`}>
          {resultStatusLabel[snapshot.status]}
        </span>
      </header>

      <section className="result-body" aria-label="AI 结果内容">
        {snapshot.sourcePreview && (
          <div className="result-source">
            <span>来源</span>
            <p title={snapshot.sourcePreview}>{snapshot.sourcePreview}</p>
            <strong>{snapshot.sourceCharCount} 字符</strong>
          </div>
        )}

        <article className="result-content">
          <p>{snapshot.content}</p>
        </article>

        <p className="result-detail">{snapshot.detail}</p>
      </section>

      <footer className="result-footer">
        {footerFeedback && (
          <span
            className={`result-feedback result-feedback-${footerFeedback.kind}`}
            aria-live="polite"
          >
            {footerFeedback.message}
          </span>
        )}
        <button
          className="ghost-control"
          type="button"
          disabled={!snapshot.canCopy || copyOperation.kind === "pending" || isRetrying}
          onClick={copyResult}
        >
          复制结果
        </button>
        <button
          className="ghost-control"
          type="button"
          disabled={!canRetry}
          onClick={retryResult}
        >
          重试
        </button>
        <button className="primary-action" type="button" onClick={closeWindow}>
          关闭
        </button>
      </footer>
    </main>
  );
}

function MainWindow({
  coreStatus,
  onCheckCore,
}: {
  coreStatus: string;
  onCheckCore: () => void;
}) {
  const [diagnostics, setDiagnostics] = useState<MainDiagnostics | null>(null);
  const [diagnosticStatus, setDiagnosticStatus] = useState<OperationState>({
    kind: "idle",
    message: "尚未刷新",
  });
  const [screenshotStatus, setScreenshotStatus] = useState<OperationState>({
    kind: "idle",
    message: "Alt+3 可框选区域截图",
  });
  const [selectionSnapshot, setSelectionSnapshot] = useState<SelectionSnapshot>(
    defaultSelectionSnapshot,
  );
  const [selectionPanelVisible, setSelectionPanelVisible] = useState(false);
  const [selectionOperation, setSelectionOperation] = useState<OperationState>({
    kind: "idle",
    message: "",
  });
  const [resultSnapshot, setResultSnapshot] =
    useState<ResultSnapshot>(defaultResultSnapshot);
  const [resultOperation, setResultOperation] = useState<OperationState>({
    kind: "idle",
    message: "",
  });
  const hasResult = false;
  const canUseSelection =
    selectionSnapshot.status === "captured" &&
    selectionSnapshot.text.trim().length > 0;
  const isSelectionPending = selectionOperation.kind === "pending";
  const retryTextAction =
    resultSnapshot.kind === "text_translate"
      ? "translate"
      : resultSnapshot.kind === "text_summarize"
        ? "summarize"
        : null;

  const refreshDiagnostics = () => {
    setDiagnosticStatus({ kind: "pending", message: "正在刷新诊断" });
    invoke<MainDiagnostics>("get_main_diagnostics")
      .then((report) => {
        setDiagnostics(report);
        setDiagnosticStatus({ kind: "success", message: "诊断已刷新" });
      })
      .catch((error) => {
        setDiagnosticStatus({
          kind: "error",
          message: typeof error === "string" ? error : "读取诊断失败",
        });
      });
  };

  const captureRegion = () => {
    setScreenshotStatus({ kind: "pending", message: "正在等待框选和菜单选择" });
    invoke<SelectionActionResult>("capture_region_to_clipboard")
      .then((result) => {
        setScreenshotStatus({ kind: "success", message: result.message });
      })
      .catch((error) => {
        setScreenshotStatus({
          kind: "error",
          message: typeof error === "string" ? error : "截图复制失败",
        });
      });
  };

  const loadSelectionSnapshot = (showPanel = false) => {
    invoke<SelectionSnapshot>("get_selection_snapshot")
      .then((snapshot) => {
        setSelectionSnapshot(snapshot);
        setSelectionOperation({ kind: "idle", message: "" });
        if (showPanel) {
          setSelectionPanelVisible(true);
        }
      })
      .catch(() => {
        setSelectionSnapshot({
          ...defaultSelectionSnapshot,
          status: "error",
          preview: "无法读取划词状态",
          message: "无法连接 QuickPick 核心，请重新打开应用",
          source: "ui",
        });
        setSelectionPanelVisible(true);
      });
  };

  const runMainSelectionAction = (
    command: "copy_selection_text" | "search_selection_text",
  ) => {
    if (!canUseSelection || isSelectionPending) {
      return;
    }

    setSelectionOperation({ kind: "pending", message: "正在处理划词" });
    invoke<SelectionActionResult>(command)
      .then((result) => {
        setSelectionOperation({ kind: "success", message: result.message });
      })
      .catch((error) => {
        setSelectionOperation({
          kind: "error",
          message: typeof error === "string" ? error : "划词操作失败",
        });
      });
  };

  const runMainTextAiAction = (action: "translate" | "summarize") => {
    if (!canUseSelection || isSelectionPending) {
      return;
    }

    setSelectionOperation({ kind: "pending", message: "正在请求 AI" });
    invoke<SelectionActionResult>("run_text_ai_action", { action })
      .then((result) => {
        setSelectionOperation({ kind: "success", message: result.message });
        loadResultSnapshot();
      })
      .catch((error) => {
        setSelectionOperation({
          kind: "error",
          message: typeof error === "string" ? error : "划词 AI 请求失败",
        });
      });
  };

  const loadResultSnapshot = () => {
    invoke<ResultSnapshot>("get_result_snapshot")
      .then((snapshot) => {
        setResultSnapshot(snapshot);
        setResultOperation({ kind: "idle", message: "" });
      })
      .catch(() => {
        setResultSnapshot({
          ...defaultResultSnapshot,
          status: "error",
          kind: "error",
          title: "结果读取失败",
          content: "无法读取当前结果状态",
          detail: "请稍后刷新主窗口。",
        });
      });
  };

  const retryResult = () => {
    if (
      retryTextAction === null ||
      !resultSnapshot.canRetry ||
      resultOperation.kind === "pending"
    ) {
      return;
    }

    setResultOperation({ kind: "pending", message: "正在重试" });
    invoke<SelectionActionResult>("run_text_ai_action", {
      action: retryTextAction,
    })
      .then((result) => {
        setResultOperation({ kind: "success", message: result.message });
        loadResultSnapshot();
      })
      .catch((error) => {
        setResultOperation({
          kind: "error",
          message: typeof error === "string" ? error : "重试失败",
        });
      });
  };

  const copyResult = () => {
    if (!resultSnapshot.canCopy || resultOperation.kind === "pending") {
      return;
    }

    setResultOperation({ kind: "pending", message: "正在复制结果" });
    invoke<SelectionActionResult>("copy_result_content")
      .then((result) => {
        setResultOperation({ kind: "success", message: result.message });
      })
      .catch((error) => {
        setResultOperation({
          kind: "error",
          message: typeof error === "string" ? error : "复制结果失败",
        });
      });
  };

  const clearResult = () => {
    setResultOperation({ kind: "pending", message: "正在清空结果" });
    invoke<SelectionActionResult>("clear_result_snapshot")
      .then((result) => {
        setResultSnapshot(defaultResultSnapshot);
        setResultOperation({ kind: "success", message: result.message });
      })
      .catch((error) => {
        setResultOperation({
          kind: "error",
          message: typeof error === "string" ? error : "清空结果失败",
        });
      });
  };

  useEffect(() => {
    refreshDiagnostics();
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen<ScreenshotStatusEvent>("screenshot-status-updated", (event) => {
      setScreenshotStatus({
        kind: event.payload.kind,
        message: event.payload.message,
      });
    }).then((dispose) => {
      unlisten = dispose;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  return (
    <main className="app-shell main-window">
      <section className="hero-band">
        <div>
          <p className="eyebrow">QuickPick</p>
          <h1>安静常驻，按键即取</h1>
          <p className="lede">
            Alt+3 可进入原生框选截图；Tauri/WebView 截图遮罩和预览子窗口仍保持暂停。
          </p>
        </div>
        <div className="status-tile" aria-label="核心状态">
          <span>核心状态</span>
          <strong>{coreStatus}</strong>
        </div>
      </section>

      <section className="panel-grid" aria-label="开发状态">
        {roadmap.map((item) => (
          <article className="surface-panel" key={item.label}>
            <span>{item.label}</span>
            <strong>{item.value}</strong>
          </article>
        ))}
      </section>

      <section className="workflow-panel" aria-label="首版能力边界">
        <div>
          <h2>当前验收点</h2>
          <p>
            Alt+2 会打开原生 Win32 划词弹窗；Alt+3 会打开原生 Win32 框选层；托盘设置、settings 验证、安全预览、
            窗口诊断和 Tauri/WebView 截图遮罩都不会触发。
          </p>
        </div>
        <div className="workflow-actions">
          <button
            className="primary-action"
            type="button"
            onClick={captureRegion}
            disabled={screenshotStatus.kind === "pending"}
          >
            区域截图
          </button>
          <button
            className="ghost-control"
            type="button"
            onClick={onCheckCore}
          >
            检查核心
          </button>
        </div>
        <p
          className={`screenshot-feedback screenshot-feedback-${screenshotStatus.kind}`}
          aria-live="polite"
        >
          {screenshotStatus.message}
        </p>
      </section>

      {selectionPanelVisible && (
        <section className="main-selection-panel" aria-label="划词弹窗">
          <header className="main-selection-header">
            <div>
              <p className="eyebrow">划词</p>
              <h2>文本操作</h2>
            </div>
            <span
              className={`selection-panel-status selection-panel-status-${selectionSnapshot.status}`}
            >
              {selectionStatusLabel[selectionSnapshot.status]}
            </span>
          </header>

          <div className="main-selection-body">
            <div className="selection-panel-preview">
              <span>当前文本</span>
              <p title={selectionSnapshot.preview}>
                {selectionSnapshot.preview || selectionSnapshot.message}
              </p>
              {selectionSnapshot.charCount > 0 && (
                <strong>{selectionSnapshot.charCount} 字符</strong>
              )}
            </div>

            <p className="selection-panel-detail">
              {selectionOperation.message || selectionSnapshot.message}
            </p>
          </div>

          <footer className="main-selection-actions">
            <button
              className="ghost-control"
              type="button"
              disabled={!canUseSelection || isSelectionPending}
              onClick={() => runMainSelectionAction("copy_selection_text")}
            >
              复制
            </button>
            <button
              className="ghost-control"
              type="button"
              disabled={!canUseSelection || isSelectionPending}
              onClick={() => runMainSelectionAction("search_selection_text")}
            >
              搜索
            </button>
            <button
              className="primary-action"
              type="button"
              disabled={!canUseSelection || isSelectionPending}
              onClick={() => runMainTextAiAction("translate")}
            >
              翻译
            </button>
            <button
              className="primary-action"
              type="button"
              disabled={!canUseSelection || isSelectionPending}
              onClick={() => runMainTextAiAction("summarize")}
            >
              总结
            </button>
            <button
              className="ghost-control"
              type="button"
              disabled={isSelectionPending}
              onClick={() => setSelectionPanelVisible(false)}
            >
              收起
            </button>
          </footer>
        </section>
      )}

      {hasResult && (
        <section className="main-result-panel" aria-label="AI 结果">
          <header className="main-result-header">
            <div>
              <p className="eyebrow">结果</p>
              <h2>{resultSnapshot.title}</h2>
            </div>
            <span className={`result-status result-status-${resultSnapshot.status}`}>
              {resultStatusLabel[resultSnapshot.status]}
            </span>
          </header>

          <div className="result-body main-result-body">
            {resultSnapshot.sourcePreview && (
              <div className="result-source">
                <span>来源</span>
                <p title={resultSnapshot.sourcePreview}>
                  {resultSnapshot.sourcePreview}
                </p>
                {resultSnapshot.sourceCharCount > 0 && (
                  <strong>{resultSnapshot.sourceCharCount} 字符</strong>
                )}
              </div>
            )}

            <article className="result-content main-result-content">
              <p>{resultSnapshot.content}</p>
            </article>

            <p className="result-detail">{resultSnapshot.detail}</p>
          </div>

          <footer className="result-footer">
            {resultOperation.message && (
              <span
                className={`result-feedback result-feedback-${resultOperation.kind}`}
                aria-live="polite"
              >
                {resultOperation.message}
              </span>
            )}
            <button
              className="ghost-control"
              type="button"
              disabled={
                !resultSnapshot.canCopy || resultOperation.kind === "pending"
              }
              onClick={copyResult}
            >
              复制结果
            </button>
            <button
              className="ghost-control"
              type="button"
              disabled={
                retryTextAction === null ||
                !resultSnapshot.canRetry ||
                resultOperation.kind === "pending"
              }
              onClick={retryResult}
            >
              重试
            </button>
            <button
              className="ghost-control"
              type="button"
              disabled={resultOperation.kind === "pending"}
              onClick={clearResult}
            >
              清空
            </button>
          </footer>
        </section>
      )}

      <MainAiConfigPanel />

      <section className="diagnostic-panel" aria-label="主窗口诊断">
        <div className="diagnostic-header">
          <div>
            <h2>主窗口诊断</h2>
            <p>
              {diagnostics?.summary ||
                "只在主窗口内读取状态，不创建新的子窗口。"}
            </p>
          </div>
          <button
            className="ghost-control"
            type="button"
            onClick={refreshDiagnostics}
            disabled={diagnosticStatus.kind === "pending"}
          >
            刷新诊断
          </button>
        </div>
        <div className="diagnostic-sections">
          {(diagnostics?.sections || []).map((section) => (
            <section className="diagnostic-section" key={section.title}>
              <div className="diagnostic-section-heading">
                <h3>{section.title}</h3>
                <p>{section.detail}</p>
              </div>
              <div className="diagnostic-grid">
                {section.items.map((item) => (
                  <article
                    className={`diagnostic-item diagnostic-item-${item.status}`}
                    key={`${section.title}-${item.label}`}
                  >
                    <span>{item.label}</span>
                    <strong>{item.value}</strong>
                  </article>
                ))}
              </div>
            </section>
          ))}
        </div>
        <p
          className={`diagnostic-feedback diagnostic-feedback-${diagnosticStatus.kind}`}
          aria-live="polite"
        >
          {diagnosticStatus.message}
        </p>
      </section>
    </main>
  );
}

function MainAiConfigPanel() {
  const [settings, setSettings] = useState<AppSettings>(defaultAppSettings);
  const [settingsStatus, setSettingsStatus] = useState<SettingsStatus>({
    kind: "loading",
    message: "正在读取 AI 配置",
  });
  const [textApiKeyInput, setTextApiKeyInput] = useState("");
  const [visionApiKeyInput, setVisionApiKeyInput] = useState("");
  const [textApiKeyConfigured, setTextApiKeyConfigured] = useState(false);
  const [visionApiKeyConfigured, setVisionApiKeyConfigured] = useState(false);
  const [textApiKeyStatus, setTextApiKeyStatus] = useState<SettingsStatus>({
    kind: "loading",
    message: "正在检查文本模型 Key",
  });
  const [visionApiKeyStatus, setVisionApiKeyStatus] = useState<SettingsStatus>({
    kind: "loading",
    message: "正在检查视觉模型 Key",
  });
  const apiKeyInput = textApiKeyInput;
  const setApiKeyInput = setTextApiKeyInput;
  const apiKeyConfigured = textApiKeyConfigured;
  const setApiKeyConfigured = setTextApiKeyConfigured;
  const apiKeyStatus = textApiKeyStatus;
  const setApiKeyStatus = setTextApiKeyStatus;

  useEffect(() => {
    applyDocumentAppearance(settings);
  }, [settings.themeMode, settings.windowEffect, settings.panelOpacity]);

  useEffect(() => {
    let isMounted = true;

    invoke<AppSettings>("get_app_settings")
      .then((value) => {
        if (isMounted) {
          setSettings(normalizeAppSettings(value));
          setSettingsStatus({ kind: "idle", message: "AI 配置已载入" });
        }
      })
      .catch((error) => {
        if (isMounted) {
          setSettings(defaultAppSettings);
          setSettingsStatus({
            kind: "error",
            message:
              typeof error === "string" ? error : "读取 AI 配置失败，已使用默认值",
          });
        }
      });

    invoke<ApiKeyStatus>("get_api_key_status", { scope: "text" })
      .then((value) => {
        if (isMounted) {
          setTextApiKeyConfigured(value.configured);
          setTextApiKeyStatus({
            kind: "idle",
            message: value.configured
              ? "文本模型 Key 已加密保存"
              : "文本模型 Key 未配置",
          });
        }
      })
      .catch((error) => {
        if (isMounted) {
          setTextApiKeyConfigured(false);
          setTextApiKeyStatus({
            kind: "error",
            message:
              typeof error === "string" ? error : "检查文本模型 Key 状态失败",
          });
        }
      });

    invoke<ApiKeyStatus>("get_api_key_status", { scope: "vision" })
      .then((value) => {
        if (isMounted) {
          setVisionApiKeyConfigured(value.configured);
          setVisionApiKeyStatus({
            kind: "idle",
            message: value.configured
              ? "视觉模型 Key 已加密保存"
              : "视觉模型 Key 未配置",
          });
        }
      })
      .catch((error) => {
        if (isMounted) {
          setVisionApiKeyConfigured(false);
          setVisionApiKeyStatus({
            kind: "error",
            message:
              typeof error === "string" ? error : "检查视觉模型 Key 状态失败",
          });
        }
      });

    return () => {
      isMounted = false;
    };
  }, []);

  const markSettingsDirty = (message = "AI 配置有未保存的修改") => {
    setSettingsStatus((current) =>
      current.kind === "loading" || current.kind === "saving"
        ? current
        : { kind: "idle", message },
    );
  };

  const updateSetting = <K extends keyof AppSettings>(
    key: K,
    value: AppSettings[K],
  ) => {
    setSettings((current) => ({ ...current, [key]: value }));
    markSettingsDirty();
  };

  const selectProvider = (providerId: string) => {
    setSettings((current) => applyAiProviderDefaults(current, providerId));
    markSettingsDirty("供应商默认值已填入，请保存配置");
  };

  const saveSettings = () => {
    if (settingsStatus.kind === "saving") {
      return;
    }

    const nextSettings = normalizeAppSettings(settings);
    setSettings(nextSettings);
    setSettingsStatus({ kind: "saving", message: "正在保存 AI 配置" });
    invoke<SelectionActionResult>("save_app_settings", { settings: nextSettings })
      .then((result) => {
        setSettingsStatus({ kind: "success", message: result.message });
      })
      .catch((error) => {
        setSettingsStatus({
          kind: "error",
          message: typeof error === "string" ? error : "保存 AI 配置失败",
        });
      });
  };

  const saveApiKey = () => {
    if (apiKeyStatus.kind === "saving") {
      return;
    }

    if (!apiKeyInput.trim()) {
      setApiKeyStatus({ kind: "error", message: "API Key 不能为空" });
      return;
    }

    const nextSettings = normalizeAppSettings(settings);
    setSettings(nextSettings);
    setSettingsStatus({ kind: "saving", message: "正在同步 AI 配置" });
    setApiKeyStatus({ kind: "saving", message: "正在加密保存 API Key" });
    invoke<SelectionActionResult>("save_app_settings", {
      settings: nextSettings,
    })
      .then(() => {
        setSettingsStatus({ kind: "success", message: "AI 配置已保存" });
        invoke<SelectionActionResult>("save_api_key", {
          apiKey: apiKeyInput,
        })
          .then((result) => {
            setApiKeyConfigured(true);
            setApiKeyInput("");
            setApiKeyStatus({ kind: "success", message: result.message });
          })
          .catch((error) => {
            setApiKeyStatus({
              kind: "error",
              message: typeof error === "string" ? error : "保存 API Key 失败",
            });
          });
      })
      .catch((error) => {
        setSettingsStatus({
          kind: "error",
          message: typeof error === "string" ? error : "同步 AI 配置失败",
        });
        setApiKeyStatus({
          kind: "error",
          message: "保存 API Key 前同步配置失败",
        });
      });
  };

  const clearApiKey = () => {
    if (apiKeyStatus.kind === "saving") {
      return;
    }

    setApiKeyStatus({ kind: "saving", message: "正在清除 API Key" });
    invoke<SelectionActionResult>("clear_api_key")
      .then((result) => {
        setApiKeyConfigured(false);
        setApiKeyInput("");
        setApiKeyStatus({ kind: "success", message: result.message });
      })
      .catch((error) => {
        setApiKeyStatus({
          kind: "error",
          message: typeof error === "string" ? error : "清除 API Key 失败",
        });
      });
  };

  return (
    <section className="main-ai-panel" aria-label="AI 配置">
      <header className="main-ai-header">
        <div>
          <p className="eyebrow">AI</p>
          <h2>截图识别配置</h2>
        </div>
        <span className="status-pill">
          {apiKeyConfigured ? "Key 已保存" : "Key 未配置"}
        </span>
      </header>

      <div className="main-ai-grid">
        <SettingField label="供应商">
          <GlassSelect
            ariaLabel="选择 AI 供应商"
            value={settings.aiProvider}
            options={aiProviderOptions}
            onChange={selectProvider}
          />
        </SettingField>
        <SettingField label="Base URL">
          <input
            className="setting-input"
            type="url"
            value={settings.aiBaseUrl}
            placeholder="https://api.openai.com/v1"
            spellCheck={false}
            onChange={(event) =>
              updateSetting("aiBaseUrl", event.currentTarget.value)
            }
          />
        </SettingField>
        <SettingField label="视觉模型">
          <input
            className="setting-input"
            type="text"
            value={settings.aiVisionModel}
            placeholder="gpt-4.1-mini"
            spellCheck={false}
            onChange={(event) =>
              updateSetting("aiVisionModel", event.currentTarget.value)
            }
          />
        </SettingField>
        <SettingField label="文本模型">
          <input
            className="setting-input"
            type="text"
            value={settings.aiTextModel}
            placeholder="gpt-4.1-mini"
            spellCheck={false}
            onChange={(event) =>
              updateSetting("aiTextModel", event.currentTarget.value)
            }
          />
        </SettingField>
        <SettingField label="超时秒数">
          <input
            className="setting-input"
            type="number"
            min={5}
            max={120}
            value={settings.aiTimeoutSeconds}
            onChange={(event) =>
              updateSetting(
                "aiTimeoutSeconds",
                Number(event.currentTarget.value),
              )
            }
          />
        </SettingField>
      </div>

      <div className="main-ai-actions">
        <button
          className="primary-action"
          type="button"
          onClick={saveSettings}
          disabled={settingsStatus.kind === "saving"}
        >
          保存配置
        </button>
        <span
          className={`settings-feedback settings-feedback-${settingsStatus.kind}`}
          aria-live="polite"
        >
          {settingsStatus.message}
        </span>
      </div>

      <div className="main-ai-key-row">
        <SettingField label="API Key">
          <input
            className="setting-input"
            type="password"
            value={apiKeyInput}
            placeholder={apiKeyConfigured ? "已加密保存" : "输入后加密保存"}
            spellCheck={false}
            onChange={(event) => setApiKeyInput(event.currentTarget.value)}
          />
        </SettingField>
        <button
          className="ghost-control"
          type="button"
          onClick={saveApiKey}
          disabled={apiKeyStatus.kind === "saving" || apiKeyInput.trim() === ""}
        >
          保存 Key
        </button>
        <button
          className="ghost-control"
          type="button"
          onClick={clearApiKey}
          disabled={apiKeyStatus.kind === "saving" || !apiKeyConfigured}
        >
          清除 Key
        </button>
      </div>

      <p
        className={`settings-feedback settings-feedback-${apiKeyStatus.kind}`}
        aria-live="polite"
      >
        {apiKeyStatus.message}
      </p>
    </section>
  );
}

function SettingsTitlebar() {
  useEffect(() => {
    getCurrentWindow().setDecorations(false).catch(() => {});
  }, []);

  const minimizeWindow = () => {
    getCurrentWindow().minimize();
  };

  const toggleMaximizeWindow = () => {
    getCurrentWindow().toggleMaximize();
  };

  const hideWindow = () => {
    getCurrentWindow().hide();
  };

  return (
    <div className="settings-titlebar" data-tauri-drag-region>
      <div className="settings-titlebar-brand" data-tauri-drag-region>
        <span className="settings-titlebar-mark" aria-hidden="true" />
        <span data-tauri-drag-region>QuickPick 设置</span>
      </div>
      <div className="settings-titlebar-controls">
        <button
          className="settings-titlebar-button"
          type="button"
          aria-label="最小化"
          onClick={minimizeWindow}
        >
          <span
            className="settings-titlebar-glyph settings-titlebar-glyph-minimize"
            aria-hidden="true"
          />
        </button>
        <button
          className="settings-titlebar-button"
          type="button"
          aria-label="最大化或还原"
          onClick={toggleMaximizeWindow}
        >
          <span
            className="settings-titlebar-glyph settings-titlebar-glyph-maximize"
            aria-hidden="true"
          />
        </button>
        <button
          className="settings-titlebar-button settings-titlebar-button-close"
          type="button"
          aria-label="关闭设置"
          onClick={hideWindow}
        >
          <span
            className="settings-titlebar-glyph settings-titlebar-glyph-close"
            aria-hidden="true"
          />
        </button>
      </div>
    </div>
  );
}

function GlassSelect<T extends string>({
  value,
  options,
  onChange,
  ariaLabel,
}: {
  value: T;
  options: readonly GlassSelectOption<T>[];
  onChange: (value: T) => void;
  ariaLabel: string;
}) {
  const [open, setOpen] = useState(false);
  const [menuStyle, setMenuStyle] = useState<CSSProperties>({});
  const triggerRef = useRef<HTMLButtonElement | null>(null);
  const menuRef = useRef<HTMLDivElement | null>(null);
  const selected = options.find((option) => option.id === value) || options[0];

  const selectOption = (nextValue: T) => {
    onChange(nextValue);
    setOpen(false);
  };

  const updateMenuPosition = () => {
    const trigger = triggerRef.current;
    if (!trigger) {
      return;
    }

    const rect = trigger.getBoundingClientRect();
    setMenuStyle({
      left: rect.left,
      top: rect.bottom + 6,
      width: rect.width,
    });
  };

  useEffect(() => {
    if (!open) {
      return;
    }

    updateMenuPosition();
    const closeOnOutsideMouseDown = (event: MouseEvent) => {
      const target = event.target;
      if (!(target instanceof Node)) {
        return;
      }
      if (
        triggerRef.current?.contains(target) ||
        menuRef.current?.contains(target)
      ) {
        return;
      }
      setOpen(false);
    };
    const refreshPosition = () => updateMenuPosition();

    document.addEventListener("mousedown", closeOnOutsideMouseDown, true);
    window.addEventListener("resize", refreshPosition);
    window.addEventListener("scroll", refreshPosition, true);

    return () => {
      document.removeEventListener("mousedown", closeOnOutsideMouseDown, true);
      window.removeEventListener("resize", refreshPosition);
      window.removeEventListener("scroll", refreshPosition, true);
    };
  }, [open]);

  return (
    <div className={`glass-select ${open ? "glass-select-open" : ""}`}>
      <button
        ref={triggerRef}
        className="glass-select-trigger setting-input"
        type="button"
        aria-label={ariaLabel}
        aria-haspopup="listbox"
        aria-expanded={open}
        onClick={() => setOpen((current) => !current)}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            setOpen(false);
          }
        }}
      >
        <span>{selected.label}</span>
        <span className="glass-select-arrow" aria-hidden="true" />
      </button>
      {open &&
        createPortal(
          <div
            className="glass-select-menu"
            role="listbox"
            ref={menuRef}
            style={menuStyle}
          >
          {options.map((option) => (
            <button
              className={`glass-select-option ${
                option.id === value ? "glass-select-option-active" : ""
              }`}
              type="button"
              role="option"
              aria-selected={option.id === value}
              key={option.id}
              onMouseDown={(event) => event.preventDefault()}
              onClick={() => selectOption(option.id)}
            >
              {option.label}
            </button>
          ))}
          </div>,
          document.body,
        )}
    </div>
  );
}

function SettingsWindow({ coreStatus }: { coreStatus: string }) {
  const settings = useSettingsStore((state) => state.settings);
  const settingsStatus = useSettingsStore((state) => state.status);
  const setSettings = useSettingsStore((state) => state.setSettings);
  const setSettingsStatus = useSettingsStore((state) => state.setStatus);
  const sectionsRef = useRef<HTMLDivElement | null>(null);
  const navRef = useRef<HTMLElement | null>(null);
  const navIndicatorRef = useRef<HTMLSpanElement | null>(null);
  const navButtonRefs = useRef<
    Partial<Record<SettingsSectionId, HTMLButtonElement | null>>
  >({});
  const navScrollLockRef = useRef<number | null>(null);
  const [capturingHotkey, setCapturingHotkey] =
    useState<HotkeySettingKey | null>(null);
  const [activeSettingsSection, setActiveSettingsSection] =
    useState<SettingsSectionId>("general");
  const [textApiKeyInput, setTextApiKeyInput] = useState("");
  const [visionApiKeyInput, setVisionApiKeyInput] = useState("");
  const [textApiKeyConfigured, setTextApiKeyConfigured] = useState(false);
  const [visionApiKeyConfigured, setVisionApiKeyConfigured] = useState(false);
  const [textApiKeyStatus, setTextApiKeyStatus] = useState<SettingsStatus>({
    kind: "loading",
    message: "正在检查文本模型 Key",
  });
  const [visionApiKeyStatus, setVisionApiKeyStatus] = useState<SettingsStatus>({
    kind: "loading",
    message: "正在检查视觉模型 Key",
  });
  const [inputApiKeyInput, setInputApiKeyInput] = useState("");
  const [inputApiKeyConfigured, setInputApiKeyConfigured] = useState(false);
  const [inputApiKeyStatus, setInputApiKeyStatus] = useState<SettingsStatus>({
    kind: "loading",
    message: "正在检查输入模型 Key",
  });

  const loadInputApiKeyStatus = () => {
    invoke<ApiKeyStatus>("get_api_key_status", { scope: "input" })
      .then((value) => {
        setInputApiKeyConfigured(value.configured);
        setInputApiKeyStatus({
          kind: "idle",
          message: value.configured
            ? "输入模型 Key 已加密保存"
            : "输入模型 Key 未配置",
        });
      })
      .catch((error) => {
        setInputApiKeyConfigured(false);
        setInputApiKeyStatus({
          kind: "error",
          message:
            typeof error === "string" ? error : "检查输入模型 Key 状态失败",
        });
      });
  };

  useEffect(() => {
    loadInputApiKeyStatus();
  }, []);

  useEffect(() => {
    applyDocumentAppearance(settings);

    if (settings.themeMode !== "system") {
      return;
    }

    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const updateAppearance = () => applyDocumentAppearance(settings);

    media.addEventListener("change", updateAppearance);

    return () => {
      media.removeEventListener("change", updateAppearance);
    };
  }, [settings.themeMode, settings.windowEffect, settings.panelOpacity]);

  useEffect(() => {
    let isMounted = true;

    invoke<AppSettings>("get_app_settings")
      .then((value) => {
        if (isMounted) {
          setSettings(normalizeAppSettings(value));
          setSettingsStatus({ kind: "idle", message: "设置已载入" });
        }
      })
      .catch((error) => {
        if (isMounted) {
          setSettings(defaultAppSettings);
          setSettingsStatus({
            kind: "error",
            message:
              typeof error === "string" ? error : "读取设置失败，已使用默认值",
          });
        }
      });

    invoke<ApiKeyStatus>("get_api_key_status", { scope: "text" })
      .then((value) => {
        if (isMounted) {
          setTextApiKeyConfigured(value.configured);
          setTextApiKeyStatus({
            kind: "idle",
            message: value.configured
              ? "文本模型 Key 已加密保存"
              : "文本模型 Key 未配置",
          });
        }
      })
      .catch((error) => {
        if (isMounted) {
          setTextApiKeyConfigured(false);
          setTextApiKeyStatus({
            kind: "error",
            message:
              typeof error === "string" ? error : "检查文本模型 Key 状态失败",
          });
        }
      });

    invoke<ApiKeyStatus>("get_api_key_status", { scope: "vision" })
      .then((value) => {
        if (isMounted) {
          setVisionApiKeyConfigured(value.configured);
          setVisionApiKeyStatus({
            kind: "idle",
            message: value.configured
              ? "视觉模型 Key 已加密保存"
              : "视觉模型 Key 未配置",
          });
        }
      })
      .catch((error) => {
        if (isMounted) {
          setVisionApiKeyConfigured(false);
          setVisionApiKeyStatus({
            kind: "error",
            message:
              typeof error === "string" ? error : "检查视觉模型 Key 状态失败",
          });
        }
      });

    return () => {
      isMounted = false;
    };
  }, []);

  const markSettingsDirty = (message = "有未保存的修改") => {
    setSettingsStatus((current) =>
      current.kind === "loading" || current.kind === "saving"
        ? current
        : { kind: "idle", message },
    );
  };

  const updateSetting = <K extends keyof AppSettings>(
    key: K,
    value: AppSettings[K],
  ) => {
    setSettings((current) => ({ ...current, [key]: value }));
    markSettingsDirty();
  };

  const updateUiLanguage = (value: UiLanguage) => {
    const target = targetLanguageForUiLanguage(value);
    setSettings((current) => ({
      ...current,
      uiLanguage: value,
      translationTargetLanguage: target,
      inputTranslateTargetLanguage: target,
    }));
    markSettingsDirty();
  };

  const setHotkeyCaptureMode = (enabled: boolean) => {
    invoke<SelectionActionResult>("set_hotkey_capture_mode", { enabled }).catch(
      () => {},
    );
  };

  const startHotkeyCapture = (key: HotkeySettingKey) => {
    setCapturingHotkey(key);
    setHotkeyCaptureMode(true);
    setSettingsStatus({
      kind: "idle",
      message: "请直接按下新的快捷键组合，Esc 取消",
    });
  };

  const finishHotkeyCapture = (message: string) => {
    setCapturingHotkey(null);
    setHotkeyCaptureMode(false);
    setSettingsStatus({ kind: "idle", message });
  };

  const captureHotkey = (
    key: HotkeySettingKey,
    event: ReactKeyboardEvent<HTMLButtonElement>,
  ) => {
    if (capturingHotkey !== key) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();

    if (event.key === "Escape") {
      finishHotkeyCapture("已取消快捷键录制");
      return;
    }
    if (modifierKeyNames.has(event.key)) {
      return;
    }
    if (!event.altKey) {
      setSettingsStatus({
        kind: "error",
        message: "快捷键必须包含 Alt，请重新按下组合键",
      });
      return;
    }

    const shortcut = shortcutFromKeyboardEvent(event);
    if (!shortcut) {
      setSettingsStatus({
        kind: "error",
        message: "这个按键暂不支持，请换一个组合键",
      });
      return;
    }

    const conflictWith =
      key === "settingsHotkey"
        ? "设置"
        : key === "selectionHotkey"
        ? "划词和截图"
        : key === "screenshotHotkey"
          ? "截图和输入翻译"
          : "输入翻译和划词";
    if (
      (key !== "settingsHotkey" &&
        sameShortcut(shortcut, settings.settingsHotkey)) ||
      (key !== "selectionHotkey" &&
        sameShortcut(shortcut, settings.selectionHotkey)) ||
      (key !== "screenshotHotkey" &&
        sameShortcut(shortcut, settings.screenshotHotkey)) ||
      (key !== "inputTranslateHotkey" &&
        sameShortcut(shortcut, settings.inputTranslateHotkey))
    ) {
      setSettingsStatus({
        kind: "error",
        message: `${conflictWith}不能使用同一个快捷键`,
      });
      return;
    }

    setSettings((current) => ({ ...current, [key]: shortcut }));
    finishHotkeyCapture("快捷键已记录，请保存设置");
  };

  useEffect(() => {
    return () => {
      setHotkeyCaptureMode(false);
    };
  }, []);

  const saveSettings = () => {
    if (settingsStatus.kind === "saving") {
      return;
    }

    const nextSettings = normalizeAppSettings(settings);

    setSettingsStatus({ kind: "saving", message: "正在保存" });
    invoke<SelectionActionResult>("save_app_settings", {
      settings: nextSettings,
    })
      .then((result) => {
        setSettings(nextSettings);
        setSettingsStatus({ kind: "success", message: result.message });
      })
      .catch((error) => {
        setSettingsStatus({
          kind: "error",
          message: typeof error === "string" ? error : "保存设置失败",
        });
      });
  };

  const resetDefaultSettings = () => {
    if (settingsStatus.kind === "saving") {
      return;
    }

    setSettingsStatus({ kind: "saving", message: "正在重置默认设置" });
    const nextSettings = normalizeAppSettings(defaultAppSettings);
    invoke<SelectionActionResult>("save_app_settings", {
      settings: nextSettings,
    })
      .then((result) => {
        setSettings(nextSettings);
        setSettingsStatus({ kind: "success", message: result.message });
      })
      .catch((error) => {
        setSettingsStatus({
          kind: "error",
          message:
            typeof error === "string" ? error : "重置默认设置失败",
        });
      });
  };

  const selectTextProvider = (providerId: string) => {
    setSettings((current) => applyTextAiProviderDefaults(current, providerId));
    markSettingsDirty("文本模型供应商默认值已填入，请保存设置");
  };

  const selectVisionProvider = (providerId: string) => {
    setSettings((current) => applyVisionAiProviderDefaults(current, providerId));
    markSettingsDirty("视觉模型供应商默认值已填入，请保存设置");
  };

  const selectInputProvider = (providerId: string) => {
    setSettings((current) => applyInputAiProviderDefaults(current, providerId));
    markSettingsDirty("输入模型供应商默认值已填入，请保存设置");
  };

  const updateTextApiKeyInput = (value: string) => {
    setTextApiKeyInput(value);
    if (textApiKeyStatus.kind === "success" || textApiKeyStatus.kind === "error") {
      setTextApiKeyStatus({
        kind: "idle",
        message: textApiKeyConfigured
          ? "输入新 Key 后保存会替换文本模型 Key"
          : "文本模型 Key 未配置",
      });
    }
  };

  const updateVisionApiKeyInput = (value: string) => {
    setVisionApiKeyInput(value);
    if (visionApiKeyStatus.kind === "success" || visionApiKeyStatus.kind === "error") {
      setVisionApiKeyStatus({
        kind: "idle",
        message: visionApiKeyConfigured
          ? "输入新 Key 后保存会替换视觉模型 Key"
          : "视觉模型 Key 未配置",
      });
    }
  };

  const updateInputApiKeyInput = (value: string) => {
    setInputApiKeyInput(value);
    if (inputApiKeyStatus.kind === "success" || inputApiKeyStatus.kind === "error") {
      setInputApiKeyStatus({
        kind: "idle",
        message: inputApiKeyConfigured
          ? "输入新 Key 后保存会替换输入模型 Key"
          : "输入模型 Key 未配置",
      });
    }
  };

  const saveScopedApiKey = (scope: "text" | "vision" | "input") => {
    const isText = scope === "text";
    const isVision = scope === "vision";
    const input = isText
      ? textApiKeyInput
      : isVision
        ? visionApiKeyInput
        : inputApiKeyInput;
    const status = isText
      ? textApiKeyStatus
      : isVision
        ? visionApiKeyStatus
        : inputApiKeyStatus;
    const setInput = isText
      ? setTextApiKeyInput
      : isVision
        ? setVisionApiKeyInput
        : setInputApiKeyInput;
    const setConfigured = isText
      ? setTextApiKeyConfigured
      : isVision
        ? setVisionApiKeyConfigured
        : setInputApiKeyConfigured;
    const setStatus = isText
      ? setTextApiKeyStatus
      : isVision
        ? setVisionApiKeyStatus
        : setInputApiKeyStatus;
    const label = isText
      ? "文本模型 Key"
      : isVision
        ? "视觉模型 Key"
        : "输入模型 Key";

    if (status.kind === "saving") {
      return;
    }

    if (!input.trim()) {
      setStatus({ kind: "error", message: `${label} 不能为空` });
      return;
    }

    const nextSettings = normalizeAppSettings(settings);
    setSettings(nextSettings);
    setSettingsStatus({ kind: "saving", message: "正在同步 AI 设置" });
    setStatus({ kind: "saving", message: "正在加密保存" });
    invoke<SelectionActionResult>("save_app_settings", {
      settings: nextSettings,
    })
      .then(() => {
        setSettingsStatus({ kind: "success", message: "设置已保存" });
        invoke<SelectionActionResult>("save_api_key", {
          apiKey: input,
          scope,
        })
          .then((result) => {
            setInput("");
            setConfigured(true);
            setStatus({ kind: "success", message: result.message });
          })
          .catch((error) => {
            setStatus({
              kind: "error",
              message: typeof error === "string" ? error : `保存${label}失败`,
            });
          });
      })
      .catch((error) => {
        setSettingsStatus({
          kind: "error",
          message: typeof error === "string" ? error : "同步 AI 设置失败",
        });
        setStatus({
          kind: "error",
          message: `保存${label}前同步设置失败`,
        });
      });
  };

  const clearScopedApiKey = (scope: "text" | "vision" | "input") => {
    const isText = scope === "text";
    const isVision = scope === "vision";
    const status = isText
      ? textApiKeyStatus
      : isVision
        ? visionApiKeyStatus
        : inputApiKeyStatus;
    const setInput = isText
      ? setTextApiKeyInput
      : isVision
        ? setVisionApiKeyInput
        : setInputApiKeyInput;
    const setConfigured = isText
      ? setTextApiKeyConfigured
      : isVision
        ? setVisionApiKeyConfigured
        : setInputApiKeyConfigured;
    const setStatus = isText
      ? setTextApiKeyStatus
      : isVision
        ? setVisionApiKeyStatus
        : setInputApiKeyStatus;
    const label = isText
      ? "文本模型 Key"
      : isVision
        ? "视觉模型 Key"
        : "输入模型 Key";

    if (status.kind === "saving") {
      return;
    }

    setStatus({ kind: "saving", message: "正在清除" });
    invoke<SelectionActionResult>("clear_api_key", { scope })
      .then((result) => {
        setInput("");
        setConfigured(false);
        setStatus({ kind: "success", message: result.message });
      })
      .catch((error) => {
        setStatus({
          kind: "error",
          message: typeof error === "string" ? error : `清除${label}失败`,
        });
      });
  };

  const scrollToSettingsSection = (sectionId: SettingsSectionId) => {
    const container = sectionsRef.current;
    const section = document.getElementById(sectionId);

    if (!container || !section) {
      return;
    }

    setActiveSettingsSection(sectionId);
    moveSettingsNavIndicator(sectionId);
    if (navScrollLockRef.current !== null) {
      window.clearTimeout(navScrollLockRef.current);
    }
    navScrollLockRef.current = window.setTimeout(() => {
      navScrollLockRef.current = null;
    }, 120);

    const containerRect = container.getBoundingClientRect();
    const sectionRect = section.getBoundingClientRect();
    const targetTop =
      sectionRect.top - containerRect.top + container.scrollTop;
    container.scrollTo({
      top: Math.max(0, targetTop),
      behavior: "auto",
    });
  };

  const updateActiveSettingsSection = () => {
    const container = sectionsRef.current;
    if (!container || navScrollLockRef.current !== null) {
      return;
    }

    const scrollTop = container.scrollTop + 28;
    const containerTop = container.getBoundingClientRect().top;
    const nextSection = settingsNavItems.reduce<SettingsSectionId>(
      (current, item) => {
        const section = document.getElementById(item.id);
        if (!section) {
          return current;
        }

        const sectionTop =
          section.getBoundingClientRect().top - containerTop + container.scrollTop;
        if (sectionTop <= scrollTop) {
          return item.id;
        }

        return current;
      },
      "general",
    );
    setActiveSettingsSection(nextSection);
  };

  const moveSettingsNavIndicator = (sectionId = activeSettingsSection) => {
    const button = navButtonRefs.current[sectionId];
    const indicator = navIndicatorRef.current;

    if (!button || !indicator) {
      return;
    }

    indicator.style.width = `${button.offsetWidth}px`;
    indicator.style.height = `${button.offsetHeight}px`;
    indicator.style.transform = `translate3d(${button.offsetLeft}px, ${button.offsetTop}px, 0)`;
  };

  useEffect(() => {
    moveSettingsNavIndicator(activeSettingsSection);
  }, [activeSettingsSection, settings.themeMode, settings.windowEffect, settings.panelOpacity]);

  useEffect(() => {
    const onResize = () => moveSettingsNavIndicator(activeSettingsSection);
    window.addEventListener("resize", onResize);

    return () => {
      window.removeEventListener("resize", onResize);
      if (navScrollLockRef.current !== null) {
        window.clearTimeout(navScrollLockRef.current);
      }
    };
  }, [activeSettingsSection]);

  return (
    <main className="app-shell settings-window">
      <SettingsTitlebar />

      <div className="settings-shell">
        <section className="settings-layout">
          <nav className="settings-nav" aria-label="设置分类" ref={navRef}>
          <span
            className="settings-nav-indicator"
            ref={navIndicatorRef}
            aria-hidden="true"
          />
          {settingsNavItems.map((item) => (
            <button
              className={
                item.id === activeSettingsSection ? "settings-nav-active" : ""
              }
              type="button"
              key={item.id}
              ref={(element) => {
                navButtonRefs.current[item.id] = element;
              }}
              aria-current={
                item.id === activeSettingsSection ? "page" : undefined
              }
              onPointerDown={(event) => {
                if (event.button !== 0) {
                  return;
                }
                setActiveSettingsSection(item.id);
                moveSettingsNavIndicator(item.id);
              }}
              onClick={() => scrollToSettingsSection(item.id)}
            >
              {item.label}
            </button>
          ))}
          </nav>

          <div
            className="settings-sections"
            ref={sectionsRef}
            onScroll={updateActiveSettingsSection}
          >
          <SettingsSection id="general" title="通用">
            <SettingField label="重置默认设置">
              <button
                className="ghost-control"
                type="button"
                onClick={resetDefaultSettings}
                disabled={settingsStatus.kind === "saving"}
              >
                重置默认设置
              </button>
            </SettingField>
            <SettingField label="开机自启">
              <label className="toggle-control">
                <input
                  type="checkbox"
                  checked={settings.autostartEnabled}
                  onChange={(event) =>
                    updateSetting("autostartEnabled", event.currentTarget.checked)
                  }
                />
                <span>{settings.autostartEnabled ? "已启用" : "已关闭"}</span>
              </label>
            </SettingField>
            <SettingField label="语言">
              <GlassSelect
                ariaLabel="语言"
                value={settings.uiLanguage}
                options={uiLanguageOptions}
                onChange={(value) => updateUiLanguage(value as UiLanguage)}
              />
            </SettingField>
            <SettingField label="AI 请求超时">
              <input
                className="setting-input setting-input-number"
                type="number"
                min={5}
                max={120}
                value={settings.aiTimeoutSeconds}
                onChange={(event) =>
                  updateSetting(
                    "aiTimeoutSeconds",
                    Number(event.currentTarget.value),
                  )
                }
              />
            </SettingField>
          </SettingsSection>
          <SettingsSection id="hotkeys" title="快捷键">
            <SettingField label="设置">
              <HotkeyCaptureButton
                label="设置"
                value={settings.settingsHotkey}
                placeholder="Alt+1"
                active={capturingHotkey === "settingsHotkey"}
                onStart={() => startHotkeyCapture("settingsHotkey")}
                onCancel={() => finishHotkeyCapture("已取消快捷键录制")}
                onKeyDown={(event) => captureHotkey("settingsHotkey", event)}
              />
            </SettingField>
            <SettingField label="划词菜单">
              <HotkeyCaptureButton
                label="划词菜单"
                value={settings.selectionHotkey}
                placeholder="Alt+2"
                active={capturingHotkey === "selectionHotkey"}
                onStart={() => startHotkeyCapture("selectionHotkey")}
                onCancel={() => finishHotkeyCapture("已取消快捷键录制")}
                onKeyDown={(event) => captureHotkey("selectionHotkey", event)}
              />
            </SettingField>
            <SettingField label="区域截图">
              <HotkeyCaptureButton
                label="区域截图"
                value={settings.screenshotHotkey}
                placeholder="Alt+3"
                active={capturingHotkey === "screenshotHotkey"}
                onStart={() => startHotkeyCapture("screenshotHotkey")}
                onCancel={() => finishHotkeyCapture("已取消快捷键录制")}
                onKeyDown={(event) => captureHotkey("screenshotHotkey", event)}
              />
            </SettingField>
            <SettingField label="输入翻译">
              <HotkeyCaptureButton
                label="输入翻译"
                value={settings.inputTranslateHotkey}
                placeholder="Alt+4"
                active={capturingHotkey === "inputTranslateHotkey"}
                onStart={() => startHotkeyCapture("inputTranslateHotkey")}
                onCancel={() => finishHotkeyCapture("已取消快捷键录制")}
                onKeyDown={(event) =>
                  captureHotkey("inputTranslateHotkey", event)
                }
              />
            </SettingField>
          </SettingsSection>
          <SettingsSection id="ai-selection" title="划词模型">
            <SettingField label="文本供应商">
              <GlassSelect
                ariaLabel="选择文本模型供应商"
                value={settings.textAiProvider}
                options={textAiProviderOptions}
                onChange={selectTextProvider}
              />
            </SettingField>
            <SettingField label="文本 Base URL">
              <input
                className="setting-input"
                type="url"
                value={settings.textAiBaseUrl}
                placeholder="https://api.deepseek.com"
                spellCheck={false}
                onChange={(event) =>
                  updateSetting("textAiBaseUrl", event.currentTarget.value)
                }
              />
            </SettingField>
            <SettingField label="文本模型">
              <input
                className="setting-input"
                type="text"
                value={settings.textAiModel}
                placeholder="deepseek-v4-flash"
                spellCheck={false}
                onChange={(event) =>
                  updateSetting("textAiModel", event.currentTarget.value)
                }
              />
            </SettingField>
            <SettingField label="文本 API Key">
              <div className="secret-control">
                <input
                  className="setting-input"
                  type="password"
                  value={textApiKeyInput}
                  placeholder={
                    textApiKeyConfigured
                      ? "文本模型 Key 已加密保存，输入新 Key 可替换"
                      : "输入文本模型 API Key"
                  }
                  spellCheck={false}
                  autoComplete="off"
                  onChange={(event) =>
                    updateTextApiKeyInput(event.currentTarget.value)
                  }
                />
                <button
                  className="ghost-control"
                  type="button"
                  onClick={() => saveScopedApiKey("text")}
                  disabled={
                    textApiKeyStatus.kind === "loading" ||
                    textApiKeyStatus.kind === "saving"
                  }
                >
                  保存文本 Key
                </button>
                <button
                  className="ghost-control"
                  type="button"
                  onClick={() => clearScopedApiKey("text")}
                  disabled={
                    !textApiKeyConfigured ||
                    textApiKeyStatus.kind === "loading" ||
                    textApiKeyStatus.kind === "saving"
                  }
                >
                  清除
                </button>
              </div>
            </SettingField>
            <TextRow
              label="文本 Key 状态"
              value={textApiKeyConfigured ? "已加密保存" : "未配置"}
            />
            <div className="settings-key-feedback">
              <span
                className={`settings-feedback settings-feedback-${textApiKeyStatus.kind}`}
              >
                {textApiKeyStatus.message}
              </span>
            </div>
          </SettingsSection>
          <SettingsSection id="ai-screenshot" title="截图模型">
            <SettingField label="视觉供应商">
              <GlassSelect
                ariaLabel="选择视觉模型供应商"
                value={settings.visionAiProvider}
                options={visionAiProviderOptions}
                onChange={selectVisionProvider}
              />
            </SettingField>
            <SettingField label="视觉 Base URL">
              <input
                className="setting-input"
                type="url"
                value={settings.visionAiBaseUrl}
                placeholder="https://api.xiaomimimo.com/v1"
                spellCheck={false}
                onChange={(event) =>
                  updateSetting("visionAiBaseUrl", event.currentTarget.value)
                }
              />
            </SettingField>
            <SettingField label="视觉模型">
              <input
                className="setting-input"
                type="text"
                value={settings.visionAiModel}
                placeholder="mimo-v2.5"
                spellCheck={false}
                onChange={(event) =>
                  updateSetting("visionAiModel", event.currentTarget.value)
                }
              />
            </SettingField>
            <SettingField label="视觉 API Key">
              <div className="secret-control">
                <input
                  className="setting-input"
                  type="password"
                  value={visionApiKeyInput}
                  placeholder={
                    visionApiKeyConfigured
                      ? "视觉模型 Key 已加密保存，输入新 Key 可替换"
                      : "输入视觉模型 API Key"
                  }
                  spellCheck={false}
                  autoComplete="off"
                  onChange={(event) =>
                    updateVisionApiKeyInput(event.currentTarget.value)
                  }
                />
                <button
                  className="ghost-control"
                  type="button"
                  onClick={() => saveScopedApiKey("vision")}
                  disabled={
                    visionApiKeyStatus.kind === "loading" ||
                    visionApiKeyStatus.kind === "saving"
                  }
                >
                  保存视觉 Key
                </button>
                <button
                  className="ghost-control"
                  type="button"
                  onClick={() => clearScopedApiKey("vision")}
                  disabled={
                    !visionApiKeyConfigured ||
                    visionApiKeyStatus.kind === "loading" ||
                    visionApiKeyStatus.kind === "saving"
                  }
                >
                  清除
                </button>
              </div>
            </SettingField>
            <TextRow
              label="视觉 Key 状态"
              value={visionApiKeyConfigured ? "已加密保存" : "未配置"}
            />
            <div className="settings-key-feedback">
              <span
                className={`settings-feedback settings-feedback-${visionApiKeyStatus.kind}`}
              >
                {visionApiKeyStatus.message}
              </span>
            </div>
          </SettingsSection>
          <SettingsSection id="ai-input" title="输入模型">
            <SettingField label="输入供应商">
              <GlassSelect
                ariaLabel="选择输入模型供应商"
                value={settings.inputAiProvider}
                options={inputAiProviderOptions}
                onChange={selectInputProvider}
              />
            </SettingField>
            <SettingField label="输入 Base URL">
              <input
                className="setting-input"
                type="url"
                value={settings.inputAiBaseUrl}
                placeholder="https://api.deepseek.com"
                spellCheck={false}
                onChange={(event) =>
                  updateSetting("inputAiBaseUrl", event.currentTarget.value)
                }
              />
            </SettingField>
            <SettingField label="输入模型">
              <input
                className="setting-input"
                type="text"
                value={settings.inputAiModel}
                placeholder="deepseek-v4-flash"
                spellCheck={false}
                onChange={(event) =>
                  updateSetting("inputAiModel", event.currentTarget.value)
                }
              />
            </SettingField>
            <SettingField label="输入 API Key">
              <div className="secret-control">
                <input
                  className="setting-input"
                  type="password"
                  value={inputApiKeyInput}
                  placeholder={
                    inputApiKeyConfigured
                      ? "输入模型 Key 已加密保存，输入新 Key 可替换"
                      : "输入输入模型 API Key"
                  }
                  spellCheck={false}
                  autoComplete="off"
                  onChange={(event) =>
                    updateInputApiKeyInput(event.currentTarget.value)
                  }
                />
                <button
                  className="ghost-control"
                  type="button"
                  onClick={() => saveScopedApiKey("input")}
                  disabled={
                    inputApiKeyStatus.kind === "loading" ||
                    inputApiKeyStatus.kind === "saving"
                  }
                >
                  保存输入 Key
                </button>
                <button
                  className="ghost-control"
                  type="button"
                  onClick={() => clearScopedApiKey("input")}
                  disabled={
                    !inputApiKeyConfigured ||
                    inputApiKeyStatus.kind === "loading" ||
                    inputApiKeyStatus.kind === "saving"
                  }
                >
                  清除
                </button>
              </div>
            </SettingField>
            <TextRow
              label="输入 Key 状态"
              value={inputApiKeyConfigured ? "已加密保存" : "未配置"}
            />
            <div className="settings-key-feedback">
              <span
                className={`settings-feedback settings-feedback-${inputApiKeyStatus.kind}`}
              >
                {inputApiKeyStatus.message}
              </span>
            </div>
          </SettingsSection>
          <SettingsSection id="appearance" title="外观">
            <SettingField label="主题">
              <GlassSelect
                ariaLabel="选择外观主题"
                value={settings.themeMode}
                options={themeModeOptions}
                onChange={(value) => updateSetting("themeMode", value)}
              />
            </SettingField>
            <SettingField label="窗口效果">
              <GlassSelect
                ariaLabel="选择窗口效果"
                value={settings.windowEffect}
                options={windowEffectOptions}
                onChange={(value) => updateSetting("windowEffect", value)}
              />
            </SettingField>
            <SettingField label="不透明度">
              <div className="opacity-slider-row">
                <input
                  className="setting-input opacity-slider"
                  type="range"
                  min={30}
                  max={100}
                  step={5}
                  value={settings.panelOpacity}
                  disabled={settings.windowEffect === "mica"}
                  onChange={(event) =>
                    updateSetting("panelOpacity", Number(event.currentTarget.value))
                  }
                />
                <strong className="opacity-slider-value">
                  {settings.panelOpacity}%
                </strong>
              </div>
            </SettingField>
          </SettingsSection>
          <SettingsSection id="privacy" title="隐私">
            <TextRow label="内容历史" value="默认不保存文本、截图和 AI 结果" />
            <TextRow label="API Key" value="使用 Windows DPAPI 本机加密保存" />
          </SettingsSection>
        </div>
      </section>
      <footer className="settings-savebar" aria-label="保存设置">
        <span
          className={`settings-feedback settings-feedback-${settingsStatus.kind}`}
          aria-live="polite"
        >
          {settingsStatus.message}
        </span>
        <button
          className="primary-action settings-save-button"
          type="button"
          onClick={saveSettings}
          disabled={
            settingsStatus.kind === "loading" || settingsStatus.kind === "saving"
          }
        >
          保存设置
        </button>
      </footer>
      </div>
    </main>
  );
}

function SettingsSection({
  id,
  title,
  children,
}: {
  id: string;
  title: string;
  children: React.ReactNode;
}) {
  return (
    <section className="settings-section" id={id}>
      <h2>{title}</h2>
      <div className="setting-list">{children}</div>
    </section>
  );
}

function SettingField({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div className="setting-field">
      <span>{label}</span>
      <div className="setting-control">{children}</div>
    </div>
  );
}

function HotkeyCaptureButton({
  label,
  value,
  placeholder,
  active,
  onStart,
  onCancel,
  onKeyDown,
}: {
  label: string;
  value: string;
  placeholder: string;
  active: boolean;
  onStart: () => void;
  onCancel: () => void;
  onKeyDown: (event: ReactKeyboardEvent<HTMLButtonElement>) => void;
}) {
  return (
    <button
      className={`setting-input setting-input-shortcut hotkey-capture ${
        active ? "hotkey-capture-active" : ""
      }`}
      type="button"
      aria-label={`录制${label}快捷键`}
      aria-pressed={active}
      onClick={onStart}
      onBlur={() => {
        if (active) {
          onCancel();
        }
      }}
      onKeyDown={onKeyDown}
    >
      <span>{active ? "按下快捷键组合" : value || placeholder}</span>
    </button>
  );
}

function ToggleRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="setting-row">
      <span>{label}</span>
      <button className="ghost-control" type="button" disabled>
        {value}
      </button>
    </div>
  );
}

function TextRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="setting-row">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

export { SettingsWindow };
export { defaultAppSettings, normalizeAppSettings };
export type { ApiKeyStatus, AppSettings, SettingsStatus };
export default App;
