import { useEffect, useRef, useState } from "react";
import ReactCrop, { type PercentCrop, type PixelCrop } from "react-image-crop";
import "react-image-crop/dist/ReactCrop.css";
import { listen } from "@tauri-apps/api/event";
import {
  cancelScreenshot,
  captureRegionRect,
  getScreenshotSnapshot,
  type MonitorScreenshotPayload,
  type ScreenshotClosedPayload,
  type ScreenshotErrorPayload,
} from "../../services/invoke";
import { normalizeScreenshotCrop } from "../../lib/screenshotCoordinates";
import { ActionButton } from "../../components/ActionButton";
import { SkeletonScreen } from "../../components/LoadingSkeleton";
import { useI18n } from "../../lib/i18n";

const SCREENSHOT_TOOLBAR_WIDTH = 264;
const SCREENSHOT_TOOLBAR_HEIGHT = 48;
const MIN_REGION_SIZE = 8;
type ScreenshotAction = "copy" | "extract" | "translate";
type PendingOperation = { sessionId: number; kind: "capture" | "cancel" };

function validSessionId(value: number) {
  return Number.isSafeInteger(value) && value > 0;
}

export function ScreenshotOverlay() {
  const t = useI18n();
  const [screenshot, setScreenshot] = useState<MonitorScreenshotPayload | null>(null);
  const [crop, setCrop] = useState<PercentCrop | undefined>(undefined);
  const [pixelCrop, setPixelCrop] = useState<PixelCrop | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const imageRef = useRef<HTMLImageElement>(null);
  const screenshotRef = useRef<MonitorScreenshotPayload | null>(null);
  const currentSessionRef = useRef<number | null>(null);
  const latestSessionRef = useRef(0);
  const retiredSessionRef = useRef(0);
  const revisionRef = useRef(0);
  const pendingRef = useRef<PendingOperation | null>(null);
  const mountedRef = useRef(false);
  const renderedSessionId = screenshot?.sessionId;

  const errorMessage = (cause: unknown) => {
    if (typeof cause === "string") return t(cause);
    if (cause instanceof Error && cause.message) return t(cause.message);
    return t("selection.readFailed");
  };

  const releaseImage = () => {
    screenshotRef.current = null;
    setScreenshot(null);
    setCrop(undefined);
    setPixelCrop(null);
  };

  const retireSession = (sessionId: number) => {
    retiredSessionRef.current = Math.max(retiredSessionRef.current, sessionId);
    if (currentSessionRef.current === sessionId) currentSessionRef.current = null;
    releaseImage();
  };

  useEffect(() => {
    let disposed = false;
    const cleanups = new Set<() => void>();
    mountedRef.current = true;
    const startupRevision = revisionRef.current;

    const acceptScreenshot = (payload: MonitorScreenshotPayload) => {
      if (
        disposed ||
        !validSessionId(payload.sessionId) ||
        payload.sessionId < latestSessionRef.current ||
        payload.sessionId <= retiredSessionRef.current ||
        (currentSessionRef.current === payload.sessionId && screenshotRef.current)
      ) {
        return;
      }
      latestSessionRef.current = payload.sessionId;
      currentSessionRef.current = payload.sessionId;
      screenshotRef.current = payload;
      revisionRef.current += 1;
      pendingRef.current = null;
      setBusy(false);
      setError("");
      setCrop(undefined);
      setPixelCrop(null);
      setScreenshot(payload);
    };

    const ready = listen<MonitorScreenshotPayload>("screenshot-ready", ({ payload }) => {
      acceptScreenshot(payload);
    });
    const failed = listen<ScreenshotErrorPayload>("screenshot-error", ({ payload }) => {
      if (
        disposed || !validSessionId(payload.sessionId) ||
        payload.sessionId < latestSessionRef.current
      ) return;
      latestSessionRef.current = payload.sessionId;
      revisionRef.current += 1;
      currentSessionRef.current = null;
      retiredSessionRef.current = Math.max(retiredSessionRef.current, payload.sessionId);
      pendingRef.current = null;
      releaseImage();
      setBusy(false);
      setError(t(payload.message));
    });
    const closed = listen<ScreenshotClosedPayload>("screenshot-closed", ({ payload }) => {
      if (
        disposed || !validSessionId(payload.sessionId) ||
        payload.sessionId < latestSessionRef.current
      ) return;
      latestSessionRef.current = payload.sessionId;
      revisionRef.current += 1;
      retiredSessionRef.current = Math.max(retiredSessionRef.current, payload.sessionId);
      currentSessionRef.current = null;
      releaseImage();
      setBusy(false);
      if (pendingRef.current && pendingRef.current.sessionId < payload.sessionId) {
        pendingRef.current = null;
      }
    });

    const subscriptions = [ready, failed, closed].map(async (subscription) => {
      const cleanup = await subscription;
      if (disposed) cleanup();
      else cleanups.add(cleanup);
    });
    // Install listeners before querying, so no new-session event can fall
    // between the initial request and listener registration.
    void Promise.all(subscriptions)
      .then(async () => {
        if (disposed || revisionRef.current !== startupRevision) return;
        const requestRevision = revisionRef.current;
        try {
          const payload = await getScreenshotSnapshot();
          if (!disposed && revisionRef.current === requestRevision && payload) {
            acceptScreenshot(payload);
          }
        } catch (cause) {
          if (!disposed && revisionRef.current === requestRevision) {
            releaseImage();
            setError(errorMessage(cause));
          }
        }
      })
      .catch((cause) => {
        if (!disposed && revisionRef.current === startupRevision) {
          setError(errorMessage(cause));
        }
      });

    return () => {
      disposed = true;
      mountedRef.current = false;
      revisionRef.current += 1;
      screenshotRef.current = null;
      currentSessionRef.current = null;
      pendingRef.current = null;
      for (const cleanup of cleanups) cleanup();
      cleanups.clear();
    };
  }, []);

  const finishOperation = (operation: PendingOperation, failure: string) => {
    if (
      !mountedRef.current || pendingRef.current !== operation ||
      latestSessionRef.current !== operation.sessionId
    ) return;
    pendingRef.current = null;
    retireSession(operation.sessionId);
    setBusy(false);
    setError(failure);
  };

  const closeOverlay = async () => {
    let sessionId = currentSessionRef.current;
    if (sessionId === null) {
      const revision = ++revisionRef.current;
      pendingRef.current = null;
      releaseImage();
      setBusy(true);
      setError("");
      try {
        // A window hide has no session identity. Query only the ID and cancel
        // through the backend, which can refuse a stale request safely.
        const queriedSessionId = await getScreenshotSnapshot().then((payload) => payload?.sessionId ?? null);
        if (
          !mountedRef.current || revisionRef.current !== revision ||
          currentSessionRef.current !== null
        ) return;
        if (
          queriedSessionId === null || !validSessionId(queriedSessionId) ||
          queriedSessionId < latestSessionRef.current ||
          queriedSessionId <= retiredSessionRef.current
        ) return;
        sessionId = queriedSessionId;
        latestSessionRef.current = queriedSessionId;
        currentSessionRef.current = queriedSessionId;
      } catch (cause) {
        if (
          mountedRef.current && revisionRef.current === revision &&
          currentSessionRef.current === null
        ) setError(errorMessage(cause));
        return;
      } finally {
        if (
          mountedRef.current && revisionRef.current === revision &&
          currentSessionRef.current === null
        ) setBusy(false);
      }
    }
    if (pendingRef.current?.kind === "cancel") return;
    const operation: PendingOperation = { sessionId, kind: "cancel" };
    pendingRef.current = operation;
    setBusy(true);
    setError("");
    let failure = "";
    try {
      await cancelScreenshot(sessionId);
    } catch (cause) {
      failure = errorMessage(cause);
    } finally {
      finishOperation(operation, failure);
    }
  };

  useEffect(() => {
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") void closeOverlay();
    };
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, []);

  const viewportSize = () => {
    const bounds = imageRef.current?.getBoundingClientRect();
    return {
      width: bounds?.width || window.innerWidth,
      height: bounds?.height || window.innerHeight,
    };
  };

  const capture = async (action: ScreenshotAction) => {
    if (
      pendingRef.current || !pixelCrop || !screenshotRef.current ||
      renderedSessionId !== currentSessionRef.current ||
      renderedSessionId !== screenshotRef.current.sessionId
    ) return;
    // Only scalar metadata crosses the await; do not retain a full PNG in an
    // in-flight AI request after the native closed event releases the image.
    const sessionId = screenshotRef.current.sessionId;
    const region = normalizeScreenshotCrop(pixelCrop, viewportSize(), {
      width: screenshotRef.current.width,
      height: screenshotRef.current.height,
    });
    if (!region) return;
    const operation: PendingOperation = { sessionId, kind: "capture" };
    pendingRef.current = operation;
    setBusy(true);
    setError("");
    let failure = "";
    try {
      await captureRegionRect({ sessionId, ...region, action });
    } catch (cause) {
      failure = errorMessage(cause);
    } finally {
      finishOperation(operation, failure);
    }
  };

  const validCrop = screenshot && pixelCrop
    ? normalizeScreenshotCrop(pixelCrop, viewportSize(), {
      width: screenshot.width,
      height: screenshot.height,
    })
    : null;

  return (
    <div
      className="screenshot-overlay"
      onContextMenu={(event) => {
        event.preventDefault();
        void closeOverlay();
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
      {screenshot ? (
        <ReactCrop
          crop={crop}
          disabled={busy}
          onChange={(_pixel, percent) => {
            if (currentSessionRef.current === renderedSessionId) setCrop(percent);
          }}
          onComplete={(pixel, percent) => {
            if (currentSessionRef.current === renderedSessionId) {
              setCrop(percent);
              setPixelCrop(pixel);
            }
          }}
          onDragStart={() => {
            if (currentSessionRef.current === renderedSessionId) setPixelCrop(null);
          }}
          keepSelection
          minWidth={MIN_REGION_SIZE * window.innerWidth / screenshot.width}
          minHeight={MIN_REGION_SIZE * window.innerHeight / screenshot.height}
          ruleOfThirds
          className="screenshot-crop"
          style={{ width: "100vw", height: "100vh", display: "block" }}
        >
          <img
            ref={imageRef}
            src={screenshot.pngDataUrl}
            alt={t("screenshot.currentScreen")}
            onError={() => {
              if (currentSessionRef.current === renderedSessionId) {
                releaseImage();
                setError(t("selection.readFailed"));
              }
            }}
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
      ) : error ? (
        <div
          role="alert"
          style={{ display: "grid", minHeight: "100vh", placeItems: "center", color: "var(--qp-text-primary)" }}
        >
          {error}
        </div>
      ) : (
        <SkeletonScreen className="screenshot-skeleton" count={8} testId="screenshot-loading-skeleton" />
      )}
      {validCrop && pixelCrop && (
        <div
          className="screenshot-toolbar"
          style={{
            position: "absolute",
            left: Math.min(pixelCrop.x + pixelCrop.width + 8, Math.max(8, window.innerWidth - SCREENSHOT_TOOLBAR_WIDTH - 8)),
            top: Math.min(pixelCrop.y + pixelCrop.height + 8, Math.max(8, window.innerHeight - SCREENSHOT_TOOLBAR_HEIGHT - 8)),
            display: "flex",
            gap: 8,
            padding: 8,
            zIndex: 20,
          }}
        >
          {([
            { key: "screenshot.copy", action: "copy" },
            { key: "screenshot.extract", action: "extract" },
            { key: "screenshot.translate", action: "translate" },
          ] as const).map(({ key, action }) => (
            <ActionButton
              key={key}
              className="screenshot-tool-button"
              type="button"
              disabled={busy}
              onClick={() => void capture(action)}
              style={{ minHeight: 32, padding: "0 14px" }}
            >
              {t(key)}
            </ActionButton>
          ))}
        </div>
      )}
    </div>
  );
}
