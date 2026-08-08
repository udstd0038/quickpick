import {
  useEffect,
  type CSSProperties,
  type PointerEvent as ReactPointerEvent,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useResultStore } from "../../stores/resultStore";

type ResultSnapshotPayload = {
  status: "empty" | "placeholder" | "loading" | "success" | "error";
  title: string;
  content: string;
  detail: string;
  sourceLanguage: string;
  targetLanguage: string;
  translationDirection: string;
  canSwitchLanguage: boolean;
};

const statusLabels = {
  empty: "暂无结果",
  placeholder: "待处理",
  loading: "正在处理",
  success: "翻译结果",
  error: "处理失败",
} as const;

export function ResultWindow() {
  const content = useResultStore((state) => state.content);
  const detail = useResultStore((state) => state.detail);
  const status = useResultStore((state) => state.status);
  const pinned = useResultStore((state) => state.pinned);
  const setContent = useResultStore((state) => state.setContent);
  const setStatus = useResultStore((state) => state.setStatus);
  const setPinned = useResultStore((state) => state.setPinned);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen<ResultSnapshotPayload>("result-ready", (event) => {
      const payload = event.payload;
      setContent(payload.content, payload.detail);
      setStatus(payload.status);
    }).then((cleanup) => {
      unlisten = cleanup;
    });

    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        void getCurrentWindow().hide();
      }
    };
    window.addEventListener("keydown", closeOnEscape);

    return () => {
      unlisten?.();
      window.removeEventListener("keydown", closeOnEscape);
    };
  }, [setContent, setPinned, setStatus]);

  const startDrag = (event: ReactPointerEvent<HTMLElement>) => {
    const target = event.target as HTMLElement;
    if (target.closest("button, select, textarea, input, a")) {
      return;
    }
    void getCurrentWindow().startDragging();
  };

  const copy = async () => {
    try {
      await invoke("copy_result_content");
    } catch (error) {
      console.error("copy result failed", error);
    }
  };

  const close = () => {
    void getCurrentWindow().hide();
  };

  return (
    <main
      style={{
        display: "flex",
        flexDirection: "column",
        gap: 10,
        width: "100vw",
        height: "100vh",
        padding: 12,
        boxSizing: "border-box",
        color: "#e7ecf3",
        background: "rgba(10, 14, 19, 0.96)",
        fontFamily: "Segoe UI, Microsoft YaHei, sans-serif",
      }}
    >
      <header
        onPointerDown={startDrag}
        style={{
          display: "flex",
          alignItems: "center",
          gap: 8,
          minHeight: 36,
          cursor: "grab",
        }}
      >
        <div
          style={{
            flex: 1,
            minWidth: 0,
            overflow: "hidden",
            color: "#d8e2ef",
            fontSize: 14,
            fontWeight: 700,
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
          }}
        >
          {statusLabels[status]}
        </div>
        <button
          type="button"
          onClick={() => setPinned(!pinned)}
          style={controlButtonStyle}
        >
          {pinned ? "取消固定" : "固定"}
        </button>
        <button type="button" onClick={close} style={controlButtonStyle}>
          关闭
        </button>
      </header>
      <div
        style={{
          flex: 1,
          minHeight: 0,
          border: "1px solid rgba(164, 180, 202, 0.16)",
          borderRadius: 10,
          padding: 12,
          overflow: "auto",
          background: "rgba(19, 23, 29, 0.94)",
          color: "#e7ecf3",
          whiteSpace: "pre-wrap",
        }}
      >
        {status === "loading"
          ? "正在处理..."
          : content || detail || "暂无结果"}
      </div>
      <footer
        style={{
          display: "flex",
          justifyContent: "flex-end",
          gap: 8,
          minHeight: 36,
          alignItems: "center",
        }}
      >
        <button
          type="button"
          onClick={copy}
          disabled={status !== "success"}
          style={actionButtonStyle}
        >
          复制
        </button>
      </footer>
    </main>
  );
}

const controlButtonStyle: CSSProperties = {
  flex: "0 0 auto",
  minHeight: 30,
  border: "1px solid rgba(164, 180, 202, 0.2)",
  borderRadius: 7,
  padding: "0 10px",
  color: "#e7ecf3",
  background: "rgba(30, 36, 45, 0.96)",
  cursor: "pointer",
};

const actionButtonStyle: CSSProperties = {
  flex: "0 0 auto",
  minHeight: 30,
  border: "1px solid rgba(164, 180, 202, 0.2)",
  borderRadius: 7,
  padding: "0 12px",
  color: "#e7ecf3",
  background: "rgba(30, 36, 45, 0.96)",
  cursor: "pointer",
};
