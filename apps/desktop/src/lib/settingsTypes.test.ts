import { describe, expect, it } from "vitest";
import {
  defaultAppSettings,
  normalizeAppSettings,
} from "./settingsTypes";

describe("settingsTypes", () => {
  it("normalizes missing fields to defaults", () => {
    const settings = normalizeAppSettings({});

    expect(settings.settingsHotkey).toBe("Alt+0");
    expect(settings.selectionHotkey).toBe("Alt+2");
    expect(settings.panelOpacity).toBe(70);
    expect(settings.textAiProvider).toBe("deepseek");
  });

  it("fills provider defaults when base url or model is empty", () => {
    const settings = normalizeAppSettings({
      textAiProvider: "deepseek",
      textAiBaseUrl: "",
      textAiModel: "",
      inputAiProvider: "deepseek",
      inputAiBaseUrl: "",
      inputAiModel: "",
    });

    expect(settings.textAiBaseUrl).toBe("https://api.deepseek.com");
    expect(settings.textAiModel).toBe("deepseek-v4-flash");
    expect(settings.inputAiBaseUrl).toBe("https://api.deepseek.com");
    expect(settings.inputAiModel).toBe("deepseek-v4-flash");
  });

  it("keeps explicit language target when ui language is explicit", () => {
    const settings = normalizeAppSettings({
      uiLanguage: "en",
      inputTranslateTargetLanguage: "ja",
    });

    expect(settings.inputTranslateTargetLanguage).toBe("ja");
    expect(settings.translationTargetLanguage).toBe("en");
  });
});
