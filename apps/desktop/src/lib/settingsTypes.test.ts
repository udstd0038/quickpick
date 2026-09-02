import { describe, expect, it } from "vitest";
import {
  defaultAppSettings,
  normalizeAppSettings,
  visionAiProviderOptions,
} from "./settingsTypes";
import { runningOnMac } from "./platform";

const expectedSettingsHotkey = runningOnMac ? "Command+Comma" : "Alt+0";
const expectedSelectionHotkey = runningOnMac ? "Command+Option+2" : "Alt+2";

describe("settingsTypes", () => {
  it("normalizes missing fields to defaults", () => {
    const settings = normalizeAppSettings({});

    expect(settings.settingsHotkey).toBe(expectedSettingsHotkey);
    expect(settings.selectionHotkey).toBe(expectedSelectionHotkey);
    expect(settings.allowClipboardFallback).toBe(true);
    expect(settings.panelOpacity).toBe(100);
    expect(settings.textAiProvider).toBe("deepseek");
  });

  it("preserves clipboard fallback opt-in", () => {
    const settings = normalizeAppSettings({
      allowClipboardFallback: true,
    });

    expect(settings.allowClipboardFallback).toBe(true);
  });

  it("keeps deepseek in screenshot model providers with vision default", () => {
    const provider = visionAiProviderOptions.find(
      (option) => option.id === "deepseek",
    );

    expect(provider?.visionModel).toBe("deepseek-v4-flash-vision-exp");

    const settings = normalizeAppSettings({
      visionAiProvider: "deepseek",
      visionAiBaseUrl: "",
      visionAiModel: "",
    });

    expect(settings.visionAiProvider).toBe("deepseek");
    expect(settings.visionAiBaseUrl).toBe("https://api.deepseek.com");
    expect(settings.visionAiModel).toBe("deepseek-v4-flash-vision-exp");
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
