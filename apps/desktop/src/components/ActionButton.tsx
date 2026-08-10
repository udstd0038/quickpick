import { useState, type ButtonHTMLAttributes } from "react";

type ActionButtonState = "idle" | "hover" | "active" | "focus";

export function ActionButton({
  className = "",
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement>) {
  const [state, setState] = useState<ActionButtonState>("idle");

  return (
    <button
      {...props}
      className={`popup-button popup-action-button ${className} ${
        state === "idle" ? "" : state
      }`.trim()}
      onPointerEnter={(event) => {
        setState((current) => (current === "active" ? current : "hover"));
        props.onPointerEnter?.(event);
      }}
      onPointerLeave={(event) => {
        setState((current) => (current === "focus" ? current : "idle"));
        props.onPointerLeave?.(event);
      }}
      onPointerDown={(event) => {
        if (event.button === 0) {
          setState("active");
        }
        props.onPointerDown?.(event);
      }}
      onPointerUp={(event) => {
        setState((current) => (current === "active" ? "hover" : current));
        props.onPointerUp?.(event);
      }}
      onPointerCancel={(event) => {
        setState("idle");
        props.onPointerCancel?.(event);
      }}
      onFocus={(event) => {
        setState((current) => (current === "active" ? current : "focus"));
        props.onFocus?.(event);
      }}
      onBlur={(event) => {
        setState("idle");
        props.onBlur?.(event);
      }}
    />
  );
}
