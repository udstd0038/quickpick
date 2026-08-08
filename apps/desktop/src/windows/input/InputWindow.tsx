import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useInputStore } from "../../stores/inputStore";

type InputReadyPayload = {
  status: "waiting" | "loading" | "success" | "error";
  content: string;
  detail: string;
  sourceLanguage: string;
  targetLanguage: string;
  direction: "left" | "right";
};

export function InputWindow() {
  const text = useInputStore((state) => state.text);
  const result = useInputStore((state) => state.result);
  const status = useInputStore((state) => state.status);
  const sourceLanguage = useInputStore((state) => state.sourceLanguage);
  const targetLanguage = useInputStore((state) => state.targetLanguage);
  const direction = useInputStore((state) => state.direction);
  const setText = useInputStore((state) => state.setText);
  const setResult = useInputStore((state) => state.setResult);
  const setStatus = useInputStore((state) => state.setStatus);
  const setLanguages = useInputStore((state) => state.setLanguages);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen<InputReadyPayload>("input-ready", (event) => {
      const payload = event.payload;
      setResult(payload.content || payload.detail);
      setStatus(payload.status);
      setLanguages(
        payload.sourceLanguage,
        payload.targetLanguage,
        payload.direction,
      );
    }).then((cleanup) => {
      unlisten = cleanup;
    });

    return () => {
      unlisten?.();
    };
  }, [setLanguages, setResult, setStatus]);

  const translate = async () => {
    if (!text.trim() || busy) {
      return;
    }
    setBusy(true);
    setStatus("loading");
    try {
      await invoke("request_input_translation", {
        inputText: text,
        sourceLanguage,
        targetLanguage,
        direction,
      });
    } finally {
      setBusy(false);
    }
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
        onClick={translate}
        disabled={!text.trim() || busy}
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
