import { invoke } from "@tauri-apps/api/core";

export type MacPermissionKind =
  | "accessibility"
  | "screen_recording"
  | "input_monitoring";

export type MacPermissionStatus = {
  accessibility: boolean;
  screenRecording: boolean;
  inputMonitoring: boolean;
};

export async function getMacosPermissionStatus(): Promise<MacPermissionStatus> {
  const status = await invoke<MacPermissionStatus>("get_macos_permission_status");
  if (
    !status ||
    typeof status.accessibility !== "boolean" ||
    typeof status.screenRecording !== "boolean" ||
    typeof status.inputMonitoring !== "boolean"
  ) {
    throw new Error("Invalid macOS permission status");
  }
  return status;
}

export function requestMacosPermission(kind: MacPermissionKind) {
  return invoke<MacPermissionStatus>("request_macos_permission", { kind });
}

export function openMacosPermissionSettings(kind: MacPermissionKind) {
  return invoke<void>("open_macos_permission_settings", { kind });
}
