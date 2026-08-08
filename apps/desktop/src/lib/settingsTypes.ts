export type AppSettings = {
  autostartEnabled: boolean;
  selectionHotkey: string;
  screenshotHotkey: string;
  inputTranslateHotkey: string;
  themeMode: "system" | "light" | "dark";
  windowEffect: "mica";
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
  selectionHotkey: "Alt+2",
  screenshotHotkey: "Alt+3",
  inputTranslateHotkey: "Alt+4",
  themeMode: "system",
  windowEffect: "mica",
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
  return {
    ...defaultAppSettings,
    ...current,
    selectionHotkey: current.selectionHotkey?.trim() || defaultAppSettings.selectionHotkey,
    screenshotHotkey: current.screenshotHotkey?.trim() || defaultAppSettings.screenshotHotkey,
    inputTranslateHotkey:
      current.inputTranslateHotkey?.trim() || defaultAppSettings.inputTranslateHotkey,
    aiTimeoutSeconds:
      Number.isFinite(current.aiTimeoutSeconds) &&
      Number(current.aiTimeoutSeconds) >= 5 &&
      Number(current.aiTimeoutSeconds) <= 120
        ? Number(current.aiTimeoutSeconds)
        : defaultAppSettings.aiTimeoutSeconds,
  };
}
