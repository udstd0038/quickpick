import {
  useEffect,
  useState,
  type PointerEvent as ReactPointerEvent,
} from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  clearPopupState,
  getSelectionSnapshot,
  runSelectionAction,
  type SelectionSnapshot,
} from "../../services/invoke";
import { ActionButton } from "../../components/ActionButton";
import { useI18n } from "../../lib/i18n";

const actions = [
  { key: "common.copy", command: "copy_selection_text" },
  {
    key: "common.translate",
    command: "run_text_ai_action",
    args: { action: "translate" },
  },
  {
    key: "common.summarize",
    command: "run_text_ai_action",
    args: { action: "summarize" },
  },
  { key: "common.search", command: "search_selection_text" },
] as const;

export function SelectionWindow() {
  const t = useI18n();
  const [snapshot, setSnapshot] = useState<SelectionSnapshot | null>(null);
  const [busy, setBusy] = useState(false);

  const clearSelection = () => {
    setSnapshot(null);
    void clearPopupState("selection");
  };

  useEffect(() => {
    const currentWindow = getCurrentWindow();
    let disposed = false;

    getSelectionSnapshot()
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
      clearSelection();
      void currentWindow.hide();
    };
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        clearSelection();
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
      const action =
        command === "copy_selection_text"
          ? "copy"
          : command === "search_selection_text"
            ? "search"
            : args?.action === "summarize"
              ? "summarize"
              : "translate";
      await runSelectionAction(action);
    } catch (error) {
      console.error("selection action failed", error);
    } finally {
      setBusy(false);
      clearSelection();
      await getCurrentWindow().hide();
    }
  };

  const startDrag = (event: ReactPointerEvent<HTMLElement>) => {
    const target = event.target as HTMLElement;
    if (target.closest("button, select, textarea, input, a")) {
      return;
    }
    void getCurrentWindow().startDragging();
  };

  return (
    <main
      className="popup-window selection-window"
      onPointerDown={startDrag}
      style={{
        display: "grid",
        gridTemplateColumns: "repeat(4, minmax(0, 1fr))",
        gap: 8,
        alignItems: "center",
        width: "100vw",
        height: "100vh",
        padding: "8px",
        boxSizing: "border-box",
        cursor: "grab",
        userSelect: "none",
      }}
      title={snapshot?.preview || snapshot?.message || t("selection.title")}
    >
      {actions.map((action) => (
        <ActionButton
          key={action.key}
          type="button"
          className="selection-tool-button"
          disabled={busy}
          onClick={() => run(action.command, "args" in action ? action.args : undefined)}
          style={{
            width: "100%",
            height: 32,
            minHeight: 32,
            paddingTop: "8px",
            paddingRight: "8px",
            paddingBottom: "8px",
            paddingLeft: "8px",
            cursor: busy ? "wait" : "pointer",
            whiteSpace: "nowrap",
          }}
        >
          {t(action.key)}
        </ActionButton>
      ))}
    </main>
  );
}
