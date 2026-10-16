import {
  useEffect,
  useRef,
  useState,
  type PointerEvent as ReactPointerEvent,
} from "react";
import { LogicalSize } from "@tauri-apps/api/dpi";
import { listen } from "@tauri-apps/api/event";
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
  const [readError, setReadError] = useState("");
  const [busy, setBusy] = useState(false);
  const toolbarRef = useRef<HTMLElement | null>(null);
  const snapshotRevision = useRef(0);
  const hasSelection = snapshot?.status === "captured" && !!snapshot.text.trim();
  const statusMessage = hasSelection
    ? ""
    : t(
        readError ||
          snapshot?.message ||
          (snapshot ? "selection.noText" : "selection.reading"),
      );

  const clearSelection = () => {
    snapshotRevision.current += 1;
    setSnapshot(null);
    setReadError("");
    void clearPopupState("selection");
  };

  useEffect(() => {
    const currentWindow = getCurrentWindow();
    let disposed = false;
    let unlisten: (() => void) | undefined;

    const refreshSnapshot = () => {
      const revision = ++snapshotRevision.current;
      setSnapshot(null);
      setReadError("");
      void getSelectionSnapshot()
        .then((value) => {
          if (!disposed && revision === snapshotRevision.current) {
            setSnapshot(value);
          }
        })
        .catch((error) => {
          if (!disposed && revision === snapshotRevision.current) {
            setReadError(typeof error === "string" ? error : "selection.readFailed");
          }
        });
    };

    // Hidden WebViews stay mounted; each activation must replace the old selection.
    void listen<SelectionSnapshot>("selection-ready", (event) => {
      if (!disposed) {
        snapshotRevision.current += 1;
        setReadError("");
        setSnapshot(event.payload);
      }
    })
      .then((cleanup) => {
        if (disposed) {
          cleanup();
          return;
        }
        unlisten = cleanup;
        refreshSnapshot();
      })
      .catch(() => {
        if (!disposed) {
          refreshSnapshot();
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

    window.addEventListener("focus", refreshSnapshot);
    window.addEventListener("blur", closeOnBlur);
    window.addEventListener("keydown", closeOnEscape);

    return () => {
      disposed = true;
      unlisten?.();
      window.removeEventListener("focus", refreshSnapshot);
      window.removeEventListener("blur", closeOnBlur);
      window.removeEventListener("keydown", closeOnEscape);
    };
  }, []);

  useEffect(() => {
    const frame = window.requestAnimationFrame(() => {
      const element = toolbarRef.current;
      if (!element) {
        return;
      }
      const width = Math.ceil(element.getBoundingClientRect().width);
      const height = Math.ceil(element.getBoundingClientRect().height);
      void getCurrentWindow().setSize(new LogicalSize(width, height));
    });

    return () => window.cancelAnimationFrame(frame);
  }, [statusMessage]);

  const run = async (
    command: string,
    args?: Record<string, unknown>,
  ) => {
    if (busy || !hasSelection) {
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
      ref={toolbarRef}
      className="popup-window selection-window"
      onPointerDown={startDrag}
      style={{
        display: "flex",
        flexDirection: "column",
        gap: 8,
        alignItems: "center",
        width: "fit-content",
        height: "fit-content",
        minHeight: 32,
        padding: 8,
        boxSizing: "border-box",
        cursor: "grab",
        userSelect: "none",
      }}
      title={hasSelection ? snapshot.preview : statusMessage || t("selection.title")}
    >
      {statusMessage && (
        <div role="status" style={{ maxWidth: 420, fontSize: 12, cursor: "default" }}>
          {statusMessage}
        </div>
      )}
      <div style={{ display: "flex", gap: 8 }}>
        {actions.map((action) => (
          <ActionButton
            key={action.key}
            type="button"
            className="screenshot-tool-button"
            disabled={busy || !hasSelection}
            onClick={() =>
              run(action.command, "args" in action ? action.args : undefined)
            }
            style={{
              minHeight: 32,
              padding: "0 14px",
              cursor: busy ? "wait" : "pointer",
              whiteSpace: "nowrap",
            }}
          >
            {t(action.key)}
          </ActionButton>
        ))}
      </div>
    </main>
  );
}
