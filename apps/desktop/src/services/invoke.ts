import { invoke } from "@tauri-apps/api/core";

export type SelectionActionResult = {
  message: string;
};

export function requestInputTranslation(input: {
  inputText: string;
  sourceLanguage: string;
  targetLanguage: string;
  direction: "left" | "right";
}) {
  return invoke<SelectionActionResult>("request_input_translation", input);
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

export function copyResultContent() {
  return invoke<SelectionActionResult>("copy_result_content");
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
