import {
  useEffect,
  useState,
  type CSSProperties,
  type KeyboardEvent as ReactKeyboardEvent,
  type PointerEvent as ReactPointerEvent,
} from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Pin, PinOff, X } from "lucide-react";
import {
  clearPopupState,
  copyInputResult,
  getInputSnapshot,
  requestInputTranslation,
  type InputReadyPayload,
} from "../../services/invoke";
import { GlassSelect } from "../../components/GlassSelect";
import { ActionButton } from "../../components/ActionButton";
import { useInputStore } from "../../stores/inputStore";
import { useI18n } from "../../lib/i18n";

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

export function InputWindow() {
  const t = useI18n();
  const languageSelectOptions = languageOptions.map((option) => ({
    id: option.value,
    label: t(option.key),
  }));
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

  const closeInput = () => {
    void clearPopupState("input");
    useInputStore.getState().reset();
    void getCurrentWindow().hide();
  };

  const languageLabel = (value: string) => {
    const option = languageOptions.find((item) => item.value === value);
    return option ? t(option.key) : value;
  };

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    getInputSnapshot()
      .then((payload) => {
        if (!payload) {
          return;
        }
        setResult(payload.content || payload.detail);
        setStatus(payload.status);
        setLanguages(
          payload.sourceLanguage,
          payload.targetLanguage,
          payload.direction,
        );
      })
      .catch(() => {
        // The Rust event path still covers normally-opened windows.
      });

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
        closeInput();
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
      await requestInputTranslation({
        inputText: text,
        sourceLanguage,
        targetLanguage,
        direction,
      });
    } catch (error) {
      setStatus("error");
      setResult(typeof error === "string" ? error : t("input.requestFailed"));
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

  const copy = async () => {
    try {
      await copyInputResult();
    } catch (error) {
      console.error("copy input result failed", error);
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
          {t("input.title")}
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
          onClick={closeInput}
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
          onChange={(value) => setLanguages(value, targetLanguage, direction)}
          ariaLabel={t("input.sourceLanguage")}
          className="popup-language-select"
        />
        <button
          className="popup-button"
          type="button"
          onClick={() =>
            setLanguages(
              sourceLanguage,
              targetLanguage,
              direction === "left" ? "right" : "left",
            )
          }
          aria-label={t("input.switchDirection")}
        >
          {direction === "left" ? "←" : "→"}
        </button>
        <GlassSelect
          value={targetLanguage}
          options={languageSelectOptions}
          onChange={(value) => setLanguages(sourceLanguage, value, direction)}
          ariaLabel={t("input.targetLanguage")}
          className="popup-language-select"
        />
      </div>

      <textarea
        value={text}
        onChange={(event) => setText(event.currentTarget.value)}
        onKeyDown={translateOnEnter}
        placeholder={t("input.placeholder")}
        autoFocus
        className="popup-textarea"
        style={{
          minHeight: 76,
          resize: "none",
          padding: 10,
          outline: "none",
        }}
      />

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
          ? t("input.processing")
          : result ||
            t("input.sourceTarget", {
              source: languageLabel(sourceLanguage),
              target: languageLabel(targetLanguage),
            })}
      </div>
      <div
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
          onClick={translate}
          disabled={!text.trim() || busy}
          style={actionButtonStyle}
        >
          {busy ? t("input.translating") : t("common.translate")}
        </ActionButton>
        <ActionButton
          type="button"
          onClick={copy}
          disabled={status !== "success"}
          style={actionButtonStyle}
        >
          {t("common.copy")}
        </ActionButton>
      </div>
    </main>
  );
}

const actionButtonStyle: CSSProperties = {
  flex: "0 0 auto",
  minHeight: 30,
  padding: "0 12px",
  cursor: "pointer",
};
