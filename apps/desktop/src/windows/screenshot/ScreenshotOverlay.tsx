import { useEffect, useRef, useState } from "react";
import ReactCrop, {
  type PercentCrop,
  type PixelCrop,
} from "react-image-crop";
import "react-image-crop/dist/ReactCrop.css";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  captureRegionRect,
  clearPopupState,
  getScreenshotSnapshot,
  type MonitorScreenshotPayload,
} from "../../services/invoke";
import { ActionButton } from "../../components/ActionButton";
import { SkeletonScreen } from "../../components/LoadingSkeleton";
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
  const snapshotRevision = useRef(0);

  useEffect(() => {
    let disposed = false;
    const initialRevision = snapshotRevision.current;

    let unlistenReady: (() => void) | undefined;
    let unlistenError: (() => void) | undefined;

    const applyScreenshot = (payload: MonitorScreenshotPayload) => {
      setBusy(false);
      setError("");
      setCrop(undefined);
      setPixelCrop(null);
      setScreenshot(payload);
    };
    const applyError = (message: string) => {
      setBusy(false);
      setCrop(undefined);
      setPixelCrop(null);
      setScreenshot(null);
      setError(message);
    };

    const ready = listen<MonitorScreenshotPayload>("screenshot-ready", (event) => {
      if (!disposed) {
        snapshotRevision.current += 1;
        applyScreenshot(event.payload);
      }
    }).then((cleanup) => {
      if (disposed) {
        cleanup();
      } else {
        unlistenReady = cleanup;
      }
    });

    const failed = listen<string>("screenshot-error", (event) => {
      if (!disposed) {
        snapshotRevision.current += 1;
        applyError(event.payload);
      }
    }).then((cleanup) => {
      if (disposed) {
        cleanup();
      } else {
        unlistenError = cleanup;
      }
    });

    // The WebView is reused. A newer activation or error always wins over the
    // initial snapshot, including events received during listener registration.
    void Promise.allSettled([ready, failed]).then(async () => {
      if (disposed || initialRevision !== snapshotRevision.current) {
        return;
      }
      try {
        const payload = await getScreenshotSnapshot();
        if (payload && !disposed && initialRevision === snapshotRevision.current) {
          applyScreenshot(payload);
        }
      } catch (error) {
        if (!disposed && initialRevision === snapshotRevision.current) {
          applyError(typeof error === "string" ? error : "result.error");
        }
      }
    });

    return () => {
      disposed = true;
      snapshotRevision.current += 1;
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
    snapshotRevision.current += 1;
    setBusy(false);
    setScreenshot(null);
    setCrop(undefined);
    setPixelCrop(null);
    setError("");
    void clearPopupState("screenshot_overlay");
    void getCurrentWindow().hide();
  };

  const capture = async (action: string) => {
    if (!screenshot || !pixelCrop || pixelCrop.width < 8 || pixelCrop.height < 8 || busy) {
      return;
    }

    const scaleX = screenshot.width / window.innerWidth;
    const scaleY = screenshot.height / window.innerHeight;
    const revision = ++snapshotRevision.current;
    setBusy(true);
    try {
      await getCurrentWindow().hide();
      if (revision !== snapshotRevision.current) {
        return;
      }
      setScreenshot(null);
      setError("");
      void clearPopupState("screenshot_overlay").catch(() => undefined);
      await captureRegionRect({
        screenX: screenshot.monitorX + Math.round(pixelCrop.x * scaleX),
        screenY: screenshot.monitorY + Math.round(pixelCrop.y * scaleY),
        width: Math.round(pixelCrop.width * scaleX),
        height: Math.round(pixelCrop.height * scaleY),
        action,
      });
    } catch (error) {
      if (revision === snapshotRevision.current) {
        setScreenshot(null);
        setError(typeof error === "string" ? error : "result.error");
      }
    } finally {
      if (revision === snapshotRevision.current) {
        setBusy(false);
        setCrop(undefined);
        setPixelCrop(null);
      }
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
          role="alert"
          style={{
            display: "grid",
            minHeight: "100vh",
            placeItems: "center",
            alignContent: "center",
            gap: 16,
            color: "var(--qp-text-primary)",
          }}
        >
          {t(error)}
          <ActionButton type="button" onClick={closeOverlay}>
            {t("common.close")}
          </ActionButton>
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
        <SkeletonScreen
          className="screenshot-skeleton"
          count={8}
          testId="screenshot-loading-skeleton"
        />
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
