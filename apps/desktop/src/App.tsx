import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { SettingsWindow } from "./windows/settings/SettingsWindow";
import { SelectionWindow } from "./windows/selection/SelectionWindow";
import { ResultWindow } from "./windows/result/ResultWindow";
import { InputWindow } from "./windows/input/InputWindow";
import { ScreenshotOverlay } from "./windows/screenshot/ScreenshotOverlay";
import { ScreenshotPreview } from "./windows/screenshot/ScreenshotPreview";

export default function App() {
  const [coreStatus, setCoreStatus] = useState("正在检查核心连接");
  const [windowKind, setWindowKind] = useState("main");

  useEffect(() => {
    let mounted = true;
    invoke<string>("ping")
      .then((value) => {
        if (mounted) {
          setCoreStatus(value);
        }
      })
      .catch(() => {
        if (mounted) {
          setCoreStatus("前端预览模式");
        }
      });

    return () => {
      mounted = false;
    };
  }, []);

  useEffect(() => {
    const label = getCurrentWindow().label;
    const knownLabels = new Set([
      "selection",
      "result",
      "input",
      "screenshot_overlay",
      "screenshot_preview",
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
    case "screenshot_preview":
      return <ScreenshotPreview />;
    default:
      return <SettingsWindow coreStatus={coreStatus} />;
  }
}
