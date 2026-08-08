import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useResultStore } from "../../stores/resultStore";

export function ResultWindow() {
  const content = useResultStore((state) => state.content);
  const status = useResultStore((state) => state.status);
  const pinned = useResultStore((state) => state.pinned);
  const setPinned = useResultStore((state) => state.setPinned);

  const copy = async () => {
    await invoke("copy_result_content");
  };

  const close = () => {
    void getCurrentWindow().hide();
  };

  return (
    <main
      style={{
        display: "flex",
        flexDirection: "column",
        gap: 12,
        width: "100vw",
        height: "100vh",
        padding: 14,
        boxSizing: "border-box",
        color: "#e7ecf3",
        background: "rgba(10, 14, 19, 0.96)",
        fontFamily: "Segoe UI, Microsoft YaHei, sans-serif",
      }}
    >
      <div style={{ display: "flex", gap: 8, justifyContent: "flex-end" }}>
        <button type="button" onClick={() => setPinned(!pinned)}>
          {pinned ? "取消固定" : "固定"}
        </button>
        <button type="button" onClick={close}>
          关闭
        </button>
      </div>
      <div
        style={{
          flex: 1,
          border: "1px solid rgba(164, 180, 202, 0.16)",
          borderRadius: 12,
          padding: 12,
          overflow: "auto",
          background: "rgba(19, 23, 29, 0.94)",
          whiteSpace: "pre-wrap",
        }}
      >
        {status === "loading" ? "正在处理..." : content || "暂无结果"}
      </div>
      <button type="button" onClick={copy} disabled={status !== "success"}>
        复制
      </button>
    </main>
  );
}
