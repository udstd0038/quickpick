$ErrorActionPreference = "Stop"

Add-Type @'
using System;
using System.Text;
using System.Runtime.InteropServices;
public static class QuickPickHotkeyVerifier {
  public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);
  [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
  [DllImport("user32.dll")] public static extern int GetClassName(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint processId);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
  [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr hWnd, uint msg, IntPtr wParam, IntPtr lParam);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
  [DllImport("kernel32.dll")] public static extern IntPtr GetConsoleWindow();
}
'@

$process = Get-Process -Name quickpick -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $process) {
  throw "QuickPick is not running"
}

$pidValue = [uint32]$process.Id
$hotkeyHwnd = [IntPtr]::Zero

[QuickPickHotkeyVerifier]::EnumWindows({
  param($h, $l)
  [uint32]$wpid = 0
  [QuickPickHotkeyVerifier]::GetWindowThreadProcessId($h, [ref]$wpid) | Out-Null
  if ($wpid -eq $pidValue) {
    $class = [Text.StringBuilder]::new(256)
    [QuickPickHotkeyVerifier]::GetClassName($h, $class, 256) | Out-Null
    if ($class.ToString() -eq "global_hotkey_app") {
      $script:hotkeyHwnd = $h
    }
  }
  return $true
}, [IntPtr]::Zero) | Out-Null

if ($hotkeyHwnd -eq [IntPtr]::Zero) {
  throw "global_hotkey_app window was not found"
}

function Get-VisibleQuickPickTitles {
  $script:rows = [System.Collections.Generic.List[string]]::new()
  [QuickPickHotkeyVerifier]::EnumWindows({
    param($h, $l)
    [uint32]$wpid = 0
    [QuickPickHotkeyVerifier]::GetWindowThreadProcessId($h, [ref]$wpid) | Out-Null
    if ($wpid -eq $script:pidValue -and [QuickPickHotkeyVerifier]::IsWindowVisible($h)) {
      $title = [Text.StringBuilder]::new(512)
      [QuickPickHotkeyVerifier]::GetWindowText($h, $title, 512) | Out-Null
      $script:rows.Add($title.ToString())
    }
    return $true
  }, [IntPtr]::Zero) | Out-Null
  return $script:rows
}

function Hide-QuickPickPopups {
  [QuickPickHotkeyVerifier]::EnumWindows({
    param($h, $l)
    [uint32]$wpid = 0
    [QuickPickHotkeyVerifier]::GetWindowThreadProcessId($h, [ref]$wpid) | Out-Null
    if ($wpid -eq $script:pidValue) {
      $current = [Text.StringBuilder]::new(512)
      [QuickPickHotkeyVerifier]::GetWindowText($h, $current, 512) | Out-Null
      if ($current.ToString().StartsWith("QuickPick ")) {
        [QuickPickHotkeyVerifier]::ShowWindow($h, 0) | Out-Null
      }
    }
    return $true
  }, [IntPtr]::Zero) | Out-Null
}

# Alt+2 selection needs the app itself to be foreground so the WebView window can keep focus.
$selectionTitle = "QuickPick " + [char]0x5212 + [char]0x8BCD
$settingsTitle = "QuickPick " + [char]0x8BBE + [char]0x7F6E
$settingsHwnd = [IntPtr]::Zero

[QuickPickHotkeyVerifier]::EnumWindows({
  param($h, $l)
  [uint32]$wpid = 0
  [QuickPickHotkeyVerifier]::GetWindowThreadProcessId($h, [ref]$wpid) | Out-Null
  if ($wpid -eq $script:pidValue) {
    $title = [Text.StringBuilder]::new(512)
    [QuickPickHotkeyVerifier]::GetWindowText($h, $title, 512) | Out-Null
    if ($title.ToString() -eq $settingsTitle) {
      $script:settingsHwnd = $h
    }
  }
  return $true
}, [IntPtr]::Zero) | Out-Null

if ($settingsHwnd -ne [IntPtr]::Zero) {
  [QuickPickHotkeyVerifier]::ShowWindow($settingsHwnd, 5) | Out-Null
  [QuickPickHotkeyVerifier]::SetForegroundWindow($settingsHwnd) | Out-Null
  Start-Sleep -Milliseconds 300
}

Hide-QuickPickPopups
Start-Sleep -Milliseconds 200

# Alt+1 settings id: (Modifiers::ALT = 1) << 16 | Code::Digit1 = 6
[QuickPickHotkeyVerifier]::PostMessageW($hotkeyHwnd, 0x0312, [IntPtr]65542, [IntPtr]0x00310000) | Out-Null
Start-Sleep -Milliseconds 900
$titles = Get-VisibleQuickPickTitles
Write-Output ("Alt+1 visible titles: " + ($titles -join ", "))
if ($titles -notcontains $settingsTitle) {
  throw "Alt+1 event did not open the settings window"
}
Hide-QuickPickPopups

# Alt+2 selection id: (Modifiers::ALT = 1) << 16 | Code::Digit2 = 7
[QuickPickHotkeyVerifier]::PostMessageW($hotkeyHwnd, 0x0312, [IntPtr]65543, [IntPtr]0x00320000) | Out-Null
Start-Sleep -Milliseconds 900
$titles = Get-VisibleQuickPickTitles
Write-Output ("Alt+2 visible titles: " + ($titles -join ", "))
if ($titles -notcontains $selectionTitle) {
  throw "Alt+2 event did not open the selection window"
}
$consoleWindow = [QuickPickHotkeyVerifier]::GetConsoleWindow()
if ($consoleWindow -ne [IntPtr]::Zero) {
  [QuickPickHotkeyVerifier]::SetForegroundWindow($consoleWindow) | Out-Null
  Start-Sleep -Milliseconds 500
  $titles = Get-VisibleQuickPickTitles
  if ($titles -contains $selectionTitle) {
    throw "Alt+2 selection window did not hide after focus loss"
  }
}
Hide-QuickPickPopups

# Alt+4 input translate id: (Modifiers::ALT = 1) << 16 | Code::Digit4 = 9
[QuickPickHotkeyVerifier]::PostMessageW($hotkeyHwnd, 0x0312, [IntPtr]65545, [IntPtr]0x00340000) | Out-Null
Start-Sleep -Milliseconds 900
$titles = Get-VisibleQuickPickTitles
Write-Output ("Alt+4 visible titles: " + ($titles -join ", "))
if (@($titles | Where-Object { $_ -like "QuickPick *" }).Count -eq 0) {
  throw "Alt+4 event did not open the input translation window"
}
Hide-QuickPickPopups

# Alt+3 screenshot overlay id: (Modifiers::ALT = 1) << 16 | Code::Digit3 = 8
[QuickPickHotkeyVerifier]::PostMessageW($hotkeyHwnd, 0x0312, [IntPtr]65544, [IntPtr]0x00330000) | Out-Null
Start-Sleep -Milliseconds 900
$titles = Get-VisibleQuickPickTitles
Write-Output ("Alt+3 visible titles: " + ($titles -join ", "))
if (@($titles | Where-Object { $_ -like "QuickPick *" }).Count -eq 0) {
  throw "Alt+3 event did not open the screenshot overlay"
}
Hide-QuickPickPopups

Write-Output "hotkey event chain verified: Alt+1 settings, Alt+2 selection, Alt+4 input, Alt+3 screenshot"
