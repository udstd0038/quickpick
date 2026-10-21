import { useCallback, useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useI18n } from "../../lib/i18n";
import { runningOnMac } from "../../lib/platform";
import {
  getMacosPermissionStatus,
  openMacosPermissionSettings,
  requestMacosPermission,
  type MacPermissionKind,
  type MacPermissionStatus,
} from "../../services/macosPermissions";

const permissions = [
  {
    kind: "accessibility",
    field: "accessibility",
    label: "macPermissions.accessibility",
    description: "macPermissions.accessibilityDescription",
  },
  {
    kind: "screen_recording",
    field: "screenRecording",
    label: "macPermissions.screenRecording",
    description: "macPermissions.screenRecordingDescription",
  },
  {
    kind: "input_monitoring",
    field: "inputMonitoring",
    label: "macPermissions.inputMonitoring",
    description: "macPermissions.inputMonitoringDescription",
  },
] as const;

type Props = { clipboardFallbackEnabled: boolean };

export function MacPermissionsPanel(props: Props) {
  return runningOnMac ? <PermissionsPanel {...props} /> : null;
}

function PermissionsPanel({ clipboardFallbackEnabled }: Props) {
  const t = useI18n();
  const mounted = useRef(false);
  const checkSequence = useRef(0);
  const actionSequence = useRef(0);
  const actionPending = useRef(false);
  const [status, setStatus] = useState<MacPermissionStatus | null>(null);
  const [checking, setChecking] = useState(true);
  const [busy, setBusy] = useState(false);
  const [checkFailed, setCheckFailed] = useState(false);
  const [focusUnavailable, setFocusUnavailable] = useState(false);
  const [feedback, setFeedback] = useState<{
    kind: MacPermissionKind;
    key: string;
  } | null>(null);

  const refresh = useCallback(async () => {
    const sequence = ++checkSequence.current;
    setChecking(true);
    setCheckFailed(false);
    setStatus(null);
    try {
      const value = await getMacosPermissionStatus();
      if (mounted.current && sequence === checkSequence.current) {
        setStatus(value);
        setFeedback((current) => {
          const field = permissions.find((item) => item.kind === current?.kind)?.field;
          return field && value[field] ? null : current;
        });
        return value;
      }
    } catch {
      if (mounted.current && sequence === checkSequence.current) {
        setStatus(null);
        setCheckFailed(true);
        setFeedback(null);
      }
    } finally {
      if (mounted.current && sequence === checkSequence.current) {
        setChecking(false);
      }
    }
    return null;
  }, []);

  useEffect(() => {
    mounted.current = true;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void refresh();
    try {
      getCurrentWindow().onFocusChanged(({ payload }) => {
        if (payload && !disposed) {
          void refresh();
        }
      }).then((stop) => {
        if (disposed) {
          stop();
        } else {
          unlisten = stop;
        }
      }).catch(() => {
        if (!disposed) {
          setFocusUnavailable(true);
        }
      });
    } catch {
      setFocusUnavailable(true);
    }
    return () => {
      disposed = true;
      mounted.current = false;
      ++checkSequence.current;
      ++actionSequence.current;
      actionPending.current = false;
      unlisten?.();
    };
  }, [refresh]);

  const performAction = async (
    kind: MacPermissionKind,
    action: "request" | "settings",
  ) => {
    if (actionPending.current) {
      return;
    }
    const sequence = ++actionSequence.current;
    actionPending.current = true;
    setBusy(true);
    setFeedback(null);
    try {
      if (action === "settings") {
        await openMacosPermissionSettings(kind);
      } else {
        // A completed request is not proof of access. Only a fresh check is.
        await requestMacosPermission(kind);
        if (mounted.current && sequence === actionSequence.current) {
          const value = await refresh();
          const field = permissions.find((item) => item.kind === kind)!.field;
          if (
            mounted.current && sequence === actionSequence.current &&
            value && !value[field]
          ) {
            setFeedback({ kind, key: "macPermissions.notEffective" });
          }
        }
      }
    } catch {
      if (mounted.current && sequence === actionSequence.current) {
        setFeedback({
          kind,
          key: action === "settings"
            ? "macPermissions.openFailed"
            : "macPermissions.requestFailed",
        });
        if (action === "request") {
          await refresh();
        }
      }
    } finally {
      if (mounted.current && sequence === actionSequence.current) {
        actionPending.current = false;
        setBusy(false);
      }
    }
  };

  return (
    <div aria-label={t("macPermissions.title")}>
      <div className="setting-row">
        <span>{t("macPermissions.title")}</span>
        <button
          type="button"
          className="ghost-control"
          disabled={checking}
          onClick={() => void refresh()}
        >
          {t("macPermissions.recheck")}
        </button>
      </div>
      {permissions.filter((item) =>
        item.kind !== "input_monitoring" || clipboardFallbackEnabled,
      ).map((item) => (
        <div className="setting-field" key={item.kind}>
          <span>{t(item.label)}</span>
          <div className="setting-control">
            <strong>
              {checking
                ? t("macPermissions.checking")
                : status === null
                  ? t("macPermissions.unknown")
                  : t(status[item.field]
                    ? "macPermissions.granted"
                    : "macPermissions.denied")}
            </strong>
            <p className="setting-hint">{t(item.description)}</p>
            <div className="secret-control">
              <button
                type="button"
                className="ghost-control"
                aria-label={`${t("macPermissions.request")} ${t(item.label)}`}
                disabled={busy || checking || status?.[item.field] === true}
                onClick={() => void performAction(item.kind, "request")}
              >
                {t("macPermissions.request")}
              </button>
              <button
                type="button"
                className="ghost-control"
                aria-label={`${t("macPermissions.settings")} ${t(item.label)}`}
                disabled={busy}
                onClick={() => void performAction(item.kind, "settings")}
              >
                {t("macPermissions.settings")}
              </button>
            </div>
            {feedback?.kind === item.kind && (
              <p className="setting-hint" role="status">{t(feedback.key)}</p>
            )}
          </div>
        </div>
      ))}
      {checkFailed && (
        <p className="settings-feedback settings-feedback-error" role="alert">
          {t("macPermissions.checkFailed")}
        </p>
      )}
      <p className="setting-hint">{t("macPermissions.returnHint")}</p>
      {focusUnavailable && (
        <p className="setting-hint">{t("macPermissions.focusUnavailable")}</p>
      )}
    </div>
  );
}
