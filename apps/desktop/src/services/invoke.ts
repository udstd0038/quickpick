import { invoke } from "@tauri-apps/api/core";

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

export function getSelectionSnapshot() {
  return invoke<SelectionSnapshot>("get_selection_snapshot");
}

export type ResultSnapshotPayload = {
  status: "empty" | "placeholder" | "loading" | "success" | "error";
  title: string;
  content: string;
  detail: string;
  sourceLanguage: string;
  targetLanguage: string;
  translationDirection: "left" | "right";
  canSwitchLanguage: boolean;
};

export function getResultSnapshot() {
  return invoke<ResultSnapshotPayload>("get_result_snapshot");
}

export type InputReadyPayload = {
  status: "waiting" | "loading" | "success" | "error";
  content: string;
  detail: string;
  sourceLanguage: string;
  targetLanguage: string;
  direction: "left" | "right";
};

export function getInputSnapshot() {
  return invoke<InputReadyPayload | null>("get_input_snapshot");
}

export function getScreenshotSnapshot() {
  return invoke<MonitorScreenshotPayload | null>("get_screenshot_snapshot");
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
