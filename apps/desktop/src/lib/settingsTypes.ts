import {
  targetLanguageForUiLanguage,
  translationLanguageOptions,
  uiLanguageOptions,
  type UiLanguage,
} from "./i18n";

export type AppSettings = {
  autostartEnabled: boolean;
  allowClipboardFallback: boolean;
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

export type AiProviderOption = {
  id: string;
  label: string;
  baseUrl: string;
  textModel: string;
  visionModel: string;
};

export const textAiProviderOptions: AiProviderOption[] = [
  {
    id: "openai_compatible",
    label: "providers.openaiCompatible",
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
    label: "providers.xiaomiMimo",
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

export const visionAiProviderOptions = textAiProviderOptions.filter(
  (option) => option.id !== "deepseek",
);
export const inputAiProviderOptions = textAiProviderOptions;

export const themeModeOptions = [
  { id: "system", label: "settings.themeSystem" },
  { id: "light", label: "settings.themeLight" },
  { id: "dark", label: "settings.themeDark" },
] as const;

export const windowEffectOptions = [
  { id: "acrylic", label: "Acrylic" },
  { id: "mica", label: "Mica" },
] as const;

export function applyTextAiProviderDefaults(
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

export function applyVisionAiProviderDefaults(
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

export function applyInputAiProviderDefaults(
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

export const defaultAppSettings: AppSettings = {
  autostartEnabled: true,
  allowClipboardFallback: true,
  settingsHotkey: "Alt+0",
  selectionHotkey: "Alt+2",
  screenshotHotkey: "Alt+3",
  inputTranslateHotkey: "Alt+4",
  uiLanguage: "system",
  themeMode: "system",
  windowEffect: "acrylic",
  panelOpacity: 100,
  textAiProvider: "deepseek",
  textAiBaseUrl: "https://api.deepseek.com",
  textAiModel: "deepseek-v4-flash",
  visionAiProvider: "xiaomi_mimo",
  visionAiBaseUrl: "https://api.xiaomimimo.com/v1",
  visionAiModel: "mimo-v2.5",
  inputAiProvider: "deepseek",
  inputAiBaseUrl: "https://api.deepseek.com",
  inputAiModel: "deepseek-v4-flash",
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
  const textProvider =
    textAiProviderOptions.find(
      (option) => option.id === current.textAiProvider,
    ) || textAiProviderOptions[1];
  const visionProvider =
    visionAiProviderOptions.find(
      (option) => option.id === current.visionAiProvider,
    ) || visionAiProviderOptions[0];
  const inputProvider =
    inputAiProviderOptions.find(
      (option) => option.id === current.inputAiProvider,
    ) || inputAiProviderOptions[1];
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
    textAiProvider: textProvider.id,
    textAiBaseUrl:
      current.textAiBaseUrl?.trim() || textProvider.baseUrl,
    textAiModel:
      current.textAiModel?.trim() || textProvider.textModel,
    visionAiProvider: visionProvider.id,
    visionAiBaseUrl:
      current.visionAiBaseUrl?.trim() || visionProvider.baseUrl,
    visionAiModel:
      current.visionAiModel?.trim() ||
      (visionProvider.id === "openai_compatible"
        ? ""
        : visionProvider.visionModel),
    inputAiProvider: inputProvider.id,
    inputAiBaseUrl:
      current.inputAiBaseUrl?.trim() || inputProvider.baseUrl,
    inputAiModel:
      current.inputAiModel?.trim() ||
      (inputProvider.id === "openai_compatible"
        ? ""
        : inputProvider.textModel),
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
