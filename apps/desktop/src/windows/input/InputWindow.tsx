import { useInputStore } from "../../stores/inputStore";

export function InputWindow() {
  const text = useInputStore((state) => state.text);
  const result = useInputStore((state) => state.result);
  const status = useInputStore((state) => state.status);
  const setText = useInputStore((state) => state.setText);
  const setStatus = useInputStore((state) => state.setStatus);

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
      <textarea
        value={text}
        onChange={(event) => setText(event.currentTarget.value)}
        placeholder="请输入要翻译的文本"
        style={{
          minHeight: 72,
          resize: "none",
          border: "1px solid rgba(164, 180, 202, 0.2)",
          borderRadius: 10,
          padding: 10,
          color: "#e7ecf3",
          background: "rgba(19, 23, 29, 0.94)",
        }}
      />
      <button
        type="button"
        onClick={() => setStatus("loading")}
        disabled={!text.trim()}
      >
        翻译
      </button>
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
        {status === "loading" ? "正在处理..." : result || "暂无结果"}
      </div>
    </main>
  );
}
