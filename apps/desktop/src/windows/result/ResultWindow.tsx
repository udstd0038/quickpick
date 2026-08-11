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
  clearPopupState,
  copyResultContent,
  getResultSnapshot,
  requestResultTranslation,
  type ResultSnapshotPayload,
} from "../../services/invoke";
import { GlassSelect } from "../../components/GlassSelect";
import { ActionButton } from "../../components/ActionButton";
import { useResultStore } from "../../stores/resultStore";
import { useI18n } from "../../lib/i18n";

const statusKeys = {
  empty: "result.empty",
  placeholder: "result.pending",
  loading: "result.loading",
  success: "result.success",
  error: "result.error",
} as const;

const languageOptions = [
  { value: "auto", key: "language.auto" },
  { value: "zh-Hans", key: "language.zhHans" },
  { value: "zh-Hant", key: "language.zhHant" },
  { value: "en", key: "language.en" },
  { value: "ja", key: "language.ja" },
  { value: "ko", key: "language.ko" },
  { value: "fr", key: "language.fr" },
  { value: "de", key: "language.de" },
  { value: "ru", key: "language.ru" },
  { value: "es", key: "language.es" },
] as const;

export function ResultWindow() {
  const t = useI18n();
  const languageSelectOptions = languageOptions.map((option) => ({
    id: option.value,
    label: t(option.key),
  }));
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

  const closeResult = () => {
    void clearPopupState("result");
    useResultStore.getState().reset();
    void getCurrentWindow().hide();
  };

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    getResultSnapshot()
      .then((payload) => {
        setContent(payload.content, payload.detail);
        setStatus(payload.status);
        setLanguages(
          payload.sourceLanguage,
          payload.targetLanguage,
          payload.translationDirection,
        );
      })
      .catch(() => {
        // The Rust event path still covers first paint for normally-opened windows.
      });

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
        closeResult();
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
      setContent("", typeof error === "string" ? error : t("result.switchFailed"));
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
          {t(statusKeys[status])}
        </div>
        <button
          className="popup-button popup-icon-button"
          type="button"
          onClick={togglePinned}
          aria-label={pinned ? t("common.unpin") : t("common.pin")}
          title={pinned ? t("common.unpin") : t("common.pin")}
        >
          {pinned ? <PinOff size={15} /> : <Pin size={15} />}
        </button>
        <button
          className="popup-button popup-icon-button"
          type="button"
          onClick={closeResult}
          aria-label={t("common.close")}
          title={t("common.close")}
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
          ariaLabel={t("result.sourceLanguage")}
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
          aria-label={t("result.switchDirection")}
        >
          {direction === "left" ? "←" : "→"}
        </button>
        <GlassSelect
          value={targetLanguage}
          options={languageSelectOptions}
          onChange={(value) =>
            void changeLanguages(sourceLanguage, value, direction)
          }
          ariaLabel={t("result.targetLanguage")}
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
          ? t("result.processing")
          : content || detail || t("result.noResult")}
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
          {t("result.copy")}
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
