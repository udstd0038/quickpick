import { useEffect, useState } from "react";
import ReactCrop, {
  type PercentCrop,
  type PixelCrop,
} from "react-image-crop";
import "react-image-crop/dist/ReactCrop.css";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  captureMonitorScreenshot,
  captureRegionRect,
  type MonitorScreenshotPayload,
} from "../../services/invoke";

export function ScreenshotOverlay() {
  const [screenshot, setScreenshot] = useState<MonitorScreenshotPayload | null>(
    null,
  );
  const [crop, setCrop] = useState<PercentCrop | undefined>(undefined);
  const [pixelCrop, setPixelCrop] = useState<PixelCrop | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    let disposed = false;

    captureMonitorScreenshot()
      .then((value) => {
        if (!disposed) {
          setScreenshot(value);
        }
      })
      .catch((reason) => {
        if (!disposed) {
          setError(typeof reason === "string" ? reason : "截图读取失败");
        }
      });

    return () => {
      disposed = true;
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
    try {
      await captureRegionRect({
        screenX: Math.round(pixelCrop.x * scaleX),
        screenY: Math.round(pixelCrop.y * scaleY),
        width: Math.round(pixelCrop.width * scaleX),
        height: Math.round(pixelCrop.height * scaleY),
        action,
      });
      await getCurrentWindow().hide();
    } finally {
      setBusy(false);
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
        background: "#0a1122",
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
            color: "#fff",
          }}
        >
          {error}
        </div>
      ) : screenshot ? (
        <ReactCrop
          crop={crop}
          onChange={(pixel, percent) => {
            setCrop(percent);
            setPixelCrop(pixel);
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
            alt="当前屏幕"
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
            color: "#fff",
          }}
        >
          正在读取屏幕截图...
        </div>
      )}

      {pixelCrop && pixelCrop.width >= 8 && pixelCrop.height >= 8 && (
        <div
          style={{
            position: "absolute",
            left: Math.min(pixelCrop.x, window.innerWidth - 300),
            top: Math.min(pixelCrop.y + pixelCrop.height + 10, window.innerHeight - 52),
            display: "flex",
            gap: 8,
            padding: 8,
            border: "1px solid rgba(255, 255, 255, 0.3)",
            borderRadius: 8,
            background: "rgba(12, 20, 38, 0.92)",
            boxShadow: "0 18px 50px rgba(0, 0, 0, 0.35)",
            zIndex: 20,
          }}
        >
          {["复制", "提取", "翻译"].map((label, index) => (
            <button
              key={label}
              type="button"
              disabled={busy}
              onClick={() =>
                void capture(index === 0 ? "copy" : index === 1 ? "extract" : "translate")
              }
              style={{
                minHeight: 32,
                padding: "0 14px",
                border: "1px solid rgba(164, 180, 202, 0.28)",
                borderRadius: 7,
                color: "#e7ecf3",
                background: "rgba(38, 47, 58, 0.96)",
                cursor: "pointer",
              }}
            >
              {label}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
