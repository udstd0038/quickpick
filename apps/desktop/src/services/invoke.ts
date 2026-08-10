import { invoke } from "@tauri-apps/api/core";
import type {
  ApiKeyStatus,
  AppSettings,
} from "../lib/settingsTypes";

export type SelectionActionResult = {
  message: string;
};

export type SelectionSnapshot = {
  status: "captured" | "empty" | "unsupported" | "error";
  text: string;
  preview: string;
  charCount: number;
  message: string;
  source: string;
};

export function getAppSettings() {
  return invoke<AppSettings>("get_app_settings");
}

export function saveSettings(settings: AppSettings) {
  return invoke<SelectionActionResult>("save_app_settings", { settings });
}

export function getApiKeyStatus(scope: "text" | "vision" | "input") {
  return invoke<ApiKeyStatus>("get_api_key_status", { scope });
}

export function saveApiKey(
  scope: "text" | "vision" | "input",
  apiKey: string,
) {
  return invoke<SelectionActionResult>("save_api_key", { apiKey, scope });
}

export function clearApiKey(scope: "text" | "vision" | "input") {
  return invoke<SelectionActionResult>("clear_api_key", { scope });
}

export function setHotkeyCaptureMode(enabled: boolean) {
  return invoke<SelectionActionResult>("set_hotkey_capture_mode", { enabled });
}

export function getSelectionSnapshot() {
  return invoke<SelectionSnapshot>("get_selection_snapshot");
}

export function requestInputTranslation(input: {
  inputText: string;
  sourceLanguage: string;
  targetLanguage: string;
  direction: "left" | "right";
}) {
  return invoke<SelectionActionResult>("request_input_translation", input);
}

export function requestResultTranslation(input: {
  sourceLanguage: string;
  targetLanguage: string;
  direction: "left" | "right";
}) {
  return invoke<SelectionActionResult>("request_result_translation", input);
}

export function captureRegionRect(input: {
  screenX: number;
  screenY: number;
  width: number;
  height: number;
  action: string;
}) {
  return invoke<SelectionActionResult>("capture_region_rect", input);
}

export type MonitorScreenshotPayload = {
  width: number;
  height: number;
  monitorX: number;
  monitorY: number;
  monitorName: string;
  pngDataUrl: string;
};

export function captureMonitorScreenshot() {
  return invoke<MonitorScreenshotPayload>("capture_monitor_screenshot");
}

export function copyResultContent() {
  return invoke<SelectionActionResult>("copy_result_content");
}

export function copyInputResult() {
  return invoke<SelectionActionResult>("copy_input_result");
}

export function runSelectionAction(
  action: "copy" | "translate" | "summarize" | "search",
) {
  switch (action) {
    case "copy":
      return invoke<SelectionActionResult>("copy_selection_text");
    case "translate":
      return invoke<SelectionActionResult>("run_text_ai_action", {
        action: "translate",
      });
    case "summarize":
      return invoke<SelectionActionResult>("run_text_ai_action", {
        action: "summarize",
      });
    case "search":
      return invoke<SelectionActionResult>("search_selection_text");
  }
}
