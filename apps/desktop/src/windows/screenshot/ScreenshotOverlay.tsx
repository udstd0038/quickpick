import { useEffect, useRef, useState, type PointerEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

type Point = { x: number; y: number };
type Rect = { left: number; top: number; width: number; height: number };

function normalizeRect(start: Point, end: Point): Rect {
  const left = Math.min(start.x, end.x);
  const top = Math.min(start.y, end.y);
  return {
    left,
    top,
    width: Math.abs(end.x - start.x),
    height: Math.abs(end.y - start.y),
  };
}

export function ScreenshotOverlay() {
  const [start, setStart] = useState<Point | null>(null);
  const [current, setCurrent] = useState<Point | null>(null);
  const [busy, setBusy] = useState(false);
  const overlayRef = useRef<HTMLDivElement | null>(null);
  const activeRect =
    start && current ? normalizeRect(start, current) : null;

  const capture = async (action: string) => {
    if (!activeRect || activeRect.width < 8 || activeRect.height < 8 || busy) {
      return;
    }
    setBusy(true);
    try {
      await invoke("capture_region_rect", {
        screenX: Math.round(activeRect.left),
        screenY: Math.round(activeRect.top),
        width: Math.round(activeRect.width),
        height: Math.round(activeRect.height),
        action,
      });
      await getCurrentWindow().hide();
    } finally {
      setBusy(false);
    }
  };

  const onPointerDown = (event: PointerEvent<HTMLDivElement>) => {
    const point = { x: event.clientX, y: event.clientY };
    setStart(point);
    setCurrent(point);
    event.currentTarget.setPointerCapture(event.pointerId);
  };

  const onPointerMove = (event: PointerEvent<HTMLDivElement>) => {
    if (!start) {
      return;
    }
    setCurrent({ x: event.clientX, y: event.clientY });
  };

  const onPointerUp = () => {
    setStart(null);
    setCurrent(null);
  };

  const closeOverlay = () => {
    setStart(null);
    setCurrent(null);
    void getCurrentWindow().hide();
  };

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

  return (
    <div
      className="screenshot-overlay"
      ref={overlayRef}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
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
        background: "rgba(0, 0, 0, 0.28)",
        userSelect: "none",
        touchAction: "none",
      }}
    >
      {activeRect && (
        <div
          className={`screenshot-frame ${
            activeRect.width >= 8 && activeRect.height >= 8
              ? "screenshot-frame-ready"
              : ""
          }`}
          style={{
            position: "absolute",
            left: activeRect.left,
            top: activeRect.top,
            width: activeRect.width,
            height: activeRect.height,
            pointerEvents: "none",
          }}
        />
      )}
      {activeRect && activeRect.width >= 8 && activeRect.height >= 8 && (
        <div
          style={{
            position: "absolute",
            left: Math.min(activeRect.left, window.innerWidth - 300),
            top: Math.min(activeRect.top + activeRect.height + 8, window.innerHeight - 48),
            display: "flex",
            gap: 8,
            padding: 6,
            background: "rgba(20, 25, 32, 0.96)",
            borderRadius: 8,
          }}
        >
          {["复制", "提取", "翻译"].map((label, index) => (
            <button
              key={label}
              type="button"
              disabled={busy}
              onClick={() => capture(index === 0 ? "copy" : index === 1 ? "extract" : "translate")}
              style={{
                minHeight: 32,
                padding: "0 12px",
                border: "1px solid rgba(164, 180, 202, 0.24)",
                borderRadius: 6,
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
