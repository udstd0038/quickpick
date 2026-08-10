import {
  useEffect,
  useState,
  type CSSProperties,
  type PointerEvent as ReactPointerEvent,
} from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Pin, PinOff, X } from "lucide-react";
import {
  copyResultContent,
  requestResultTranslation,
} from "../../services/invoke";
import { GlassSelect } from "../../components/GlassSelect";
import { ActionButton } from "../../components/ActionButton";
import { useResultStore } from "../../stores/resultStore";

type ResultSnapshotPayload = {
  status: "empty" | "placeholder" | "loading" | "success" | "error";
  title: string;
  content: string;
  detail: string;
  sourceLanguage: string;
  targetLanguage: string;
  translationDirection: "left" | "right";
  canSwitchLanguage: boolean;
};

const statusLabels = {
  empty: "暂无结果",
  placeholder: "待处理",
  loading: "正在处理",
  success: "翻译结果",
  error: "处理失败",
} as const;

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

const languageSelectOptions = languageOptions.map((option) => ({
  id: option.value,
  label: option.label,
}));

export function ResultWindow() {
  const content = useResultStore((state) => state.content);
  const detail = useResultStore((state) => state.detail);
  const status = useResultStore((state) => state.status);
  const sourceLanguage = useResultStore((state) => state.sourceLanguage);
  const targetLanguage = useResultStore((state) => state.targetLanguage);
  const direction = useResultStore((state) => state.direction);
  const pinned = useResultStore((state) => state.pinned);
  const setContent = useResultStore((state) => state.setContent);
  const setStatus = useResultStore((state) => state.setStatus);
  const setLanguages = useResultStore((state) => state.setLanguages);
  const setPinned = useResultStore((state) => state.setPinned);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen<ResultSnapshotPayload>("result-ready", (event) => {
      const payload = event.payload;
      setContent(payload.content, payload.detail);
      setStatus(payload.status);
      setLanguages(
        payload.sourceLanguage,
        payload.targetLanguage,
        payload.translationDirection,
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
  }, [setContent, setLanguages, setPinned, setStatus]);

  const startDrag = (event: ReactPointerEvent<HTMLElement>) => {
    const target = event.target as HTMLElement;
    if (target.closest("button, select, textarea, input, a")) {
      return;
    }
    void getCurrentWindow().startDragging();
  };

  const copy = async () => {
    try {
      await copyResultContent();
    } catch (error) {
      console.error("copy result failed", error);
    }
  };

  const close = () => {
    void getCurrentWindow().hide();
  };

  const togglePinned = () => {
    const next = !pinned;
    setPinned(next);
    void getCurrentWindow().setAlwaysOnTop(next);
  };

  const changeLanguages = async (
    nextSource: string,
    nextTarget: string,
    nextDirection: "left" | "right",
  ) => {
    if (busy) {
      return;
    }
    setLanguages(nextSource, nextTarget, nextDirection);
    setStatus("loading");
    setBusy(true);
    try {
      await requestResultTranslation({
        sourceLanguage: nextSource,
        targetLanguage: nextTarget,
        direction: nextDirection,
      });
    } catch (error) {
      setStatus("error");
      setContent("", typeof error === "string" ? error : "切换语言失败");
    } finally {
      setBusy(false);
    }
  };

  return (
    <main
      className="popup-window"
      style={{
        display: "flex",
        flexDirection: "column",
        gap: 10,
        width: "100vw",
        height: "100vh",
        padding: 12,
        boxSizing: "border-box",
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
            fontSize: 14,
            fontWeight: 700,
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
          }}
        >
          {statusLabels[status]}
        </div>
        <button
          className="popup-button popup-icon-button"
          type="button"
          onClick={togglePinned}
          aria-label={pinned ? "取消固定" : "固定"}
          title={pinned ? "取消固定" : "固定"}
        >
          {pinned ? <PinOff size={15} /> : <Pin size={15} />}
        </button>
        <button
          className="popup-button popup-icon-button"
          type="button"
          onClick={close}
          aria-label="关闭"
          title="关闭"
        >
          <X size={15} />
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
        <GlassSelect
          value={sourceLanguage}
          options={languageSelectOptions}
          onChange={(value) =>
            void changeLanguages(value, targetLanguage, direction)
          }
          ariaLabel="源语言"
          className="popup-language-select"
        />
        <button
          className="popup-button"
          type="button"
          onClick={() =>
            void changeLanguages(
              sourceLanguage,
              targetLanguage,
              direction === "left" ? "right" : "left",
            )
          }
          aria-label="切换翻译方向"
        >
          {direction === "left" ? "←" : "→"}
        </button>
        <GlassSelect
          value={targetLanguage}
          options={languageSelectOptions}
          onChange={(value) =>
            void changeLanguages(sourceLanguage, value, direction)
          }
          ariaLabel="目标语言"
          className="popup-language-select"
        />
      </div>
      <div
        className="popup-result-box"
        style={{
          flex: 1,
          minHeight: 0,
          padding: 12,
          overflow: "auto",
          whiteSpace: "pre-wrap",
        }}
      >
        {status === "loading"
          ? "正在处理..."
          : content || detail || "暂无结果"}
      </div>
      <footer
        className="popup-footer"
        style={{
          display: "flex",
          justifyContent: "flex-end",
          gap: 8,
          alignItems: "center",
        }}
      >
        <ActionButton
          type="button"
          onClick={copy}
          disabled={status !== "success"}
          style={actionButtonStyle}
        >
          复制
        </ActionButton>
      </footer>
    </main>
  );
}

const actionButtonStyle: CSSProperties = {
  flex: "0 0 auto",
  minHeight: 30,
  padding: "0 12px",
  cursor: "pointer",
};
