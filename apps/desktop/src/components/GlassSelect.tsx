import {
  useEffect,
  useRef,
  useState,
  type CSSProperties,
} from "react";
import { createPortal } from "react-dom";

export type GlassSelectOption<T extends string> = {
  id: T;
  label: string;
};

export function GlassSelect<T extends string>({
  value,
  options,
  onChange,
  ariaLabel,
  className = "",
}: {
  value: T;
  options: readonly GlassSelectOption<T>[];
  onChange: (value: T) => void;
  ariaLabel: string;
  className?: string;
}) {
  const [open, setOpen] = useState(false);
  const [menuStyle, setMenuStyle] = useState<CSSProperties>({});
  const triggerRef = useRef<HTMLButtonElement | null>(null);
  const menuRef = useRef<HTMLDivElement | null>(null);
  const selected = options.find((option) => option.id === value) || options[0];

  const selectOption = (nextValue: T) => {
    onChange(nextValue);
    setOpen(false);
  };

  const updateMenuPosition = () => {
    const trigger = triggerRef.current;
    if (!trigger) {
      return;
    }

    const rect = trigger.getBoundingClientRect();
    setMenuStyle({
      left: rect.left,
      top: rect.bottom + 6,
      width: rect.width,
    });
  };

  useEffect(() => {
    if (!open) {
      return;
    }

    updateMenuPosition();
    const closeOnOutsideMouseDown = (event: MouseEvent) => {
      const target = event.target;
      if (!(target instanceof Node)) {
        return;
      }
      if (
        triggerRef.current?.contains(target) ||
        menuRef.current?.contains(target)
      ) {
        return;
      }
      setOpen(false);
    };
    const refreshPosition = () => updateMenuPosition();

    document.addEventListener("mousedown", closeOnOutsideMouseDown, true);
    window.addEventListener("resize", refreshPosition);
    window.addEventListener("scroll", refreshPosition, true);

    return () => {
      document.removeEventListener("mousedown", closeOnOutsideMouseDown, true);
      window.removeEventListener("resize", refreshPosition);
      window.removeEventListener("scroll", refreshPosition, true);
    };
  }, [open]);

  return (
    <div
      className={`glass-select glass-select-popup ${className} ${
        open ? "glass-select-open" : ""
      }`.trim()}
    >
      <button
        ref={triggerRef}
        className="glass-select-trigger setting-input"
        type="button"
        aria-label={ariaLabel}
        aria-haspopup="listbox"
        aria-expanded={open}
        onClick={() => setOpen((current) => !current)}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            setOpen(false);
          }
        }}
      >
        <span>{selected.label}</span>
        <span className="glass-select-arrow" aria-hidden="true" />
      </button>
      {open &&
        createPortal(
          <div
            className="glass-select-menu"
            role="listbox"
            ref={menuRef}
            style={menuStyle}
          >
            {options.map((option) => (
              <button
                className={`glass-select-option ${
                  option.id === value ? "glass-select-option-active" : ""
                }`}
                type="button"
                role="option"
                aria-selected={option.id === value}
                key={option.id}
                onMouseDown={(event) => event.preventDefault()}
                onClick={() => selectOption(option.id)}
              >
                {option.label}
              </button>
            ))}
          </div>,
          document.body,
        )}
    </div>
  );
}
