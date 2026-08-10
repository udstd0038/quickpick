import { lazy, Suspense, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  defaultAppSettings,
  normalizeAppSettings,
  type AppSettings,
} from "./lib/settingsTypes";
import { setI18nLanguage } from "./lib/i18n";
import { SettingsWindow } from "./windows/settings/SettingsWindow";

const SelectionWindow = lazy(() =>
  import("./windows/selection/SelectionWindow").then((module) => ({
    default: module.SelectionWindow,
  })),
);
const ResultWindow = lazy(() =>
  import("./windows/result/ResultWindow").then((module) => ({
    default: module.ResultWindow,
  })),
);
const InputWindow = lazy(() =>
  import("./windows/input/InputWindow").then((module) => ({
    default: module.InputWindow,
  })),
);
const ScreenshotOverlay = lazy(() =>
  import("./windows/screenshot/ScreenshotOverlay").then((module) => ({
    default: module.ScreenshotOverlay,
  })),
);

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

  return (
    <Suspense fallback={null}>
      {windowKind === "selection" ? (
        <SelectionWindow />
      ) : windowKind === "result" ? (
        <ResultWindow />
      ) : windowKind === "input" ? (
        <InputWindow />
      ) : windowKind === "screenshot_overlay" ? (
        <ScreenshotOverlay />
      ) : (
        <SettingsWindow />
      )}
    </Suspense>
  );
}
