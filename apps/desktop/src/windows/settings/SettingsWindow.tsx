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
import { RotateCcw } from "lucide-react";
import { LoadingSkeleton } from "../../components/LoadingSkeleton";
import {
  setI18nLanguage,
  targetLanguageForUiLanguage,
  translationLanguageOptions,
  uiLanguageOptions,
  useI18n,
  type UiLanguage,
} from "../../lib/i18n";
import {
  applyInputAiProviderDefaults,
  applyTextAiProviderDefaults,
  applyVisionAiProviderDefaults,
  defaultAppSettings,
  inputAiProviderOptions,
  normalizeAppSettings,
  textAiProviderOptions,
  themeModeOptions,
  visionAiProviderOptions,
  windowEffectOptions,
  type ApiKeyStatus,
  type AppSettings,
  type SettingsStatus,
} from "../../lib/settingsTypes";
import {
  findHotkeyConflict,
  type HotkeySettingKey,
} from "../../lib/hotkeys";
import { useSettingsStore } from "../../stores/settingsStore";
import {
  runningOnMac,
  setPlatformDataset,
} from "../../lib/platform";

type SelectionActionResult = {
  message: string;
  hotkeyStatuses?: Array<{
    name: string;
    status: "registered" | "occupied" | "failed";
  }>;
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
    Comma: "Comma",
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
  if (runningOnMac && event.metaKey) {
    parts.push("Command");
  }
  if (event.ctrlKey) {
    parts.push("Ctrl");
  }
  if (event.shiftKey) {
    parts.push("Shift");
  }
  if (event.altKey) {
    parts.push(runningOnMac ? "Option" : "Alt");
  }
  parts.push(keyName);

  return parts.join("+");
}

type GlassSelectOption<T extends string = string> = {
  id: T;
  label: string;
};

const settingsNavItems = [
  { id: "general", label: "settings.general" },
  { id: "hotkeys", label: "settings.hotkeys" },
  { id: "ai-selection", label: "settings.aiSelection" },
  { id: "ai-screenshot", label: "settings.aiScreenshot" },
  { id: "ai-input", label: "settings.aiInput" },
  { id: "appearance", label: "settings.appearance" },
  { id: "privacy", label: "settings.privacy" },
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
  setI18nLanguage(settings.uiLanguage);
  const themeMode =
    settings.themeMode === "system" ? currentSystemTheme() : settings.themeMode;
  const effectiveTheme = themeMode === "dark" ? "workbench" : themeMode;

  root.dataset.theme = effectiveTheme;
  root.dataset.themePreference = settings.themeMode;
  root.style.colorScheme = effectiveTheme === "light" ? "light" : "dark";
  root.dataset.windowEffect = settings.windowEffect;
  setPlatformDataset(root);
  if (settings.windowEffect === "mica") {
    for (const name of appearanceAlphaVariables) {
      root.style.removeProperty(name);
    }
    return;
  }

  const panelOpacity =
    Math.min(100, Math.max(30, Number(settings.panelOpacity) || 100)) / 100;
  for (const name of appearanceAlphaVariables) {
    root.style.setProperty(name, String(panelOpacity));
  }
}

function isHttpBaseUrl(value: string) {
  return value.trim().startsWith("http://");
}

function SettingsTitlebar() {
  const t = useI18n();
  useEffect(() => {
    if (!runningOnMac) {
      getCurrentWindow().setDecorations(false).catch(() => {});
    }
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
        <span data-tauri-drag-region>{t("settings.title")}</span>
      </div>
      <div className="settings-titlebar-controls">
        <button
          className="settings-titlebar-button"
          type="button"
          aria-label={t("settings.minimize")}
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
          aria-label={t("settings.maximize")}
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
          aria-label={t("settings.close")}
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

function SettingsWindow() {
  const t = useI18n();
  const localizedUiLanguageOptions = uiLanguageOptions.map((option) => ({
    id: option.id,
    label: t(option.label),
  }));
  const localizedTextProviderOptions = textAiProviderOptions.map((option) => ({
    id: option.id,
    label: t(option.label),
  }));
  const localizedVisionProviderOptions = visionAiProviderOptions.map(
    (option) => ({
      id: option.id,
      label: t(option.label),
    }),
  );
  const localizedInputProviderOptions = inputAiProviderOptions.map(
    (option) => ({
      id: option.id,
      label: t(option.label),
    }),
  );
  const localizedThemeModeOptions = themeModeOptions.map((option) => ({
    id: option.id,
    label: t(option.label),
  }));
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
  const [settingsReady, setSettingsReady] = useState(false);
  const [hotkeyStatuses, setHotkeyStatuses] = useState<
    NonNullable<SelectionActionResult["hotkeyStatuses"]>
  >([]);
  const hotkeyStatusByName = (name: string) =>
    hotkeyStatuses.find((item) => item.name === name)?.status;
  const [textApiKeyInput, setTextApiKeyInput] = useState("");
  const [visionApiKeyInput, setVisionApiKeyInput] = useState("");
  const [textApiKeyConfigured, setTextApiKeyConfigured] = useState(false);
  const [visionApiKeyConfigured, setVisionApiKeyConfigured] = useState(false);
  const [textApiKeyStatus, setTextApiKeyStatus] = useState<SettingsStatus>({
    kind: "loading",
    message: t("settings.checkingTextKey"),
  });
  const [visionApiKeyStatus, setVisionApiKeyStatus] = useState<SettingsStatus>({
    kind: "loading",
    message: t("settings.checkingVisionKey"),
  });
  const [inputApiKeyInput, setInputApiKeyInput] = useState("");
  const [inputApiKeyConfigured, setInputApiKeyConfigured] = useState(false);
  const [inputApiKeyStatus, setInputApiKeyStatus] = useState<SettingsStatus>({
    kind: "loading",
    message: t("settings.checkingInputKey"),
  });

  const loadInputApiKeyStatus = () => {
    invoke<ApiKeyStatus>("get_api_key_status", { scope: "input" })
      .then((value) => {
        setInputApiKeyConfigured(value.configured);
        setInputApiKeyStatus({
          kind: "idle",
          message: value.configured
            ? t("settings.inputKeyEncrypted")
            : t("settings.inputKeyNotConfigured"),
        });
      })
      .catch((error) => {
        setInputApiKeyConfigured(false);
        setInputApiKeyStatus({
          kind: "error",
          message:
            typeof error === "string"
              ? error
              : t("settings.checkInputKeyFailed"),
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
          setSettingsStatus({ kind: "idle", message: t("settings.loaded") });
          setSettingsReady(true);
        }
      })
      .catch((error) => {
        if (isMounted) {
          setSettings(defaultAppSettings);
          setSettingsStatus({
            kind: "error",
            message:
              typeof error === "string"
                ? error
                : t("settings.loadFailed"),
          });
          setSettingsReady(true);
        }
      });

    invoke<ApiKeyStatus>("get_api_key_status", { scope: "text" })
      .then((value) => {
        if (isMounted) {
          setTextApiKeyConfigured(value.configured);
          setTextApiKeyStatus({
            kind: "idle",
            message: value.configured
              ? t("settings.textKeyEncrypted")
              : t("settings.textKeyNotConfigured"),
          });
        }
      })
      .catch((error) => {
        if (isMounted) {
          setTextApiKeyConfigured(false);
          setTextApiKeyStatus({
            kind: "error",
            message:
              typeof error === "string"
                ? error
                : t("settings.checkTextKeyFailed"),
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
              ? t("settings.visionKeyEncrypted")
              : t("settings.visionKeyNotConfigured"),
          });
        }
      })
      .catch((error) => {
        if (isMounted) {
          setVisionApiKeyConfigured(false);
          setVisionApiKeyStatus({
            kind: "error",
            message:
              typeof error === "string"
                ? error
                : t("settings.checkVisionKeyFailed"),
          });
        }
      });

    return () => {
      isMounted = false;
    };
  }, []);

  const markSettingsDirty = (message = t("settings.unsaved")) => {
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
      message: t("settings.hotkeyPrompt"),
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
      finishHotkeyCapture(t("settings.hotkeyCancelled"));
      return;
    }
    if (modifierKeyNames.has(event.key)) {
      return;
    }
    const hasRequiredModifier = runningOnMac
      ? event.metaKey || event.altKey
      : event.altKey;
    if (!hasRequiredModifier) {
      setSettingsStatus({
        kind: "error",
        message: t("settings.hotkeyNeedsAlt"),
      });
      return;
    }

    const shortcut = shortcutFromKeyboardEvent(event);
    if (!shortcut) {
      setSettingsStatus({
        kind: "error",
        message: t("settings.hotkeyUnsupported"),
      });
      return;
    }

    const conflictKey = findHotkeyConflict(shortcut, settings, key);
    if (conflictKey) {
      const conflictLabelKey =
        conflictKey === "settingsHotkey"
          ? "settings.hotkeySettings"
          : conflictKey === "selectionHotkey"
            ? "settings.hotkeySelection"
            : conflictKey === "screenshotHotkey"
              ? "settings.hotkeyScreenshot"
              : "settings.hotkeyInput";
      setSettingsStatus({
        kind: "error",
        message: t("settings.hotkeyConflict", {
          conflict: t(conflictLabelKey),
        }),
      });
      return;
    }

    setSettings((current) => ({ ...current, [key]: shortcut }));
    finishHotkeyCapture(t("settings.hotkeyRecorded"));
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

    setSettingsStatus({ kind: "saving", message: t("settings.saving") });
    invoke<SelectionActionResult>("save_app_settings", {
      settings: nextSettings,
    })
      .then((result) => {
        setSettings(nextSettings);
        setHotkeyStatuses(result.hotkeyStatuses ?? []);
        setSettingsStatus({ kind: "success", message: result.message });
      })
      .catch((error) => {
        setSettingsStatus({
          kind: "error",
          message: typeof error === "string" ? error : t("settings.saveFailed"),
        });
      });
  };

  const resetDefaultSettings = () => {
    if (settingsStatus.kind === "saving") {
      return;
    }

    setSettingsStatus({ kind: "saving", message: t("settings.resetting") });
    const nextSettings = normalizeAppSettings(defaultAppSettings);
    invoke<SelectionActionResult>("save_app_settings", {
      settings: nextSettings,
    })
      .then((result) => {
        setSettings(nextSettings);
        setHotkeyStatuses(result.hotkeyStatuses ?? []);
        setSettingsStatus({ kind: "success", message: result.message });
      })
      .catch((error) => {
        setSettingsStatus({
          kind: "error",
          message:
            typeof error === "string" ? error : t("settings.resetFailed"),
        });
      });
  };

  const selectTextProvider = (providerId: string) => {
    setSettings((current) => applyTextAiProviderDefaults(current, providerId));
    markSettingsDirty(t("settings.textProviderFilled"));
  };

  const selectVisionProvider = (providerId: string) => {
    setSettings((current) => applyVisionAiProviderDefaults(current, providerId));
    markSettingsDirty(t("settings.visionProviderFilled"));
  };

  const selectInputProvider = (providerId: string) => {
    setSettings((current) => applyInputAiProviderDefaults(current, providerId));
    markSettingsDirty(t("settings.inputProviderFilled"));
  };

  const updateTextApiKeyInput = (value: string) => {
    setTextApiKeyInput(value);
    if (textApiKeyStatus.kind === "success" || textApiKeyStatus.kind === "error") {
      setTextApiKeyStatus({
        kind: "idle",
        message: textApiKeyConfigured
          ? t("settings.replaceTextKeyHint")
          : t("settings.textKeyNotConfigured"),
      });
    }
  };

  const updateVisionApiKeyInput = (value: string) => {
    setVisionApiKeyInput(value);
    if (visionApiKeyStatus.kind === "success" || visionApiKeyStatus.kind === "error") {
      setVisionApiKeyStatus({
        kind: "idle",
        message: visionApiKeyConfigured
          ? t("settings.replaceVisionKeyHint")
          : t("settings.visionKeyNotConfigured"),
      });
    }
  };

  const updateInputApiKeyInput = (value: string) => {
    setInputApiKeyInput(value);
    if (inputApiKeyStatus.kind === "success" || inputApiKeyStatus.kind === "error") {
      setInputApiKeyStatus({
        kind: "idle",
        message: inputApiKeyConfigured
          ? t("settings.replaceInputKeyHint")
          : t("settings.inputKeyNotConfigured"),
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
    const labelKey = isText
      ? "settings.textKeyLabel"
      : isVision
        ? "settings.visionKeyLabel"
        : "settings.inputKeyLabel";
    const label = t(labelKey);

    if (status.kind === "saving") {
      return;
    }

    if (!input.trim()) {
      setStatus({
        kind: "error",
        message: `${label} ${t("settings.cannotBeEmpty")}`,
      });
      return;
    }

    const nextSettings = normalizeAppSettings(settings);
    setSettings(nextSettings);
    setSettingsStatus({ kind: "saving", message: t("settings.syncingAi") });
    setStatus({ kind: "saving", message: t("settings.encrypting") });
    invoke<SelectionActionResult>("save_app_settings", {
      settings: nextSettings,
    })
      .then(() => {
        setSettingsStatus({ kind: "success", message: t("settings.saved") });
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
              message:
                typeof error === "string"
                  ? error
                  : `${label} ${t("settings.saveKeyFailed")}`,
            });
          });
      })
      .catch((error) => {
        setSettingsStatus({
          kind: "error",
          message:
            typeof error === "string" ? error : t("settings.syncAiFailed"),
        });
        setStatus({
          kind: "error",
          message: `${t("settings.syncBeforeSaveFailed")}: ${label}`,
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
    const labelKey = isText
      ? "settings.textKeyLabel"
      : isVision
        ? "settings.visionKeyLabel"
        : "settings.inputKeyLabel";
    const label = t(labelKey);

    if (status.kind === "saving") {
      return;
    }

    setStatus({ kind: "saving", message: t("settings.clearing") });
    invoke<SelectionActionResult>("clear_api_key", { scope })
      .then((result) => {
        setInput("");
        setConfigured(false);
        setStatus({ kind: "success", message: result.message });
      })
      .catch((error) => {
        setStatus({
          kind: "error",
          message:
            typeof error === "string"
              ? error
              : `${label} ${t("settings.clearFailed")}`,
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

      {settingsReady ? (
        <div className="settings-shell">
          <section className="settings-layout">
          <nav className="settings-nav" aria-label={t("settings.categories")} ref={navRef}>
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
              {t(item.label)}
            </button>
          ))}
          </nav>

          <div
            className="settings-sections"
            ref={sectionsRef}
            onScroll={updateActiveSettingsSection}
          >
          <SettingsSection id="general" title={t("settings.general")}>
            <SettingField label={t("settings.autostart")}>
              <label className="toggle-control">
                <input
                  type="checkbox"
                  checked={settings.autostartEnabled}
                  onChange={(event) =>
                    updateSetting("autostartEnabled", event.currentTarget.checked)
                  }
                />
                <span>
                  {settings.autostartEnabled
                    ? t("settings.enabled")
                    : t("settings.disabled")}
                </span>
              </label>
            </SettingField>
            <SettingField label={t("settings.language")}>
              <GlassSelect
                ariaLabel={t("settings.language")}
                value={settings.uiLanguage}
                options={localizedUiLanguageOptions}
                onChange={(value) => updateUiLanguage(value as UiLanguage)}
              />
            </SettingField>
            <SettingField label={t("settings.allowClipboardFallback")}>
              <label className="toggle-control">
                <input
                  type="checkbox"
                  checked={settings.allowClipboardFallback}
                  onChange={(event) =>
                    updateSetting(
                      "allowClipboardFallback",
                      event.currentTarget.checked,
                    )
                  }
                />
                <span>
                  {settings.allowClipboardFallback
                    ? t("settings.enabled")
                    : t("settings.disabled")}
                </span>
              </label>
              <p className="setting-hint">
                {t("settings.allowClipboardFallbackDescription")}
              </p>
            </SettingField>
            <SettingField label={t("settings.aiTimeout")}>
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
            <SettingField label={t("settings.reset")}>
              <button
                className="ghost-control popup-icon-button"
                type="button"
                onClick={resetDefaultSettings}
                disabled={settingsStatus.kind === "saving"}
                aria-label={t("settings.reset")}
                title={t("settings.reset")}
              >
                <RotateCcw size={15} />
              </button>
            </SettingField>
          </SettingsSection>
          <SettingsSection id="hotkeys" title={t("settings.hotkeys")}>
            <SettingField label={t("settings.hotkeySettings")}>
              <div className="hotkey-capture-row">
                <HotkeyCaptureButton
                  label={t("settings.hotkeySettings")}
                  value={settings.settingsHotkey}
                  placeholder="Alt+0"
                  active={capturingHotkey === "settingsHotkey"}
                  onStart={() => startHotkeyCapture("settingsHotkey")}
                  onCancel={() => finishHotkeyCapture(t("settings.hotkeyCancelled"))}
                  onKeyDown={(event) => captureHotkey("settingsHotkey", event)}
                />
                {hotkeyStatusByName("设置") && (
                  <span className={`hotkey-status-badge hotkey-status-${hotkeyStatusByName("设置")}`}>
                    {t(`hotkey.status.${hotkeyStatusByName("设置")}`)}
                  </span>
                )}
              </div>
            </SettingField>
            <SettingField label={t("settings.hotkeySelection")}>
              <div className="hotkey-capture-row">
                <HotkeyCaptureButton
                  label={t("settings.hotkeySelection")}
                  value={settings.selectionHotkey}
                  placeholder="Alt+2"
                  active={capturingHotkey === "selectionHotkey"}
                  onStart={() => startHotkeyCapture("selectionHotkey")}
                  onCancel={() => finishHotkeyCapture(t("settings.hotkeyCancelled"))}
                  onKeyDown={(event) => captureHotkey("selectionHotkey", event)}
                />
                {hotkeyStatusByName("划词菜单") && (
                  <span className={`hotkey-status-badge hotkey-status-${hotkeyStatusByName("划词菜单")}`}>
                    {t(`hotkey.status.${hotkeyStatusByName("划词菜单")}`)}
                  </span>
                )}
              </div>
            </SettingField>
            <SettingField label={t("settings.hotkeyScreenshot")}>
              <div className="hotkey-capture-row">
                <HotkeyCaptureButton
                  label={t("settings.hotkeyScreenshot")}
                  value={settings.screenshotHotkey}
                  placeholder="Alt+3"
                  active={capturingHotkey === "screenshotHotkey"}
                  onStart={() => startHotkeyCapture("screenshotHotkey")}
                  onCancel={() => finishHotkeyCapture(t("settings.hotkeyCancelled"))}
                  onKeyDown={(event) => captureHotkey("screenshotHotkey", event)}
                />
                {hotkeyStatusByName("区域截图") && (
                  <span className={`hotkey-status-badge hotkey-status-${hotkeyStatusByName("区域截图")}`}>
                    {t(`hotkey.status.${hotkeyStatusByName("区域截图")}`)}
                  </span>
                )}
              </div>
            </SettingField>
            <SettingField label={t("settings.hotkeyInput")}>
              <div className="hotkey-capture-row">
                <HotkeyCaptureButton
                  label={t("settings.hotkeyInput")}
                  value={settings.inputTranslateHotkey}
                  placeholder="Alt+4"
                  active={capturingHotkey === "inputTranslateHotkey"}
                  onStart={() => startHotkeyCapture("inputTranslateHotkey")}
                  onCancel={() => finishHotkeyCapture(t("settings.hotkeyCancelled"))}
                  onKeyDown={(event) =>
                    captureHotkey("inputTranslateHotkey", event)
                  }
                />
                {hotkeyStatusByName("输入翻译") && (
                  <span className={`hotkey-status-badge hotkey-status-${hotkeyStatusByName("输入翻译")}`}>
                    {t(`hotkey.status.${hotkeyStatusByName("输入翻译")}`)}
                  </span>
                )}
              </div>
            </SettingField>
          </SettingsSection>
          <SettingsSection id="ai-selection" title={t("settings.aiSelection")}>
            <SettingField label={t("settings.textProvider")}>
              <GlassSelect
                ariaLabel={t("settings.selectTextProvider")}
                value={settings.textAiProvider}
                options={localizedTextProviderOptions}
                onChange={selectTextProvider}
              />
            </SettingField>
            <SettingField label={t("settings.textBaseUrl")}>
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
              {isHttpBaseUrl(settings.textAiBaseUrl) && (
                <p className="setting-hint">{t("settings.baseUrlHttpWarning")}</p>
              )}
            </SettingField>
            <SettingField label={t("settings.textModel")}>
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
            <SettingField label={t("settings.textApiKey")}>
              <div className="secret-control">
                <input
                  className="setting-input"
                  type="password"
                  value={textApiKeyInput}
                  placeholder={
                    textApiKeyConfigured
                      ? t("settings.textKeyEncryptedReplace")
                      : t("settings.enterTextApiKey")
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
                  {t("settings.saveTextKey")}
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
                  {t("settings.clear")}
                </button>
              </div>
            </SettingField>
            <TextRow
              label={t("settings.textKeyStatus")}
              value={
                textApiKeyConfigured
                  ? t("settings.encrypted")
                  : t("settings.notConfigured")
              }
              loading={textApiKeyStatus.kind === "loading"}
            />
            <div className="settings-key-feedback">
              {textApiKeyStatus.kind === "loading" ? (
                <LoadingSkeleton width={140} height={14} />
              ) : (
                <span
                  className={`settings-feedback settings-feedback-${textApiKeyStatus.kind}`}
                >
                  {t(textApiKeyStatus.message)}
                </span>
              )}
            </div>
          </SettingsSection>
          <SettingsSection id="ai-screenshot" title={t("settings.aiScreenshot")}>
            <SettingField label={t("settings.visionProvider")}>
              <GlassSelect
                ariaLabel={t("settings.selectVisionProvider")}
                value={settings.visionAiProvider}
                options={localizedVisionProviderOptions}
                onChange={selectVisionProvider}
              />
            </SettingField>
            <SettingField label={t("settings.visionBaseUrl")}>
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
              {isHttpBaseUrl(settings.visionAiBaseUrl) && (
                <p className="setting-hint">{t("settings.baseUrlHttpWarning")}</p>
              )}
            </SettingField>
            <SettingField label={t("settings.visionModel")}>
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
            <SettingField label={t("settings.visionApiKey")}>
              <div className="secret-control">
                <input
                  className="setting-input"
                  type="password"
                  value={visionApiKeyInput}
                  placeholder={
                    visionApiKeyConfigured
                      ? t("settings.visionKeyEncryptedReplace")
                      : t("settings.enterVisionApiKey")
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
                  {t("settings.saveVisionKey")}
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
                  {t("settings.clear")}
                </button>
              </div>
            </SettingField>
            <TextRow
              label={t("settings.visionKeyStatus")}
              value={
                visionApiKeyConfigured
                  ? t("settings.encrypted")
                  : t("settings.notConfigured")
              }
              loading={visionApiKeyStatus.kind === "loading"}
            />
            <div className="settings-key-feedback">
              {visionApiKeyStatus.kind === "loading" ? (
                <LoadingSkeleton width={140} height={14} />
              ) : (
                <span
                  className={`settings-feedback settings-feedback-${visionApiKeyStatus.kind}`}
                >
                  {t(visionApiKeyStatus.message)}
                </span>
              )}
            </div>
          </SettingsSection>
          <SettingsSection id="ai-input" title={t("settings.aiInput")}>
            <SettingField label={t("settings.inputProvider")}>
              <GlassSelect
                ariaLabel={t("settings.selectInputProvider")}
                value={settings.inputAiProvider}
                options={localizedInputProviderOptions}
                onChange={selectInputProvider}
              />
            </SettingField>
            <SettingField label={t("settings.inputBaseUrl")}>
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
              {isHttpBaseUrl(settings.inputAiBaseUrl) && (
                <p className="setting-hint">{t("settings.baseUrlHttpWarning")}</p>
              )}
            </SettingField>
            <SettingField label={t("settings.inputModel")}>
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
            <SettingField label={t("settings.inputApiKey")}>
              <div className="secret-control">
                <input
                  className="setting-input"
                  type="password"
                  value={inputApiKeyInput}
                  placeholder={
                    inputApiKeyConfigured
                      ? t("settings.inputKeyEncryptedReplace")
                      : t("settings.enterInputApiKey")
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
                  {t("settings.saveInputKey")}
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
                  {t("settings.clear")}
                </button>
              </div>
            </SettingField>
            <TextRow
              label={t("settings.inputKeyStatus")}
              value={
                inputApiKeyConfigured
                  ? t("settings.encrypted")
                  : t("settings.notConfigured")
              }
              loading={inputApiKeyStatus.kind === "loading"}
            />
            <div className="settings-key-feedback">
              {inputApiKeyStatus.kind === "loading" ? (
                <LoadingSkeleton width={140} height={14} />
              ) : (
                <span
                  className={`settings-feedback settings-feedback-${inputApiKeyStatus.kind}`}
                >
                  {t(inputApiKeyStatus.message)}
                </span>
              )}
            </div>
          </SettingsSection>
          <SettingsSection id="appearance" title={t("settings.appearance")}>
            <SettingField label={t("settings.theme")}>
              <GlassSelect
                ariaLabel={t("settings.selectTheme")}
                value={settings.themeMode}
                options={localizedThemeModeOptions}
                onChange={(value) => updateSetting("themeMode", value)}
              />
            </SettingField>
            <SettingField label={t("settings.windowEffect")}>
              <GlassSelect
                ariaLabel={t("settings.selectWindowEffect")}
                value={settings.windowEffect}
                options={windowEffectOptions}
                onChange={(value) => updateSetting("windowEffect", value)}
              />
            </SettingField>
            <SettingField label={t("settings.opacity")}>
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
          <SettingsSection id="privacy" title={t("settings.privacy")}>
            <TextRow
              label={t("settings.contentHistory")}
              value={t("settings.contentHistoryValue")}
            />
            <TextRow
              label={t("settings.apiKey")}
              value={t("settings.apiKeyValue")}
            />
          </SettingsSection>
        </div>
      </section>
      <footer className="settings-savebar" aria-label={t("settings.save")}>
        <span
          className={`settings-feedback settings-feedback-${settingsStatus.kind}`}
          aria-live="polite"
        >
          {t(settingsStatus.message)}
        </span>
        <button
          className="primary-action settings-save-button"
          type="button"
          onClick={saveSettings}
          disabled={
            settingsStatus.kind === "loading" || settingsStatus.kind === "saving"
          }
        >
          {t("settings.save")}
        </button>
      </footer>
        </div>
      ) : (
        <div className="settings-loading-screen">
          <div className="settings-loading-nav">
            <LoadingSkeleton count={7} height={34} />
          </div>
          <div className="settings-loading-content">
            <LoadingSkeleton count={9} />
          </div>
        </div>
      )}
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
  const t = useI18n();
  return (
    <button
      className={`setting-input setting-input-shortcut hotkey-capture ${
        active ? "hotkey-capture-active" : ""
      }`}
      type="button"
      aria-label={t("settings.recordHotkey", { label })}
      aria-pressed={active}
      onClick={onStart}
      onBlur={() => {
        if (active) {
          onCancel();
        }
      }}
      onKeyDown={onKeyDown}
    >
      <span>
        {active ? t("settings.hotkeyCapture") : value || placeholder}
      </span>
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

function TextRow({
  label,
  value,
  loading,
}: {
  label: string;
  value: string;
  loading?: boolean;
}) {
  return (
    <div className="setting-row">
      <span>{label}</span>
      {loading ? (
        <LoadingSkeleton width={120} height={16} />
      ) : (
        <strong>{value}</strong>
      )}
    </div>
  );
}

export { SettingsWindow };
