import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  defaultAppSettings,
  normalizeAppSettings,
  type AppSettings,
} from "./lib/settingsTypes";
import {
  setI18nLanguage,
} from "./lib/i18n";
import { SettingsWindow } from "./windows/settings/SettingsWindow";
import { SelectionWindow } from "./windows/selection/SelectionWindow";
import { ResultWindow } from "./windows/result/ResultWindow";
import { InputWindow } from "./windows/input/InputWindow";
import { ScreenshotOverlay } from "./windows/screenshot/ScreenshotOverlay";

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

function applyGlobalAppearance(settings: AppSettings) {
  const root = document.documentElement;
  setI18nLanguage(settings.uiLanguage);
  const resolvedThemeMode: string =
    settings.themeMode === "system" ? currentSystemTheme() : settings.themeMode;
  const effectiveTheme =
    resolvedThemeMode === "dark" || resolvedThemeMode === "workbench"
      ? "workbench"
      : "light";

  root.dataset.theme = effectiveTheme;
  root.dataset.themePreference = settings.themeMode;
  root.dataset.windowEffect = settings.windowEffect;
  root.style.colorScheme = effectiveTheme === "light" ? "light" : "dark";
  if (settings.windowEffect === "mica") {
    for (const name of appearanceAlphaVariables) {
      root.style.removeProperty(name);
    }
    return;
  }

  const panelOpacity =
    Math.min(100, Math.max(30, Number(settings.panelOpacity) || 70)) / 100;
  for (const name of appearanceAlphaVariables) {
    root.style.setProperty(name, String(panelOpacity));
  }
}

export default function App() {
  const [windowKind, setWindowKind] = useState("main");

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    invoke<AppSettings>("get_app_settings")
      .then((settings) => {
        applyGlobalAppearance(normalizeAppSettings(settings));
      })
      .catch(() => {
        applyGlobalAppearance(defaultAppSettings);
      });

    listen<AppSettings>("app-settings-changed", (event) => {
      applyGlobalAppearance(normalizeAppSettings(event.payload));
    }).then((cleanup) => {
      unlisten = cleanup;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    const label = getCurrentWindow().label;
    const knownLabels = new Set([
      "selection",
      "result",
      "input",
      "screenshot_overlay",
    ]);
    setWindowKind(knownLabels.has(label) ? label : "main");
  }, []);

  switch (windowKind) {
    case "selection":
      return <SelectionWindow />;
    case "result":
      return <ResultWindow />;
    case "input":
      return <InputWindow />;
    case "screenshot_overlay":
      return <ScreenshotOverlay />;
    default:
      return <SettingsWindow />;
  }
}
