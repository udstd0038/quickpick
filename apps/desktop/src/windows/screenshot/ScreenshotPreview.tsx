import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

type ScreenshotPreviewPayload = {
  screenX: number;
  screenY: number;
  width: number;
  height: number;
  status: string;
};

export function ScreenshotPreview() {
  const [preview, setPreview] = useState<ScreenshotPreviewPayload | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen<ScreenshotPreviewPayload>("screenshot-preview", (event) => {
      setPreview(event.payload);
    }).then((cleanup) => {
      unlisten = cleanup;
    });
    return () => {
      unlisten?.();
    };
  }, []);

  return (
    <main
      style={{
        width: "100vw",
        height: "100vh",
        padding: 14,
        boxSizing: "border-box",
        color: "var(--qp-text-primary)",
        background: "var(--qp-shell-bg)",
        fontFamily: "'Segoe UI', 'Microsoft YaHei', sans-serif",
      }}
    >
      {preview ? (
        <div>
          <div>{preview.status}</div>
          <div>
            {preview.width} × {preview.height}
          </div>
          <div>
            ({preview.screenX}, {preview.screenY})
          </div>
        </div>
      ) : (
        "等待截图区域"
      )}
    </main>
  );
}
