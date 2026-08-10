import { useEffect, useState } from "react";
import ReactCrop, {
  type PercentCrop,
  type PixelCrop,
} from "react-image-crop";
import "react-image-crop/dist/ReactCrop.css";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  captureRegionRect,
  type MonitorScreenshotPayload,
} from "../../services/invoke";
import { ActionButton } from "../../components/ActionButton";
import { useI18n } from "../../lib/i18n";

const SCREENSHOT_TOOLBAR_WIDTH = 264;
const SCREENSHOT_TOOLBAR_HEIGHT = 48;

export function ScreenshotOverlay() {
  const t = useI18n();
  const [screenshot, setScreenshot] = useState<MonitorScreenshotPayload | null>(
    null,
  );
  const [crop, setCrop] = useState<PercentCrop | undefined>(undefined);
  const [pixelCrop, setPixelCrop] = useState<PixelCrop | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    let disposed = false;

    let unlistenReady: (() => void) | undefined;
    let unlistenError: (() => void) | undefined;

    listen<MonitorScreenshotPayload>("screenshot-ready", (event) => {
      if (!disposed) {
        setError("");
        setCrop(undefined);
        setPixelCrop(null);
        setScreenshot(event.payload);
      }
    }).then((cleanup) => {
      unlistenReady = cleanup;
    });

    listen<string>("screenshot-error", (event) => {
      if (!disposed) {
        setCrop(undefined);
        setPixelCrop(null);
        setScreenshot(null);
        setError(event.payload);
      }
    }).then((cleanup) => {
      unlistenError = cleanup;
    });

    return () => {
      disposed = true;
      unlistenReady?.();
      unlistenError?.();
    };
  }, []);

  useEffect(() => {
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        closeOverlay();
      }
    };
    window.addEventListener("keydown", closeOnEscape);
    return () => {
      window.removeEventListener("keydown", closeOnEscape);
    };
  }, []);

  const closeOverlay = () => {
    setCrop(undefined);
    setPixelCrop(null);
    void getCurrentWindow().hide();
  };

  const capture = async (action: string) => {
    if (!screenshot || !pixelCrop || pixelCrop.width < 8 || pixelCrop.height < 8 || busy) {
      return;
    }

    const scaleX = screenshot.width / window.innerWidth;
    const scaleY = screenshot.height / window.innerHeight;
    setBusy(true);
    await getCurrentWindow().hide();
    try {
      await captureRegionRect({
        screenX: screenshot.monitorX + Math.round(pixelCrop.x * scaleX),
        screenY: screenshot.monitorY + Math.round(pixelCrop.y * scaleY),
        width: Math.round(pixelCrop.width * scaleX),
        height: Math.round(pixelCrop.height * scaleY),
        action,
      });
    } finally {
      setBusy(false);
      setCrop(undefined);
      setPixelCrop(null);
    }
  };

  return (
    <div
      className="screenshot-overlay"
      onContextMenu={(event) => {
        event.preventDefault();
        closeOverlay();
      }}
      style={{
        position: "relative",
        width: "100vw",
        height: "100vh",
        overflow: "hidden",
        cursor: "crosshair",
        background: "var(--qp-shell-bg)",
        userSelect: "none",
        touchAction: "none",
      }}
    >
      {error ? (
        <div
          style={{
            display: "grid",
            minHeight: "100vh",
            placeItems: "center",
            color: "var(--qp-text-primary)",
          }}
        >
          {error}
        </div>
      ) : screenshot ? (
        <ReactCrop
          crop={crop}
          onChange={(_pixel, percent) => {
            setCrop(percent);
          }}
          onComplete={(pixel, percent) => {
            setCrop(percent);
            setPixelCrop(pixel);
          }}
          onDragStart={() => {
            setPixelCrop(null);
          }}
          keepSelection
          minWidth={8}
          minHeight={8}
          ruleOfThirds
          className="screenshot-crop"
          style={{
            width: "100vw",
            height: "100vh",
            display: "block",
          }}
        >
          <img
            src={screenshot.pngDataUrl}
            alt={t("screenshot.currentScreen")}
            draggable={false}
            style={{
              display: "block",
              width: "100vw",
              height: "100vh",
              objectFit: "fill",
              userSelect: "none",
            }}
          />
        </ReactCrop>
      ) : (
        <div
          style={{
            display: "grid",
            minHeight: "100vh",
            placeItems: "center",
            color: "var(--qp-text-primary)",
          }}
        >
          {t("screenshot.reading")}
        </div>
      )}

      {pixelCrop && pixelCrop.width >= 8 && pixelCrop.height >= 8 && (
        <div
          className="screenshot-toolbar"
          style={{
            position: "absolute",
            left: Math.min(
              pixelCrop.x + pixelCrop.width + 8,
              Math.max(8, window.innerWidth - SCREENSHOT_TOOLBAR_WIDTH - 8),
            ),
            top: Math.min(
              pixelCrop.y + pixelCrop.height + 8,
              Math.max(8, window.innerHeight - SCREENSHOT_TOOLBAR_HEIGHT - 8),
            ),
            display: "flex",
            gap: 8,
            padding: 8,
            zIndex: 20,
          }}
        >
          {[
            { key: "screenshot.copy", action: "copy" },
            { key: "screenshot.extract", action: "extract" },
            { key: "screenshot.translate", action: "translate" },
          ].map(({ key, action }) => (
            <ActionButton
              key={key}
              className="screenshot-tool-button"
              type="button"
              disabled={busy}
              onClick={() =>
                void capture(action)
              }
              style={{
                minHeight: 32,
                padding: "0 14px",
              }}
            >
              {t(key)}
            </ActionButton>
          ))}
        </div>
      )}
    </div>
  );
}
