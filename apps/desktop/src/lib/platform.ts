export const runningOnMac =
  typeof navigator !== "undefined" &&
  /Mac|iPhone|iPad|iPod/i.test(navigator.platform || navigator.userAgent || "");

export const runningPlatform = runningOnMac ? "macos" : "windows";

export function setPlatformDataset(root: HTMLElement = document.documentElement) {
  root.dataset.platform = runningPlatform;
}
