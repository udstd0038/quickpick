import {
  targetLanguageForUiLanguage,
  translationLanguageOptions,
  uiLanguageOptions,
  type UiLanguage,
} from "./i18n";

export type AppSettings = {
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

export type SettingsStatus = {
  kind: "idle" | "loading" | "saving" | "success" | "error";
  message: string;
};

export type ApiKeyStatus = {
  configured: boolean;
};

export const defaultAppSettings: AppSettings = {
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

export function normalizeAppSettings(current: Partial<AppSettings>): AppSettings {
  const uiLanguage: UiLanguage = uiLanguageOptions.some(
    (option) => option.id === current.uiLanguage,
  )
    ? (current.uiLanguage as UiLanguage)
    : defaultAppSettings.uiLanguage;
  const defaultTargetLanguage = targetLanguageForUiLanguage(uiLanguage);
  return {
    ...defaultAppSettings,
    ...current,
    uiLanguage,
    settingsHotkey:
      current.settingsHotkey?.trim() || defaultAppSettings.settingsHotkey,
    selectionHotkey: current.selectionHotkey?.trim() || defaultAppSettings.selectionHotkey,
    screenshotHotkey: current.screenshotHotkey?.trim() || defaultAppSettings.screenshotHotkey,
    inputTranslateHotkey:
      current.inputTranslateHotkey?.trim() || defaultAppSettings.inputTranslateHotkey,
    translationTargetLanguage:
      uiLanguage === "system" || !translationLanguageOptions.some(
        (option) => option.id === current.translationTargetLanguage,
      )
        ? defaultTargetLanguage
        : (current.translationTargetLanguage as string),
    inputTranslateTargetLanguage:
      uiLanguage === "system" || !translationLanguageOptions.some(
        (option) => option.id === current.inputTranslateTargetLanguage,
      )
        ? defaultTargetLanguage
        : (current.inputTranslateTargetLanguage as string),
    windowEffect: current.windowEffect === "mica" ? "mica" : "acrylic",
    panelOpacity:
      Number.isFinite(current.panelOpacity) &&
      Number(current.panelOpacity) >= 30 &&
      Number(current.panelOpacity) <= 100
        ? Math.round(Number(current.panelOpacity))
        : defaultAppSettings.panelOpacity,
    aiTimeoutSeconds:
      Number.isFinite(current.aiTimeoutSeconds) &&
      Number(current.aiTimeoutSeconds) >= 5 &&
      Number(current.aiTimeoutSeconds) <= 120
        ? Number(current.aiTimeoutSeconds)
        : defaultAppSettings.aiTimeoutSeconds,
  };
}
