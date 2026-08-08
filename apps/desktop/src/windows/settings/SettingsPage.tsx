import {
  useEffect,
  useState,
  type CSSProperties,
  type KeyboardEvent as ReactKeyboardEvent,
  type ReactNode,
} from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Button } from "../../components/ui/button";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "../../components/ui/card";
import { Input } from "../../components/ui/input";
import { Label } from "../../components/ui/label";
import {
  defaultAppSettings,
  normalizeAppSettings,
  type AppSettings,
} from "../../lib/settingsTypes";
import {
  clearApiKey as invokeClearApiKey,
  getApiKeyStatus,
  getAppSettings,
  saveApiKey as invokeSaveApiKey,
  saveSettings,
  setHotkeyCaptureMode,
} from "../../services/invoke";
import { useSettingsStore } from "../../stores/settingsStore";

type SectionId =
  | "general"
  | "hotkeys"
  | "ai-selection"
  | "ai-screenshot"
  | "ai-input"
  | "appearance"
  | "privacy";

type HotkeyKey =
  | "selectionHotkey"
  | "screenshotHotkey"
  | "inputTranslateHotkey";

type AiScope = "text" | "vision" | "input";

type ApiKeyState = {
  configured: boolean;
  input: string;
  status: string;
};

const navItems: Array<{ id: SectionId; label: string }> = [
  { id: "general", label: "通用" },
  { id: "hotkeys", label: "快捷键" },
  { id: "ai-selection", label: "划词模型" },
  { id: "ai-screenshot", label: "截图模型" },
  { id: "ai-input", label: "输入模型" },
  { id: "appearance", label: "外观" },
  { id: "privacy", label: "隐私" },
];

const providerOptions = [
  {
    id: "openai_compatible",
    label: "通用 OpenAI 兼容",
    baseUrl: "",
    model: "",
  },
  {
    id: "deepseek",
    label: "DeepSeek",
    baseUrl: "https://api.deepseek.com",
    model: "deepseek-v4-flash",
  },
  {
    id: "xiaomi_mimo",
    label: "小米 MiMo",
    baseUrl: "https://api.xiaomimimo.com/v1",
    model: "mimo-v2.5",
  },
  {
    id: "kimi",
    label: "Kimi",
    baseUrl: "https://api.moonshot.cn/v1",
    model: "kimi-k2.6",
  },
  {
    id: "glm",
    label: "GLM",
    baseUrl: "https://open.bigmodel.cn/api/paas/v4",
    model: "glm-5.2",
  },
  {
    id: "minimax",
    label: "MiniMax",
    baseUrl: "https://api.minimaxi.com/v1",
    model: "MiniMax-M2.7",
  },
  {
    id: "qwen",
    label: "Qwen",
    baseUrl: "https://dashscope.aliyuncs.com/compatible-mode/v1",
    model: "qwen-plus",
  },
] as const;

function providerOptionsFor(scope: AiScope) {
  return scope === "vision"
    ? providerOptions.filter((option) => option.id !== "deepseek")
    : providerOptions;
}

const languageOptions = [
  { id: "auto", label: "自动检测" },
  { id: "zh-Hans", label: "简体中文" },
  { id: "zh-Hant", label: "繁体中文" },
  { id: "en", label: "英文" },
  { id: "ja", label: "日文" },
  { id: "ko", label: "韩文" },
  { id: "fr", label: "法文" },
  { id: "de", label: "德文" },
  { id: "es", label: "西班牙文" },
] as const;

const themeOptions = [
  { id: "system", label: "跟随系统" },
  { id: "light", label: "浅色" },
  { id: "dark", label: "深色" },
] as const;

const panelStyle: CSSProperties = {
  borderColor: "var(--qp-border)",
  background:
    "linear-gradient(180deg, var(--qp-panel-top-glow), rgba(255, 255, 255, 0.02)), var(--qp-panel-bg)",
  boxShadow: "var(--qp-shadow-md)",
  backdropFilter: "var(--qp-panel-blur)",
};

const fieldStyle: CSSProperties = {
  borderColor: "var(--qp-border)",
  background: "var(--qp-input-bg)",
  color: "var(--qp-text-primary)",
};

function keyNameFromCode(code: string): string | null {
  if (/^Key[A-Z]$/.test(code)) {
    return code.slice(3);
  }
  if (/^Digit[0-9]$/.test(code)) {
    return code.slice(5);
  }
  if (/^F([1-9]|1[0-9]|2[0-4])$/.test(code)) {
    return code;
  }
  const map: Record<string, string> = {
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
  return map[code] || null;
}

function shortcutFromEvent(event: ReactKeyboardEvent<HTMLElement>): string | null {
  const keyName = keyNameFromCode(event.code);
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

function providerDefaultsFor(
  scope: AiScope,
  current: AppSettings,
  providerId: string,
): AppSettings {
  const provider =
    providerOptionsFor(scope).find((option) => option.id === providerId) ||
    providerOptionsFor(scope)[0];
  if (scope === "text") {
    return {
      ...current,
      textAiProvider: provider.id,
      textAiBaseUrl: provider.baseUrl || current.textAiBaseUrl,
      textAiModel: provider.model || current.textAiModel,
    };
  }
  if (scope === "vision") {
    return {
      ...current,
      visionAiProvider: provider.id,
      visionAiBaseUrl: provider.baseUrl || current.visionAiBaseUrl,
      visionAiModel: provider.model || current.visionAiModel,
    };
  }
  return {
    ...current,
    inputAiProvider: provider.id,
    inputAiBaseUrl: provider.baseUrl || current.inputAiBaseUrl,
    inputAiModel: provider.model || current.inputAiModel,
  };
}

function FieldRow({
  label,
  children,
}: {
  label: string;
  children: ReactNode;
}) {
  return (
    <div className="grid gap-1.5">
      <Label>{label}</Label>
      {children}
    </div>
  );
}

function SelectField<T extends string>({
  value,
  options,
  onChange,
  ariaLabel,
}: {
  value: T;
  options: ReadonlyArray<{ id: T; label: string }>;
  onChange: (value: T) => void;
  ariaLabel: string;
}) {
  return (
    <select
      value={value}
      onChange={(event) => onChange(event.currentTarget.value as T)}
      aria-label={ariaLabel}
      className="h-9 w-full rounded-md border px-3 text-sm outline-none"
      style={fieldStyle}
    >
      {options.map((option) => (
        <option key={option.id} value={option.id}>
          {option.label}
        </option>
      ))}
    </select>
  );
}

function AiCard({
  title,
  scope,
  provider,
  baseUrl,
  model,
  apiKey,
  onProviderChange,
  onBaseUrlChange,
  onModelChange,
  onApiKeyInputChange,
  onSaveApiKey,
  onClearApiKey,
}: {
  title: string;
  scope: AiScope;
  provider: string;
  baseUrl: string;
  model: string;
  apiKey: ApiKeyState;
  onProviderChange: (providerId: string) => void;
  onBaseUrlChange: (value: string) => void;
  onModelChange: (value: string) => void;
  onApiKeyInputChange: (value: string) => void;
  onSaveApiKey: () => void;
  onClearApiKey: () => void;
}) {
  return (
    <Card style={panelStyle}>
      <CardHeader>
        <CardTitle>{title}</CardTitle>
      </CardHeader>
      <CardContent className="grid gap-3">
        <FieldRow label="供应商">
          <SelectField
            value={provider}
            options={providerOptionsFor(scope).map((option) => ({
              id: option.id,
              label: option.label,
            }))}
            onChange={onProviderChange}
            ariaLabel={`${title}供应商`}
          />
        </FieldRow>
        <FieldRow label="Base URL">
          <Input
            value={baseUrl}
            onChange={(event) => onBaseUrlChange(event.currentTarget.value)}
            style={fieldStyle}
            placeholder="https://api.example.com/v1"
          />
        </FieldRow>
        <FieldRow label="模型">
          <Input
            value={model}
            onChange={(event) => onModelChange(event.currentTarget.value)}
            style={fieldStyle}
            placeholder="model-name"
          />
        </FieldRow>
        <FieldRow label="API Key">
          <div className="grid gap-2 sm:grid-cols-[minmax(0,1fr)_auto_auto]">
            <Input
              type="password"
              value={apiKey.input}
              onChange={(event) => onApiKeyInputChange(event.currentTarget.value)}
              style={fieldStyle}
              placeholder={apiKey.configured ? "已加密保存" : "输入新的 API Key"}
            />
            <Button type="button" onClick={onSaveApiKey} size="sm">
              保存
            </Button>
            <Button type="button" variant="secondary" onClick={onClearApiKey} size="sm">
              清除
            </Button>
          </div>
          <p className="text-xs" style={{ color: "var(--qp-text-muted)" }}>
            {apiKey.status}
          </p>
        </FieldRow>
      </CardContent>
    </Card>
  );
}

export function SettingsPage({ coreStatus }: { coreStatus: string }) {
  const settings = useSettingsStore((state) => state.settings);
  const settingsStatus = useSettingsStore((state) => state.status);
  const setSettings = useSettingsStore((state) => state.setSettings);
  const updateSetting = useSettingsStore((state) => state.updateSetting);
  const setStatus = useSettingsStore((state) => state.setStatus);
  const [activeSection, setActiveSection] = useState<SectionId>("general");
  const [capturing, setCapturing] = useState<HotkeyKey | null>(null);
  const [apiKeys, setApiKeys] = useState<Record<AiScope, ApiKeyState>>({
    text: { configured: false, input: "", status: "正在检查文本模型 Key" },
    vision: { configured: false, input: "", status: "正在检查截图模型 Key" },
    input: { configured: false, input: "", status: "正在检查输入模型 Key" },
  });

  useEffect(() => {
    let mounted = true;

    getAppSettings()
      .then((value) => {
        if (mounted) {
          setSettings(normalizeAppSettings(value));
        }
      })
      .catch(() => {
        if (mounted) {
          setSettings(defaultAppSettings);
        }
      });

    void Promise.all(
      (["text", "vision", "input"] as const).map((scope) =>
        getApiKeyStatus(scope),
      ),
    )
      .then(([text, vision, input]) => {
        if (!mounted) {
          return;
        }
        setApiKeys({
          text: { ...apiKeys.text, configured: text.configured, status: text.configured ? "文本模型 Key 已加密保存" : "文本模型 Key 未配置" },
          vision: { ...apiKeys.vision, configured: vision.configured, status: vision.configured ? "截图模型 Key 已加密保存" : "截图模型 Key 未配置" },
          input: { ...apiKeys.input, configured: input.configured, status: input.configured ? "输入模型 Key 已加密保存" : "输入模型 Key 未配置" },
        });
      })
      .catch(() => {
        if (mounted) {
          setApiKeys((current) => ({
            ...current,
            text: { ...current.text, status: "检查 API Key 状态失败" },
            vision: { ...current.vision, status: "检查 API Key 状态失败" },
            input: { ...current.input, status: "检查 API Key 状态失败" },
          }));
        }
      });

    return () => {
      mounted = false;
      void setHotkeyCaptureMode(false);
    };
  }, [setSettings]);

  const selectProvider = (scope: AiScope, providerId: string) => {
    setSettings((current) => providerDefaultsFor(scope, current, providerId));
  };

  const saveAll = async () => {
    if (settingsStatus.kind === "saving") {
      return;
    }
    const next = normalizeAppSettings(settings);
    setStatus({ kind: "saving", message: "正在保存设置" });
    try {
      await saveSettings(next);
      setSettings(next);
      setStatus({ kind: "success", message: "设置已保存" });
    } catch (error) {
      setStatus({
        kind: "error",
        message: typeof error === "string" ? error : "保存设置失败",
      });
    }
  };

  const saveApiKey = async (scope: AiScope) => {
    const value = apiKeys[scope].input.trim();
    if (!value) {
      setApiKeys((current) => ({
        ...current,
        [scope]: { ...current[scope], status: "API Key 不能为空" },
      }));
      return;
    }

    const next = normalizeAppSettings(settings);
    setStatus({ kind: "saving", message: "正在同步 AI 设置" });
    setApiKeys((current) => ({
      ...current,
      [scope]: { ...current[scope], status: "正在加密保存" },
    }));

    try {
      await saveSettings(next);
      await invokeSaveApiKey(scope, value);
      setSettings(next);
      setApiKeys((current) => ({
        ...current,
        [scope]: {
          configured: true,
          input: "",
          status: "API Key 已加密保存",
        },
      }));
      setStatus({ kind: "success", message: "设置与 API Key 已保存" });
    } catch (error) {
      const message = typeof error === "string" ? error : "保存 API Key 失败";
      setApiKeys((current) => ({
        ...current,
        [scope]: { ...current[scope], status: message },
      }));
      setStatus({ kind: "error", message });
    }
  };

  const clearApiKey = async (scope: AiScope) => {
    try {
      await invokeClearApiKey(scope);
      setApiKeys((current) => ({
        ...current,
        [scope]: {
          configured: false,
          input: "",
          status: "API Key 已清除",
        },
      }));
    } catch (error) {
      const message = typeof error === "string" ? error : "清除 API Key 失败";
      setApiKeys((current) => ({
        ...current,
        [scope]: { ...current[scope], status: message },
      }));
    }
  };

  const startHotkeyCapture = async (key: HotkeyKey) => {
    try {
      await setHotkeyCaptureMode(true);
      setCapturing(key);
    } catch {
      setStatus({ kind: "error", message: "无法进入快捷键录制" });
    }
  };

  const finishHotkeyCapture = async () => {
    setCapturing(null);
    try {
      await setHotkeyCaptureMode(false);
    } catch {
      // Non-blocking cleanup.
    }
  };

  const handleHotkeyKeyDown = async (
    event: ReactKeyboardEvent<HTMLButtonElement>,
  ) => {
    event.preventDefault();
    event.stopPropagation();
    if (!capturing) {
      return;
    }
    if (event.key === "Escape") {
      await finishHotkeyCapture();
      return;
    }
    if (!event.altKey) {
      setStatus({ kind: "error", message: "快捷键必须包含 Alt" });
      return;
    }
    const shortcut = shortcutFromEvent(event);
    if (!shortcut) {
      return;
    }
    updateSetting(capturing, shortcut);
    setStatus({ kind: "idle", message: "快捷键已记录，保存后生效" });
    await finishHotkeyCapture();
  };

  const renderSection = () => {
    switch (activeSection) {
      case "general":
        return (
          <Card style={panelStyle}>
            <CardHeader>
              <CardTitle>通用</CardTitle>
            </CardHeader>
            <CardContent className="grid gap-3">
              <FieldRow label="开机自启">
                <label className="flex items-center gap-2 text-sm">
                  <input
                    type="checkbox"
                    checked={settings.autostartEnabled}
                    onChange={(event) =>
                      updateSetting("autostartEnabled", event.currentTarget.checked)
                    }
                    className="h-4 w-4 accent-[color:var(--qp-accent)]"
                  />
                  启动 QuickPick 时自动运行
                </label>
              </FieldRow>
              <FieldRow label="AI 请求超时（秒）">
                <Input
                  type="number"
                  min={5}
                  max={120}
                  value={settings.aiTimeoutSeconds}
                  onChange={(event) =>
                    updateSetting(
                      "aiTimeoutSeconds",
                      Number(event.currentTarget.value) || 30,
                    )
                  }
                  style={fieldStyle}
                />
              </FieldRow>
            </CardContent>
          </Card>
        );
      case "hotkeys":
        return (
          <Card style={panelStyle}>
            <CardHeader>
              <CardTitle>快捷键</CardTitle>
            </CardHeader>
            <CardContent className="grid gap-3">
              <HotkeyRow
                label="划词菜单"
                value={settings.selectionHotkey}
                active={capturing === "selectionHotkey"}
                onStart={() => startHotkeyCapture("selectionHotkey")}
                onKeyDown={handleHotkeyKeyDown}
              />
              <HotkeyRow
                label="区域截图"
                value={settings.screenshotHotkey}
                active={capturing === "screenshotHotkey"}
                onStart={() => startHotkeyCapture("screenshotHotkey")}
                onKeyDown={handleHotkeyKeyDown}
              />
              <HotkeyRow
                label="输入翻译"
                value={settings.inputTranslateHotkey}
                active={capturing === "inputTranslateHotkey"}
                onStart={() => startHotkeyCapture("inputTranslateHotkey")}
                onKeyDown={handleHotkeyKeyDown}
              />
            </CardContent>
          </Card>
        );
      case "ai-selection":
        return (
          <AiCard
            title="划词模型"
            scope="text"
            provider={settings.textAiProvider}
            baseUrl={settings.textAiBaseUrl}
            model={settings.textAiModel}
            apiKey={apiKeys.text}
            onProviderChange={(providerId) => selectProvider("text", providerId)}
            onBaseUrlChange={(value) => updateSetting("textAiBaseUrl", value)}
            onModelChange={(value) => updateSetting("textAiModel", value)}
            onApiKeyInputChange={(value) =>
              setApiKeys((current) => ({
                ...current,
                text: { ...current.text, input: value },
              }))
            }
            onSaveApiKey={() => saveApiKey("text")}
            onClearApiKey={() => clearApiKey("text")}
          />
        );
      case "ai-screenshot":
        return (
          <AiCard
            title="截图模型"
            scope="vision"
            provider={settings.visionAiProvider}
            baseUrl={settings.visionAiBaseUrl}
            model={settings.visionAiModel}
            apiKey={apiKeys.vision}
            onProviderChange={(providerId) => selectProvider("vision", providerId)}
            onBaseUrlChange={(value) => updateSetting("visionAiBaseUrl", value)}
            onModelChange={(value) => updateSetting("visionAiModel", value)}
            onApiKeyInputChange={(value) =>
              setApiKeys((current) => ({
                ...current,
                vision: { ...current.vision, input: value },
              }))
            }
            onSaveApiKey={() => saveApiKey("vision")}
            onClearApiKey={() => clearApiKey("vision")}
          />
        );
      case "ai-input":
        return (
          <>
            <AiCard
              title="输入模型"
              scope="input"
              provider={settings.inputAiProvider}
              baseUrl={settings.inputAiBaseUrl}
              model={settings.inputAiModel}
              apiKey={apiKeys.input}
              onProviderChange={(providerId) => selectProvider("input", providerId)}
              onBaseUrlChange={(value) => updateSetting("inputAiBaseUrl", value)}
              onModelChange={(value) => updateSetting("inputAiModel", value)}
              onApiKeyInputChange={(value) =>
                setApiKeys((current) => ({
                  ...current,
                  input: { ...current.input, input: value },
                }))
              }
              onSaveApiKey={() => saveApiKey("input")}
              onClearApiKey={() => clearApiKey("input")}
            />
            <Card style={panelStyle}>
              <CardHeader>
                <CardTitle>输入翻译行为</CardTitle>
              </CardHeader>
              <CardContent className="grid gap-3 sm:grid-cols-2">
                <FieldRow label="默认源语言">
                  <SelectField
                    value={settings.inputTranslateSourceLanguage}
                    options={languageOptions}
                    onChange={(value) =>
                      updateSetting("inputTranslateSourceLanguage", value)
                    }
                    ariaLabel="默认源语言"
                  />
                </FieldRow>
                <FieldRow label="默认目标语言">
                  <SelectField
                    value={settings.inputTranslateTargetLanguage}
                    options={languageOptions.filter((option) => option.id !== "auto")}
                    onChange={(value) =>
                      updateSetting("inputTranslateTargetLanguage", value)
                    }
                    ariaLabel="默认目标语言"
                  />
                </FieldRow>
              </CardContent>
            </Card>
          </>
        );
      case "appearance":
        return (
          <Card style={panelStyle}>
            <CardHeader>
              <CardTitle>外观</CardTitle>
            </CardHeader>
            <CardContent className="grid gap-3">
              <FieldRow label="主题">
                <SelectField
                  value={settings.themeMode}
                  options={themeOptions}
                  onChange={(value) => updateSetting("themeMode", value)}
                  ariaLabel="主题"
                />
              </FieldRow>
              <p className="text-sm" style={{ color: "var(--qp-text-secondary)" }}>
                QuickPick 核心已连接：{coreStatus}
              </p>
            </CardContent>
          </Card>
        );
      case "privacy":
        return (
          <Card style={panelStyle}>
            <CardHeader>
              <CardTitle>隐私</CardTitle>
            </CardHeader>
            <CardContent className="grid gap-3">
              <p className="text-sm leading-6" style={{ color: "var(--qp-text-secondary)" }}>
                API Key 使用系统 DPAPI 加密保存，不会写入前端本地存储或开发日志。翻译文本只在请求上下文中使用，关闭窗口后不保留历史。
              </p>
              <div className="grid gap-2 text-sm">
                <span>文本模型 Key：{apiKeys.text.configured ? "已配置" : "未配置"}</span>
                <span>截图模型 Key：{apiKeys.vision.configured ? "已配置" : "未配置"}</span>
                <span>输入模型 Key：{apiKeys.input.configured ? "已配置" : "未配置"}</span>
              </div>
            </CardContent>
          </Card>
        );
    }
  };

  const minimize = () => {
    void getCurrentWindow().minimize();
  };
  const toggleMaximize = () => {
    void getCurrentWindow().toggleMaximize();
  };
  const hide = () => {
    void getCurrentWindow().hide();
  };

  return (
    <div
      className="flex min-h-screen flex-col"
      style={{
        background: "var(--qp-shell-gradient), var(--qp-shell-bg)",
        color: "var(--qp-text-primary)",
        backdropFilter: "var(--qp-shell-blur)",
      }}
    >
      <header
        className="flex h-10 shrink-0 items-center justify-between border-b px-3"
        data-tauri-drag-region
        style={{ borderColor: "var(--qp-border)" }}
      >
        <div className="text-sm font-semibold" data-tauri-drag-region>
          QuickPick 设置
        </div>
        <div className="flex items-center gap-1">
          <button
            type="button"
            aria-label="最小化"
            onClick={minimize}
            className="h-8 w-9 rounded-md text-sm"
            style={{ color: "var(--qp-text-secondary)" }}
          >
            ─
          </button>
          <button
            type="button"
            aria-label="最大化或还原"
            onClick={toggleMaximize}
            className="h-8 w-9 rounded-md text-sm"
            style={{ color: "var(--qp-text-secondary)" }}
          >
            ▢
          </button>
          <button
            type="button"
            aria-label="关闭设置"
            onClick={hide}
            className="h-8 w-9 rounded-md text-sm"
            style={{ color: "var(--qp-text-secondary)" }}
          >
            ×
          </button>
        </div>
      </header>

      <div className="mx-auto flex w-full max-w-[720px] flex-1 flex-col gap-2 overflow-hidden p-2">
        <nav
          className="flex shrink-0 items-center justify-center gap-1 overflow-x-auto rounded-lg border p-1"
          style={panelStyle}
          aria-label="设置导航"
        >
          {navItems.map((item) => (
            <button
              key={item.id}
              type="button"
              onClick={() => setActiveSection(item.id)}
              className="h-8 shrink-0 rounded-md px-3 text-sm font-medium"
              style={
                activeSection === item.id
                  ? {
                      borderColor: `rgba(var(--qp-accent-rgb), 0.38)`,
                      color: "var(--qp-text-primary)",
                      background: `rgba(var(--qp-accent-rgb), 0.16)`,
                    }
                  : {
                      color: "var(--qp-text-secondary)",
                    }
              }
            >
              {item.label}
            </button>
          ))}
        </nav>

        <main className="min-h-0 flex-1 space-y-2 overflow-y-auto pb-2 pr-1">
          {renderSection()}
        </main>

        <footer
          className="flex shrink-0 flex-col items-stretch gap-2 rounded-lg border p-3"
          style={panelStyle}
        >
          <p
            className="min-h-5 text-sm"
            style={{
              color:
                settingsStatus.kind === "error"
                  ? "var(--qp-danger)"
                  : settingsStatus.kind === "success"
                    ? "var(--qp-success)"
                    : "var(--qp-text-secondary)",
            }}
          >
            {settingsStatus.message}
          </p>
          <Button
            type="button"
            onClick={() => void saveAll()}
            disabled={settingsStatus.kind === "saving"}
            className="w-full"
          >
            保存设置
          </Button>
        </footer>
      </div>
    </div>
  );
}

function HotkeyRow({
  label,
  value,
  active,
  onStart,
  onKeyDown,
}: {
  label: string;
  value: string;
  active: boolean;
  onStart: () => void;
  onKeyDown: (event: ReactKeyboardEvent<HTMLButtonElement>) => void;
}) {
  return (
    <div className="grid gap-1.5">
      <Label>{label}</Label>
      <Button
        type="button"
        variant={active ? "default" : "secondary"}
        onKeyDown={onKeyDown}
        onClick={onStart}
        className="justify-start"
      >
        {active ? "按下新的快捷键..." : value}
      </Button>
    </div>
  );
}
