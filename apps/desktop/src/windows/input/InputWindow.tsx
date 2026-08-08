import {
  useEffect,
  useState,
  type CSSProperties,
  type KeyboardEvent as ReactKeyboardEvent,
  type PointerEvent as ReactPointerEvent,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useInputStore } from "../../stores/inputStore";

type InputReadyPayload = {
  status: "waiting" | "loading" | "success" | "error";
  content: string;
  detail: string;
  sourceLanguage: string;
  targetLanguage: string;
  direction: "left" | "right";
};

const languageOptions = [
  { value: "auto", label: "自动检测" },
  { value: "zh-Hans", label: "简体中文" },
  { value: "zh-Hant", label: "繁体中文" },
  { value: "en", label: "英语" },
  { value: "ja", label: "日语" },
  { value: "ko", label: "韩语" },
  { value: "fr", label: "法语" },
  { value: "de", label: "德语" },
  { value: "ru", label: "俄语" },
  { value: "es", label: "西班牙语" },
] as const;

function languageLabel(value: string) {
  return (
    languageOptions.find((option) => option.value === value)?.label || value
  );
}

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
  const [pinned, setPinned] = useState(false);

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
  }, [setLanguages, setResult, setStatus]);

  const startDrag = (event: ReactPointerEvent<HTMLElement>) => {
    const target = event.target as HTMLElement;
    if (target.closest("button, select, textarea, input, a")) {
      return;
    }
    void getCurrentWindow().startDragging();
  };

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
    } catch (error) {
      setStatus("error");
      setResult(typeof error === "string" ? error : "翻译请求失败");
    } finally {
      setBusy(false);
    }
  };

  const translateOnEnter = (event: ReactKeyboardEvent<HTMLTextAreaElement>) => {
    if ((event.ctrlKey || event.metaKey) && event.key === "Enter") {
      event.preventDefault();
      void translate();
    }
  };

  const togglePinned = () => {
    setPinned((current) => {
      const next = !current;
      void getCurrentWindow().setAlwaysOnTop(next);
      return next;
    });
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
          输入翻译
        </div>
        <button
          type="button"
          onClick={togglePinned}
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
          display: "grid",
          gridTemplateColumns: "1fr 36px 1fr",
          gap: 8,
          alignItems: "center",
        }}
      >
        <select
          value={sourceLanguage}
          onChange={(event) =>
            setLanguages(event.currentTarget.value, targetLanguage, direction)
          }
          style={selectStyle}
          aria-label="源语言"
        >
          {languageOptions.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
        <button
          type="button"
          onClick={() =>
            setLanguages(
              sourceLanguage,
              targetLanguage,
              direction === "left" ? "right" : "left",
            )
          }
          style={controlButtonStyle}
          aria-label="切换翻译方向"
        >
          {direction === "left" ? "←" : "→"}
        </button>
        <select
          value={targetLanguage}
          onChange={(event) =>
            setLanguages(sourceLanguage, event.currentTarget.value, direction)
          }
          style={selectStyle}
          aria-label="目标语言"
        >
          {languageOptions.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      </div>

      <textarea
        value={text}
        onChange={(event) => setText(event.currentTarget.value)}
        onKeyDown={translateOnEnter}
        placeholder="输入要翻译的文本"
        autoFocus
        style={{
          minHeight: 76,
          resize: "none",
          border: "1px solid rgba(164, 180, 202, 0.2)",
          borderRadius: 10,
          padding: 10,
          outline: "none",
          color: "#e7ecf3",
          background: "rgba(19, 23, 29, 0.94)",
        }}
      />

      <div
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
          onClick={translate}
          disabled={!text.trim() || busy}
          style={actionButtonStyle}
        >
          {busy ? "翻译中" : "翻译"}
        </button>
      </div>

      <div
        style={{
          flex: 1,
          minHeight: 0,
          border: "1px solid rgba(164, 180, 202, 0.16)",
          borderRadius: 10,
          padding: 12,
          overflow: "auto",
          background: "rgba(19, 23, 29, 0.94)",
          whiteSpace: "pre-wrap",
        }}
      >
        {status === "loading"
          ? "正在处理..."
          : result || `源语言：${languageLabel(sourceLanguage)} → 目标语言：${languageLabel(targetLanguage)}`}
      </div>
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

const selectStyle: CSSProperties = {
  width: "100%",
  minHeight: 30,
  border: "1px solid rgba(164, 180, 202, 0.2)",
  borderRadius: 7,
  padding: "0 8px",
  color: "#e7ecf3",
  background: "rgba(30, 36, 45, 0.96)",
  cursor: "pointer",
};
