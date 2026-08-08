import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { SettingsWindow } from "./windows/settings/SettingsWindow";

export default function App() {
  const [coreStatus, setCoreStatus] = useState("正在检查核心连接");

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

  return <SettingsWindow coreStatus={coreStatus} />;
}
