import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

type SelectionSnapshot = {
  status: "captured" | "empty" | "unsupported" | "error";
  text: string;
  preview: string;
  charCount: number;
  message: string;
  source: string;
};

const actions = [
  { label: "复制", command: "copy_selection_text" },
  { label: "翻译", command: "run_text_ai_action", args: { action: "translate" } },
  { label: "总结", command: "run_text_ai_action", args: { action: "summarize" } },
  { label: "搜索", command: "search_selection_text" },
] as const;

export function SelectionWindow() {
  const [snapshot, setSnapshot] = useState<SelectionSnapshot | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    const currentWindow = getCurrentWindow();
    let disposed = false;

    invoke<SelectionSnapshot>("get_selection_snapshot")
      .then((value) => {
        if (!disposed) {
          setSnapshot(value);
        }
      })
      .catch(() => {
        if (!disposed) {
          setSnapshot(null);
        }
      });

    const closeOnBlur = () => {
      void currentWindow.hide();
    };
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        void currentWindow.hide();
      }
    };

    void currentWindow.setFocus();
    window.focus();
    window.addEventListener("blur", closeOnBlur);
    window.addEventListener("keydown", closeOnEscape);

    return () => {
      disposed = true;
      window.removeEventListener("blur", closeOnBlur);
      window.removeEventListener("keydown", closeOnEscape);
    };
  }, []);

  const run = async (
    command: string,
    args?: Record<string, unknown>,
  ) => {
    if (busy) {
      return;
    }

    setBusy(true);
    try {
      await invoke(command, args ?? {});
    } catch (error) {
      console.error("selection action failed", error);
    } finally {
      setBusy(false);
      await getCurrentWindow().hide();
    }
  };

  return (
    <main
      style={{
        display: "grid",
        gridTemplateColumns: "repeat(4, minmax(0, 1fr))",
        gap: 8,
        alignItems: "center",
        width: "100vw",
        height: "100vh",
        padding: 8,
        boxSizing: "border-box",
        color: "#e7ecf3",
        background: "rgba(10, 14, 19, 0.96)",
        fontFamily: "Segoe UI, Microsoft YaHei, sans-serif",
        userSelect: "none",
      }}
      title={snapshot?.preview || snapshot?.message || "QuickPick 划词"}
    >
      {actions.map((action) => (
        <button
          key={action.label}
          type="button"
          disabled={busy}
          onClick={() => run(action.command, "args" in action ? action.args : undefined)}
          style={{
            width: "100%",
            minHeight: 38,
            border: "1px solid rgba(164, 180, 202, 0.24)",
            borderRadius: 7,
            padding: "0 8px",
            color: "#e7ecf3",
            background: "rgba(30, 36, 45, 0.96)",
            cursor: busy ? "wait" : "pointer",
            whiteSpace: "nowrap",
          }}
        >
          {action.label}
        </button>
      ))}
    </main>
  );
}
