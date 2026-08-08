import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

const actions = [
  { label: "复制", command: "copy_selection_text" },
  { label: "翻译", command: "run_text_ai_action", args: { action: "translate" } },
  { label: "总结", command: "run_text_ai_action", args: { action: "summarize" } },
  { label: "搜索", command: "search_selection_text" },
] as const;

export function SelectionWindow() {
  const [busy, setBusy] = useState(false);

  const run = async (command: string, args?: Record<string, unknown>) => {
    setBusy(true);
    try {
      await invoke(command, args ?? {});
      await getCurrentWindow().hide();
    } finally {
      setBusy(false);
    }
  };

  return (
    <main
      style={{
        display: "flex",
        gap: 8,
        alignItems: "center",
        justifyContent: "center",
        width: "100vw",
        height: "100vh",
        padding: 8,
        boxSizing: "border-box",
        color: "#e7ecf3",
        background: "rgba(10, 14, 19, 0.94)",
        fontFamily: "Segoe UI, Microsoft YaHei, sans-serif",
      }}
    >
      {actions.map((action) => (
        <button
          key={action.label}
          type="button"
          disabled={busy}
          onClick={() => run(action.command, "args" in action ? action.args : undefined)}
          style={{
            minWidth: 64,
            minHeight: 38,
            border: "1px solid rgba(164, 180, 202, 0.2)",
            borderRadius: 8,
            padding: "0 14px",
            color: "#e7ecf3",
            background: "rgba(30, 36, 45, 0.96)",
            cursor: "pointer",
          }}
        >
          {action.label}
        </button>
      ))}
    </main>
  );
}
